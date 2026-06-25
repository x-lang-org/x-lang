//! HIR → MIR lowering
//!
//! 该模块把 `x_hir::Hir` 降级为 `x_mir::MirModule`。
//!
//! AST -> HIR -> MIR -> LIR -> Backend
//!
//! 相比早期“结构保真”的最小实现，这里实现了运行示例所需的真实语义：
//! - class/record/enum 展开为 `MirStruct` + 合成构造器/方法函数
//! - 数组/字典字面量降级为运行时（`x_list_*` / `x_map_*`）构造
//! - `for each` 在可迭代为列表时降级为真实 CFG 循环
//! - `when is` / match 在枚举上降级为按 tag 的 `Switch` + 负载投影绑定
//! - 字符串 `+`/`++` 降级为运行时 `x_str_concat`
//! - `println`/`print` 统一降级为运行时 `x_print`（按静态类型装箱为 XValue）
//!
//! 复杂的格式化/集合逻辑放在 C 运行时 `library/runtime/xrt.c` 中，避免在
//! 各机器后端重复实现。

use std::collections::{HashMap, HashSet};
use x_hir::HirImportSymbol;

use crate::mir::*;
use x_hir::{
    Hir, HirBinaryOp, HirBlock, HirClassDecl, HirConstructorDecl, HirDeclaration, HirEnumDecl,
    HirEnumVariantData, HirExpression, HirFunctionDecl, HirLiteral, HirParameter, HirPattern,
    HirRecordDecl, HirStatement, HirType, HirUnaryOp,
};
use x_parser::ast::Pattern;

/// HIR 到 MIR 的 lowering 错误
#[derive(Debug, thiserror::Error)]
pub enum MirLowerError {
    #[error("不支持的 HIR 特性: {0}")]
    UnsupportedFeature(String),

    #[error("未定义变量: {0}")]
    UndefinedVariable(String),

    #[error("内部 lowering 错误: {0}")]
    Internal(String),
}

pub type MirLowerResult<T> = Result<T, MirLowerError>;

/// XValue 哨兵类型：表示一个装箱的动态值（运行时以指针表示）。
fn xvalue_ty() -> MirType {
    MirType::Struct("XValue".to_string(), Vec::new())
}

fn is_xvalue(ty: &MirType) -> bool {
    matches!(ty, MirType::Struct(name, _) if name == "XValue")
}

/// 类信息（字段布局 + 方法返回类型）
#[derive(Clone)]
struct ClassInfo {
    fields: Vec<(String, MirType)>,
    methods: HashMap<String, MirType>,
}

/// 枚举信息（变体顺序即 tag；记录每个变体的负载数量）
#[derive(Clone)]
struct EnumInfo {
    /// (变体名, 负载数量)，下标即 tag
    variants: Vec<(String, usize)>,
    /// 负载字段数量（所有变体的最大值）
    max_payload: usize,
    /// 变体名 -> 负载声明类型（HIR）。用于在 match 中投影负载为真实类型。
    payloads: HashMap<String, Vec<HirType>>,
    /// 类型参数顺序（按变体负载中泛型名首次出现推断，如 Result -> [T, E]）。
    type_params: Vec<String>,
}

/// 全局类型环境
#[derive(Default, Clone)]
struct TypeCtx {
    classes: HashMap<String, ClassInfo>,
    enums: HashMap<String, EnumInfo>,
    records: HashMap<String, Vec<(String, MirType)>>,
    functions: HashMap<String, MirType>,
    globals: HashMap<String, MirType>,
    /// 变体名 -> (枚举名, 负载数量)。用于裸构造器（Some/None/Ok/Err 等）。
    variant_to_enum: HashMap<String, (String, usize)>,
}

impl TypeCtx {
    fn enum_tag(&self, enum_name: &str, variant: &str) -> Option<usize> {
        self.enums
            .get(enum_name)?
            .variants
            .iter()
            .position(|(n, _)| n == variant)
    }

    /// 若 `name` 是某个枚举的变体名，返回 (枚举名, 负载数量)。
    fn variant_lookup(&self, name: &str) -> Option<(String, usize)> {
        self.variant_to_enum.get(name).cloned()
    }
}

/// 将整个 HIR 程序 lowering 为 MIR 模块
pub fn lower_hir_to_mir(hir: &Hir) -> MirLowerResult<MirModule> {
    let mut lowerer = HirToMirLowerer::new(&hir.module_name);

    // 1) 预扫描：收集类型环境（函数/类/枚举/记录）
    lowerer.prescan(hir);

    // 2) 顶层声明
    for decl in &hir.declarations {
        lowerer.lower_declaration(decl)?;
    }

    // 3) 顶层语句 -> 合成 main
    if !hir.statements.is_empty() || !lowerer.globals_needing_init.is_empty() {
        lowerer.lower_toplevel_statements_as_main(&hir.statements)?;
    }

    let mut module = lowerer.finish();
    // 4) 整程序死函数消除：仅保留从 main 可达的函数（以及外部声明），
    //    避免链接未使用的库函数（及其依赖的内建符号）。
    eliminate_dead_functions(&mut module);
    Ok(module)
}

/// 收集操作数中引用的全局符号名。
fn collect_globals_in_operand(op: &MirOperand, out: &mut Vec<String>) {
    if let MirOperand::Global(n) = op {
        out.push(n.clone());
    }
}

fn collect_globals_in_instr(instr: &MirInstruction, out: &mut Vec<String>) {
    match instr {
        MirInstruction::Assign { value, .. } => collect_globals_in_operand(value, out),
        MirInstruction::BinaryOp { left, right, .. } => {
            collect_globals_in_operand(left, out);
            collect_globals_in_operand(right, out);
        }
        MirInstruction::UnaryOp { operand, .. } => collect_globals_in_operand(operand, out),
        MirInstruction::Call { func, args, .. } => {
            collect_globals_in_operand(func, out);
            for a in args {
                collect_globals_in_operand(a, out);
            }
        }
        MirInstruction::FieldAccess { object, .. } => collect_globals_in_operand(object, out),
        MirInstruction::SetField { object, value, .. } => {
            collect_globals_in_operand(object, out);
            collect_globals_in_operand(value, out);
        }
        MirInstruction::ArrayAccess { array, index, .. } => {
            collect_globals_in_operand(array, out);
            collect_globals_in_operand(index, out);
        }
        MirInstruction::Alloc { .. } => {}
        MirInstruction::Load { ptr, .. } => collect_globals_in_operand(ptr, out),
        MirInstruction::Store { ptr, value } => {
            collect_globals_in_operand(ptr, out);
            collect_globals_in_operand(value, out);
        }
        MirInstruction::Cast { value, .. } => collect_globals_in_operand(value, out),
        MirInstruction::Dup { src, .. } => collect_globals_in_operand(src, out),
        MirInstruction::Drop { value } => collect_globals_in_operand(value, out),
        MirInstruction::Reuse { src, .. } => collect_globals_in_operand(src, out),
        MirInstruction::WhenGuard {
            condition, body, ..
        } => {
            collect_globals_in_operand(condition, out);
            collect_globals_in_operand(body, out);
        }
    }
}

fn collect_globals_in_term(term: &MirTerminator, out: &mut Vec<String>) {
    match term {
        MirTerminator::CondBranch { cond, .. } => collect_globals_in_operand(cond, out),
        MirTerminator::Return { value: Some(v) } => collect_globals_in_operand(v, out),
        MirTerminator::Switch { value, .. } => collect_globals_in_operand(value, out),
        _ => {}
    }
}

/// 删除从 `main` 不可达的（非外部）函数。若模块没有 `main`（如库），保持不变。
fn eliminate_dead_functions(module: &mut MirModule) {
    if !module.functions.iter().any(|f| f.name == "main") {
        return;
    }
    let func_names: HashSet<String> = module
        .functions
        .iter()
        .filter(|f| !f.is_extern)
        .map(|f| f.name.clone())
        .collect();

    let mut reachable: HashSet<String> = HashSet::new();
    let mut work = vec!["main".to_string()];
    while let Some(name) = work.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        if let Some(f) = module.functions.iter().find(|f| f.name == name) {
            let mut globals = Vec::new();
            for b in &f.blocks {
                for i in &b.instructions {
                    collect_globals_in_instr(i, &mut globals);
                }
                collect_globals_in_term(&b.terminator, &mut globals);
            }
            for g in globals {
                if func_names.contains(&g) && !reachable.contains(&g) {
                    work.push(g);
                }
            }
        }
    }

    module
        .functions
        .retain(|f| f.is_extern || reachable.contains(&f.name));
}

/// 内部 lowering 状态
struct HirToMirLowerer {
    module: MirModule,
    ctx: TypeCtx,
    /// 需要在 main 中初始化的全局变量（非字面量初始化器）
    globals_needing_init: Vec<(String, HirType, HirExpression)>,
}

impl HirToMirLowerer {
    fn new(module_name: &str) -> Self {
        Self {
            module: MirModule::new(module_name),
            ctx: TypeCtx::default(),
            globals_needing_init: Vec::new(),
        }
    }

    fn finish(self) -> MirModule {
        self.module
    }

    // ------------------------------------------------------------------
    // 预扫描：建立类型环境
    // ------------------------------------------------------------------
    fn prescan(&mut self, hir: &Hir) {
        for decl in &hir.declarations {
            match decl {
                HirDeclaration::Function(f) => {
                    self.ctx
                        .functions
                        .insert(f.name.clone(), lower_type(&f.return_type));
                }
                HirDeclaration::ExternFunction(f) => {
                    self.ctx
                        .functions
                        .insert(f.name.clone(), lower_type(&f.return_type));
                }
                HirDeclaration::Class(c) => {
                    let fields = c
                        .fields
                        .iter()
                        .map(|fld| (fld.name.clone(), field_repr_ty(&fld.ty)))
                        .collect::<Vec<_>>();
                    let mut methods = HashMap::new();
                    for m in &c.methods {
                        methods.insert(m.name.clone(), lower_type(&m.return_type));
                    }
                    self.ctx
                        .classes
                        .insert(c.name.clone(), ClassInfo { fields, methods });
                }
                HirDeclaration::Enum(e) => {
                    let mut payloads: HashMap<String, Vec<HirType>> = HashMap::new();
                    let variants: Vec<(String, usize)> = e
                        .variants
                        .iter()
                        .map(|v| {
                            let tys: Vec<HirType> = match &v.data {
                                HirEnumVariantData::Unit => Vec::new(),
                                HirEnumVariantData::Tuple(t) => t.clone(),
                                HirEnumVariantData::Record(r) => {
                                    r.iter().map(|(_, t)| t.clone()).collect()
                                }
                            };
                            let arity = tys.len();
                            payloads.insert(v.name.clone(), tys);
                            (v.name.clone(), arity)
                        })
                        .collect();
                    let max_payload = variants.iter().map(|(_, a)| *a).max().unwrap_or(0);
                    // 按负载中泛型名首次出现推断类型参数顺序。
                    let mut type_params: Vec<String> = Vec::new();
                    for (vname, _) in &variants {
                        if let Some(tys) = payloads.get(vname) {
                            for t in tys {
                                if let HirType::Generic(g) = t {
                                    if !type_params.contains(g) {
                                        type_params.push(g.clone());
                                    }
                                }
                            }
                        }
                    }
                    for (vname, arity) in &variants {
                        self.ctx
                            .variant_to_enum
                            .entry(vname.clone())
                            .or_insert_with(|| (e.name.clone(), *arity));
                    }
                    self.ctx.enums.insert(
                        e.name.clone(),
                        EnumInfo {
                            variants,
                            max_payload,
                            payloads,
                            type_params,
                        },
                    );
                }
                HirDeclaration::Record(r) => {
                    // 记录“语义”字段类型（保留 Bool/Char），供成员类型推断与装箱使用；
                    // 内存布局所需的统一表示在 lower_record 中再折叠。
                    let fields = r
                        .fields
                        .iter()
                        .map(|(n, t)| (n.clone(), lower_type(t)))
                        .collect::<Vec<_>>();
                    self.ctx.records.insert(r.name.clone(), fields);
                }
                _ => {}
            }
        }

        // 全局变量类型：按源序推断（依赖前面已登记的函数/类/枚举）
        for decl in &hir.declarations {
            if let HirDeclaration::Variable(var) = decl {
                let ty = var
                    .initializer
                    .as_ref()
                    .map(|init| self.infer_type(init))
                    .filter(|t| !matches!(t, MirType::Unknown))
                    .unwrap_or_else(|| lower_type(&var.ty));
                self.ctx.globals.insert(var.name.clone(), ty);
            }
        }
    }

    // ------------------------------------------------------------------
    // 顶层类型推断（仅依赖 ctx，不依赖局部作用域）
    // ------------------------------------------------------------------
    fn infer_type(&self, expr: &HirExpression) -> MirType {
        infer_expr_type(expr, &self.ctx, None)
    }

    // ------------------------------------------------------------------
    // 声明 lowering
    // ------------------------------------------------------------------
    fn lower_declaration(&mut self, decl: &HirDeclaration) -> MirLowerResult<()> {
        match decl {
            HirDeclaration::Function(func) => {
                let mir_func = FunctionLowerer::lower_function(func, None, &self.ctx)?;
                self.module.functions.push(mir_func);
            }
            HirDeclaration::Variable(var) => {
                let ty = self
                    .ctx
                    .globals
                    .get(&var.name)
                    .cloned()
                    .unwrap_or_else(|| lower_type(&var.ty));

                let init = if let Some(expr) = &var.initializer {
                    match expr {
                        HirExpression::Literal(lit) if is_simple_global_literal(lit) => {
                            Some(lower_literal_to_constant(lit))
                        }
                        _ => {
                            self.globals_needing_init.push((
                                var.name.clone(),
                                var.ty.clone(),
                                expr.clone(),
                            ));
                            None
                        }
                    }
                } else {
                    None
                };

                self.module.globals.push(MirGlobal {
                    name: var.name.clone(),
                    ty,
                    initializer: init,
                    mutable: var.is_mutable,
                });
            }
            HirDeclaration::ExternFunction(ext) => {
                let mir_func = MirFunction {
                    name: ext.name.clone(),
                    type_params: Vec::new(),
                    parameters: ext
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(index, p)| MirParameter {
                            name: p.name.clone(),
                            ty: lower_type(&p.ty),
                            index,
                        })
                        .collect(),
                    return_type: lower_type(&ext.return_type),
                    blocks: Vec::new(),
                    locals: HashMap::new(),
                    name_to_local: HashMap::new(),
                    is_extern: true,
                };
                self.module.functions.push(mir_func);
            }
            HirDeclaration::Import(import_decl) => {
                self.module.imports.push(crate::mir::Import {
                    module_path: import_decl.module_path.clone(),
                    symbols: import_decl
                        .symbols
                        .iter()
                        .map(|sym| match sym {
                            HirImportSymbol::All => (String::new(), None),
                            HirImportSymbol::Named(name, alias) => (name.clone(), alias.clone()),
                        })
                        .collect(),
                    import_all: import_decl
                        .symbols
                        .iter()
                        .any(|sym| matches!(sym, HirImportSymbol::All)),
                });
            }
            HirDeclaration::Class(class) => {
                self.lower_class(class)?;
            }
            HirDeclaration::Enum(enm) => {
                self.lower_enum(enm);
            }
            HirDeclaration::Record(rec) => {
                self.lower_record(rec);
            }
            HirDeclaration::Trait(_)
            | HirDeclaration::Effect(_)
            | HirDeclaration::Implement
            | HirDeclaration::TypeAlias(_)
            | HirDeclaration::Newtype(_)
            | HirDeclaration::Module(_)
            | HirDeclaration::Export(_) => {
                // 这些声明属于类型层，或当前阶段无需在 MIR 显式建模。
            }
        }

        Ok(())
    }

    /// 把 class 展开为：MirStruct + 合成构造器(以类名命名) + 方法(Class__method)
    fn lower_class(&mut self, class: &HirClassDecl) -> MirLowerResult<()> {
        let info = self.ctx.classes.get(&class.name).cloned().ok_or_else(|| {
            MirLowerError::Internal(format!("类未在预扫描中登记: {}", class.name))
        })?;

        // 1) 结构体布局
        self.module.structs.push(MirStruct {
            name: class.name.clone(),
            fields: info.fields.clone(),
        });

        // 2) 构造器（命名为类名本身）
        let ctor = class.constructors.first();
        let mir_ctor = self.synthesize_constructor(&class.name, &info, ctor)?;
        self.module.functions.push(mir_ctor);

        // 3) 方法（Class__method，首参为 self）
        for method in &class.methods {
            let mangled = format!("{}__{}", class.name, method.name);
            let mut decl = method.clone();
            decl.name = mangled;
            let self_ty = HirType::Record(class.name.clone(), Vec::new());
            decl.parameters.insert(
                0,
                HirParameter {
                    name: "self".to_string(),
                    ty: self_ty,
                    default: None,
                },
            );
            let mir_method = FunctionLowerer::lower_function(&decl, None, &self.ctx)?;
            self.module.functions.push(mir_method);
        }

        Ok(())
    }

    fn synthesize_constructor(
        &self,
        class_name: &str,
        info: &ClassInfo,
        ctor: Option<&HirConstructorDecl>,
    ) -> MirLowerResult<MirFunction> {
        let params: Vec<HirParameter> = ctor.map(|c| c.parameters.clone()).unwrap_or_default();

        // 合成构造器体：
        //   let self = alloc(C);
        //   <ctor body, with self.field = ...>
        //   return self
        let mut lowerer = FunctionLowerer::new(
            MirFunction {
                name: class_name.to_string(),
                type_params: Vec::new(),
                parameters: params
                    .iter()
                    .enumerate()
                    .map(|(index, p)| MirParameter {
                        name: p.name.clone(),
                        ty: lower_type(&p.ty),
                        index,
                    })
                    .collect(),
                return_type: MirType::Struct(class_name.to_string(), Vec::new()),
                blocks: Vec::new(),
                locals: HashMap::new(),
                name_to_local: HashMap::new(),
                is_extern: false,
            },
            &self.ctx,
        );

        // self = malloc(size)
        let self_local = lowerer.new_local(MirType::Struct(class_name.to_string(), Vec::new()));
        lowerer.bind_local("self".to_string(), self_local);
        lowerer.var_types.insert(
            "self".to_string(),
            MirType::Struct(class_name.to_string(), Vec::new()),
        );
        let size = (info.fields.len().max(1)) * 8;
        lowerer
            .current_block
            .instructions
            .push(MirInstruction::Alloc {
                dest: self_local,
                ty: MirType::Struct(class_name.to_string(), Vec::new()),
                size,
            });

        // 构造器体
        if let Some(ctor) = ctor {
            for stmt in &ctor.body.statements {
                lowerer.lower_statement(stmt)?;
            }
        }

        lowerer.current_block.terminator = MirTerminator::Return {
            value: Some(MirOperand::Local(self_local)),
        };
        lowerer
            .function
            .blocks
            .push(std::mem::take(&mut lowerer.current_block));
        Ok(lowerer.function)
    }

    fn lower_enum(&mut self, enm: &HirEnumDecl) {
        let info = match self.ctx.enums.get(&enm.name) {
            Some(i) => i.clone(),
            None => return,
        };
        let mut fields = vec![("tag".to_string(), MirType::Int(64))];
        for i in 0..info.max_payload {
            fields.push((format!("payload{}", i), xvalue_ty()));
        }
        self.module.structs.push(MirStruct {
            name: enm.name.clone(),
            fields,
        });
    }

    fn lower_record(&mut self, rec: &HirRecordDecl) {
        // 布局使用统一表示（Bool/Char/Int 折叠为 8 字节槽）。
        let fields = self
            .ctx
            .records
            .get(&rec.name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(n, t)| (n, repr_of(t)))
            .collect();
        self.module.structs.push(MirStruct {
            name: rec.name.clone(),
            fields,
        });
    }

    fn lower_toplevel_statements_as_main(
        &mut self,
        statements: &[HirStatement],
    ) -> MirLowerResult<()> {
        // 为需要非字面量初始化的全局变量生成赋值语句
        let mut init_statements = Vec::new();
        for (name, _ty, init_expr) in &self.globals_needing_init {
            init_statements.push(HirStatement::Expression(HirExpression::Assign(
                Box::new(HirExpression::Variable(name.clone())),
                Box::new(init_expr.clone()),
            )));
        }

        let mut all_statements = init_statements;
        all_statements.extend(statements.iter().cloned());

        let synthetic = HirFunctionDecl {
            name: "main".to_string(),
            type_params: Vec::new(),
            parameters: Vec::<HirParameter>::new(),
            return_type: HirType::Int,
            body: HirBlock {
                statements: all_statements,
            },
            is_async: false,
            effects: Vec::new(),
        };

        let global_names: Vec<String> =
            self.module.globals.iter().map(|g| g.name.clone()).collect();
        let mut mir = FunctionLowerer::lower_function(&synthetic, Some(&global_names), &self.ctx)?;

        // main 总是 return 0，避免把 println 等返回值当作退出码
        if let Some(last) = mir.blocks.last_mut() {
            last.terminator = MirTerminator::Return {
                value: Some(MirOperand::Constant(MirConstant::Int(0))),
            };
        }

        self.module.functions.push(mir);
        Ok(())
    }
}

/// 循环上下文：`break`/`continue` 的跳转目标
struct LoopCtx {
    continue_target: usize,
    break_target: usize,
}

struct FunctionLowerer<'ctx> {
    function: MirFunction,
    current_block: MirBasicBlock,
    next_local: MirLocalId,
    next_block_id: usize,
    scopes: Vec<HashMap<String, MirLocalId>>,
    loop_stack: Vec<LoopCtx>,
    /// 局部/参数名 -> 类型
    var_types: HashMap<String, MirType>,
    ctx: &'ctx TypeCtx,
}

impl<'ctx> FunctionLowerer<'ctx> {
    fn new(function: MirFunction, ctx: &'ctx TypeCtx) -> Self {
        let mut var_types = HashMap::new();
        for p in &function.parameters {
            var_types.insert(p.name.clone(), p.ty.clone());
        }
        Self {
            function,
            current_block: MirBasicBlock {
                id: 0,
                instructions: Vec::new(),
                terminator: MirTerminator::Unreachable,
            },
            next_local: 0,
            next_block_id: 1,
            scopes: vec![HashMap::new()],
            loop_stack: Vec::new(),
            var_types,
            ctx,
        }
    }

    fn lower_function(
        func: &HirFunctionDecl,
        global_names: Option<&[String]>,
        ctx: &'ctx TypeCtx,
    ) -> MirLowerResult<MirFunction> {
        let type_params = func
            .type_params
            .iter()
            .map(|name| TypeParameter { name: name.clone() })
            .collect();

        let function = MirFunction {
            name: func.name.clone(),
            type_params,
            parameters: func
                .parameters
                .iter()
                .enumerate()
                .map(|(index, p)| MirParameter {
                    name: p.name.clone(),
                    ty: lower_type(&p.ty),
                    index,
                })
                .collect(),
            return_type: lower_type(&func.return_type),
            blocks: Vec::new(),
            locals: HashMap::new(),
            name_to_local: HashMap::new(),
            is_extern: false,
        };

        let mut lowerer = Self::new(function, ctx);

        // 全局变量作用域标记
        if let Some(globals) = global_names {
            for name in globals {
                lowerer.scopes[0].insert(name.clone(), MirLocalId::MAX);
            }
        }

        let last_expr_value = lowerer.lower_block(&func.body)?;

        if matches!(lowerer.current_block.terminator, MirTerminator::Unreachable) {
            let return_value =
                last_expr_value.or_else(|| default_return_value(&lowerer.function.return_type));
            lowerer.current_block.terminator = MirTerminator::Return {
                value: return_value,
            };
        }

        lowerer.function.blocks.push(lowerer.current_block);
        Ok(lowerer.function)
    }

    fn alloc_block_id(&mut self) -> usize {
        let id = self.next_block_id;
        self.next_block_id += 1;
        id
    }

    fn block_open(&self) -> bool {
        matches!(self.current_block.terminator, MirTerminator::Unreachable)
    }

    fn switch_to_block(&mut self, id: usize) {
        let finished = std::mem::replace(
            &mut self.current_block,
            MirBasicBlock {
                id,
                instructions: Vec::new(),
                terminator: MirTerminator::Unreachable,
            },
        );
        self.function.blocks.push(finished);
    }

    fn close_open_with_branch(&mut self, target: usize) {
        if self.block_open() {
            self.current_block.terminator = MirTerminator::Branch { target };
        }
    }

    fn lower_block(&mut self, block: &HirBlock) -> MirLowerResult<Option<MirOperand>> {
        self.push_scope();
        let mut last_expr_value: Option<MirOperand> = None;
        for stmt in &block.statements {
            if let Some(value) = self.lower_statement(stmt)? {
                last_expr_value = Some(value);
            } else {
                last_expr_value = None;
            }
        }
        self.pop_scope();
        Ok(last_expr_value)
    }

    fn lower_statement(&mut self, stmt: &HirStatement) -> MirLowerResult<Option<MirOperand>> {
        match stmt {
            HirStatement::Expression(expr) => {
                let value = self.lower_expression(expr)?;
                Ok(Some(value))
            }
            HirStatement::Variable(var) => {
                let ty = var
                    .initializer
                    .as_ref()
                    .map(|init| self.type_of(init))
                    .filter(|t| !matches!(t, MirType::Unknown))
                    .unwrap_or_else(|| lower_type(&var.ty));
                let local = self.new_local(ty.clone());
                self.bind_local(var.name.clone(), local);
                self.var_types.insert(var.name.clone(), ty);

                if let Some(init) = &var.initializer {
                    let value = self.lower_expression(init)?;
                    self.current_block
                        .instructions
                        .push(MirInstruction::Assign { dest: local, value });
                }
                Ok(None)
            }
            HirStatement::Return(expr) => {
                let value = expr
                    .as_ref()
                    .map(|e| self.lower_expression(e))
                    .transpose()?;
                self.current_block.terminator = MirTerminator::Return { value };
                Ok(None)
            }
            HirStatement::If(if_stmt) => {
                let cond = self.lower_expression(&if_stmt.condition)?;
                let then_id = self.alloc_block_id();
                let merge_id = self.alloc_block_id();
                let else_id = if if_stmt.else_block.is_some() {
                    self.alloc_block_id()
                } else {
                    merge_id
                };

                self.current_block.terminator = MirTerminator::CondBranch {
                    cond,
                    then_block: then_id,
                    else_block: else_id,
                };
                self.switch_to_block(then_id);

                self.lower_block(&if_stmt.then_block)?;
                self.close_open_with_branch(merge_id);

                if let Some(else_block) = &if_stmt.else_block {
                    self.switch_to_block(else_id);
                    self.lower_block(else_block)?;
                    self.close_open_with_branch(merge_id);
                }
                self.switch_to_block(merge_id);
                Ok(None)
            }
            HirStatement::For(for_stmt) => {
                self.lower_for_each(for_stmt)?;
                Ok(None)
            }
            HirStatement::While(while_stmt) => {
                let header_id = self.alloc_block_id();
                let body_id = self.alloc_block_id();
                let exit_id = self.alloc_block_id();

                self.current_block.terminator = MirTerminator::Branch { target: header_id };
                self.switch_to_block(header_id);

                let cond = self.lower_expression(&while_stmt.condition)?;
                self.current_block.terminator = MirTerminator::CondBranch {
                    cond,
                    then_block: body_id,
                    else_block: exit_id,
                };
                self.switch_to_block(body_id);

                self.loop_stack.push(LoopCtx {
                    continue_target: header_id,
                    break_target: exit_id,
                });
                self.lower_block(&while_stmt.body)?;
                self.loop_stack.pop();

                self.close_open_with_branch(header_id);
                self.switch_to_block(exit_id);
                Ok(None)
            }
            HirStatement::Match(match_stmt) => {
                let norm: Vec<NormCase> = match_stmt
                    .cases
                    .iter()
                    .map(|c| NormCase {
                        kind: norm_from_hir(&c.pattern),
                        guard: c.guard.clone(),
                        body: c.body.clone(),
                    })
                    .collect();
                let v = self.lower_match(&match_stmt.expression, norm)?;
                Ok(v)
            }
            HirStatement::Try(try_stmt) => {
                self.lower_block(&try_stmt.body)?;
                for catch in &try_stmt.catch_clauses {
                    self.push_scope();
                    if let Some(var) = &catch.variable_name {
                        let local = self.new_local(MirType::Unknown);
                        self.bind_local(var.clone(), local);
                    }
                    self.lower_block(&catch.body)?;
                    self.pop_scope();
                }
                if let Some(finally_block) = &try_stmt.finally_block {
                    self.lower_block(finally_block)?;
                }
                Ok(None)
            }
            HirStatement::Break => {
                if let Some(ctx) = self.loop_stack.last() {
                    let target = ctx.break_target;
                    self.current_block.terminator = MirTerminator::Branch { target };
                    let dead = self.alloc_block_id();
                    self.switch_to_block(dead);
                }
                Ok(None)
            }
            HirStatement::Continue => {
                if let Some(ctx) = self.loop_stack.last() {
                    let target = ctx.continue_target;
                    self.current_block.terminator = MirTerminator::Branch { target };
                    let dead = self.alloc_block_id();
                    self.switch_to_block(dead);
                }
                Ok(None)
            }
            HirStatement::Unsafe(block) => self.lower_block(block),
            HirStatement::Defer(expr) => {
                let _ = self.lower_expression(expr)?;
                Ok(None)
            }
            HirStatement::Yield(_expr) => Ok(None),
            HirStatement::Loop(body) => {
                let header_id = self.alloc_block_id();
                let exit_id = self.alloc_block_id();

                self.current_block.terminator = MirTerminator::Branch { target: header_id };
                self.switch_to_block(header_id);

                self.loop_stack.push(LoopCtx {
                    continue_target: header_id,
                    break_target: exit_id,
                });
                self.lower_block(body)?;
                self.loop_stack.pop();

                self.close_open_with_branch(header_id);
                self.switch_to_block(exit_id);
                Ok(None)
            }
            HirStatement::WhenGuard(condition, body) => {
                // `when cond { body }`（无 else）是条件语句：仅当 cond 为真时执行 body。
                let cond = self.lower_expression(condition)?;
                let then_id = self.alloc_block_id();
                let merge_id = self.alloc_block_id();
                self.current_block.terminator = MirTerminator::CondBranch {
                    cond,
                    then_block: then_id,
                    else_block: merge_id,
                };
                self.switch_to_block(then_id);
                let _ = self.lower_expression(body)?;
                self.close_open_with_branch(merge_id);
                self.switch_to_block(merge_id);
                Ok(None)
            }
        }
    }

    /// `for each pat in iter { body }`
    /// 当 iter 静态类型可识别为列表(XValue)/数组时，降级为真实 CFG 循环：
    ///   len = x_list_len(iter); i = 0;
    ///   header: if i < len -> body else exit
    ///   body: item = x_list_get(iter, i); <body>; i = i + 1; -> header
    fn lower_for_each(&mut self, for_stmt: &x_hir::HirForStatement) -> MirLowerResult<()> {
        let iter_ty = self.type_of(&for_stmt.iterator);
        let is_listlike = is_xvalue(&iter_ty) || matches!(iter_ty, MirType::Array(_, _));

        if !is_listlike {
            // 回退：保守地求值迭代器并执行一次循环体（用于无法识别的可迭代对象，
            // 如 prelude 中对字符串的 for-each——这些是死代码，仅需通过 lowering）。
            let _ = self.lower_expression(&for_stmt.iterator)?;
            self.push_scope();
            self.bind_pattern(&for_stmt.pattern)?;
            self.lower_block(&for_stmt.body)?;
            self.pop_scope();
            return Ok(());
        }

        // iter 操作数
        let iter_op = self.lower_expression(&for_stmt.iterator)?;
        let iter_local = self.materialize(iter_op, xvalue_ty());

        // len = x_list_len(iter)
        let len_local = self.new_local(MirType::Int(64));
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(len_local),
            func: MirOperand::Global("x_list_len".to_string()),
            args: vec![MirOperand::Local(iter_local)],
        });

        // i = 0
        let idx_local = self.new_local(MirType::Int(64));
        self.current_block
            .instructions
            .push(MirInstruction::Assign {
                dest: idx_local,
                value: MirOperand::Constant(MirConstant::Int(0)),
            });

        let header_id = self.alloc_block_id();
        let body_id = self.alloc_block_id();
        let exit_id = self.alloc_block_id();

        self.current_block.terminator = MirTerminator::Branch { target: header_id };
        self.switch_to_block(header_id);

        // cond: i < len
        let cond_local = self.new_local(MirType::Bool);
        self.current_block
            .instructions
            .push(MirInstruction::BinaryOp {
                dest: cond_local,
                op: MirBinOp::Lt,
                left: MirOperand::Local(idx_local),
                right: MirOperand::Local(len_local),
            });
        self.current_block.terminator = MirTerminator::CondBranch {
            cond: MirOperand::Local(cond_local),
            then_block: body_id,
            else_block: exit_id,
        };
        self.switch_to_block(body_id);

        self.push_scope();
        // item = x_list_get(iter, i)  —— item 为 XValue
        let item_local = self.new_local(xvalue_ty());
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(item_local),
            func: MirOperand::Global("x_list_get".to_string()),
            args: vec![MirOperand::Local(iter_local), MirOperand::Local(idx_local)],
        });
        if let HirPattern::Variable(name) = &for_stmt.pattern {
            self.bind_local(name.clone(), item_local);
            self.var_types.insert(name.clone(), xvalue_ty());
        } else {
            self.bind_pattern(&for_stmt.pattern)?;
        }

        self.loop_stack.push(LoopCtx {
            continue_target: header_id,
            break_target: exit_id,
        });
        self.lower_block(&for_stmt.body)?;
        self.loop_stack.pop();
        self.pop_scope();

        // i = i + 1
        if self.block_open() {
            self.current_block
                .instructions
                .push(MirInstruction::BinaryOp {
                    dest: idx_local,
                    op: MirBinOp::Add,
                    left: MirOperand::Local(idx_local),
                    right: MirOperand::Constant(MirConstant::Int(1)),
                });
        }
        self.close_open_with_branch(header_id);
        self.switch_to_block(exit_id);
        Ok(())
    }

    fn lower_expression(&mut self, expr: &HirExpression) -> MirLowerResult<MirOperand> {
        match expr {
            HirExpression::Literal(lit) => Ok(MirOperand::Constant(lower_literal_to_constant(lit))),
            HirExpression::Variable(name) => {
                if let Some(local) = self.lookup_local(name) {
                    Ok(MirOperand::Local(local))
                } else if let Some(param_index) = self.lookup_param(name) {
                    Ok(MirOperand::Param(param_index))
                } else if let Some((enum_name, _)) = self.ctx.variant_lookup(name) {
                    // 裸的无参枚举变体，如 None
                    self.construct_enum(&enum_name, name, &[])
                } else {
                    Ok(MirOperand::Global(name.clone()))
                }
            }
            HirExpression::Member(object, field) => {
                // 枚举构造：EnumName.Variant （无参变体）
                if let HirExpression::Variable(tyname) = object.as_ref() {
                    if self.ctx.enums.contains_key(tyname) {
                        return self.construct_enum(tyname, field, &[]);
                    }
                }
                let object_op = self.lower_expression(object)?;
                let fty = self.member_type(object, field);
                let dest = self.new_local(fty);
                self.current_block
                    .instructions
                    .push(MirInstruction::FieldAccess {
                        dest,
                        object: object_op,
                        field: field.clone(),
                    });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Call(callee, args) => self.lower_call(callee, args),
            HirExpression::Binary(op, lhs, rhs) => self.lower_binary(op, lhs, rhs),
            HirExpression::Unary(op, e) => {
                let operand = self.lower_expression(e)?;
                let dest = self.new_local(self.type_of(e));
                self.current_block
                    .instructions
                    .push(MirInstruction::UnaryOp {
                        dest,
                        op: lower_unary_op(op)?,
                        operand,
                    });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Cast(e, ty) => {
                use x_parser::ast::Type as AstType;
                // 转换为字符串：装箱后调用运行时 x_to_str（处理 Int/Float/Bool -> string）。
                if matches!(ty, AstType::String) {
                    return self.as_cstr(e);
                }
                let target = match ty {
                    AstType::Int | AstType::UnsignedInt => Some(MirType::Int(64)),
                    AstType::Float => Some(MirType::Float(64)),
                    AstType::Bool => Some(MirType::Bool),
                    _ => None,
                };
                let src = self.type_of(e);
                let op = self.lower_expression(e)?;
                // 数值类型间转换（int<->float、宽度变化）发一条 Cast 指令；其余原样传递。
                let numeric =
                    |t: &MirType| matches!(t, MirType::Int(_) | MirType::Float(_) | MirType::Bool);
                if let Some(target) = target {
                    if numeric(&src) && src != target {
                        let dest = self.new_local(target.clone());
                        self.current_block.instructions.push(MirInstruction::Cast {
                            dest,
                            value: op,
                            ty: target,
                        });
                        return Ok(MirOperand::Local(dest));
                    }
                }
                Ok(op)
            }
            HirExpression::Assign(target, value) => self.lower_assign(target, value),
            HirExpression::If(cond, then_expr, else_expr) => {
                let result = self.new_local(self.type_of(then_expr));
                let cond_op = self.lower_expression(cond)?;
                let then_id = self.alloc_block_id();
                let else_id = self.alloc_block_id();
                let merge_id = self.alloc_block_id();

                self.current_block.terminator = MirTerminator::CondBranch {
                    cond: cond_op,
                    then_block: then_id,
                    else_block: else_id,
                };
                self.switch_to_block(then_id);
                let then_val = self.lower_expression(then_expr)?;
                self.current_block
                    .instructions
                    .push(MirInstruction::Assign {
                        dest: result,
                        value: then_val,
                    });
                self.close_open_with_branch(merge_id);
                self.switch_to_block(else_id);
                let else_val = self.lower_expression(else_expr)?;
                self.current_block
                    .instructions
                    .push(MirInstruction::Assign {
                        dest: result,
                        value: else_val,
                    });
                self.close_open_with_branch(merge_id);
                self.switch_to_block(merge_id);
                Ok(MirOperand::Local(result))
            }
            HirExpression::Lambda(_params, _body) => {
                // Lambda 暂不在此 MIR 阶段完整支持
                Ok(MirOperand::Constant(MirConstant::Null))
            }
            HirExpression::Array(items) => self.construct_list(items),
            HirExpression::Tuple(items) => self.construct_list(items),
            HirExpression::Dictionary(entries) => self.construct_map(entries),
            HirExpression::Record(name, fields) => self.construct_record(name, fields),
            HirExpression::Range(start, end, _) => {
                let _ = self.lower_expression(start)?;
                let _ = self.lower_expression(end)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Pipe(input, funcs) => {
                let mut current = self.lower_expression(input)?;
                for func in funcs {
                    let callee = self.lower_expression(func)?;
                    let dest = self.new_local(MirType::Unknown);
                    self.current_block.instructions.push(MirInstruction::Call {
                        dest: Some(dest),
                        func: callee,
                        args: vec![current],
                    });
                    current = MirOperand::Local(dest);
                }
                Ok(current)
            }
            HirExpression::Wait(_, exprs) => {
                let mut last = MirOperand::Constant(MirConstant::Unit);
                for expr in exprs {
                    last = self.lower_expression(expr)?;
                }
                Ok(last)
            }
            HirExpression::Needs(name) => {
                Ok(MirOperand::Constant(MirConstant::String(name.clone())))
            }
            HirExpression::Given(_, expr) => self.lower_expression(expr),
            HirExpression::Handle(expr, handlers) => {
                let _ = self.lower_expression(expr)?;
                for (_, handler) in handlers {
                    let _ = self.lower_expression(handler)?;
                }
                Ok(MirOperand::Constant(MirConstant::Unit))
            }
            HirExpression::TryPropagate(expr) => self.lower_expression(expr),
            HirExpression::Typed(expr, _) => self.lower_expression(expr),
            HirExpression::Match(discriminant, cases) => {
                let norm: Vec<NormCase> = cases
                    .iter()
                    .map(|(p, g, b)| NormCase {
                        kind: norm_from_parser(p),
                        guard: g.as_deref().cloned(),
                        body: b.clone(),
                    })
                    .collect();
                let result = self.lower_match(discriminant, norm)?;
                Ok(result.unwrap_or(MirOperand::Constant(MirConstant::Unit)))
            }
            HirExpression::Await(expr) => self.lower_expression(expr),
            HirExpression::OptionalChain(base, _member) => {
                let _ = self.lower_expression(base)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            HirExpression::NullCoalescing(left, right) => {
                let _ = self.lower_expression(left)?;
                let _ = self.lower_expression(right)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            HirExpression::WhenGuard(condition, body) => {
                // `when cond { body }` 作为表达式：cond 为真返回 body 值，否则返回 unit。
                let result = self.new_local(self.type_of(body));
                self.current_block
                    .instructions
                    .push(MirInstruction::Assign {
                        dest: result,
                        value: MirOperand::Constant(MirConstant::Unit),
                    });
                let cond = self.lower_expression(condition)?;
                let then_id = self.alloc_block_id();
                let merge_id = self.alloc_block_id();
                self.current_block.terminator = MirTerminator::CondBranch {
                    cond,
                    then_block: then_id,
                    else_block: merge_id,
                };
                self.switch_to_block(then_id);
                let body_val = self.lower_expression(body)?;
                self.current_block
                    .instructions
                    .push(MirInstruction::Assign {
                        dest: result,
                        value: body_val,
                    });
                self.close_open_with_branch(merge_id);
                self.switch_to_block(merge_id);
                Ok(MirOperand::Local(result))
            }
            HirExpression::Block(block) => {
                let v = self.lower_block(block)?;
                Ok(v.unwrap_or(MirOperand::Constant(MirConstant::Unit)))
            }
        }
    }

    // --- 调用 ---------------------------------------------------------
    fn lower_call(
        &mut self,
        callee: &HirExpression,
        args: &[HirExpression],
    ) -> MirLowerResult<MirOperand> {
        // 打印内建：println/print/eprintln/...
        if let HirExpression::Variable(name) = callee {
            match name.as_str() {
                "println" | "print" | "eprintln" | "eprint" | "print_inline" => {
                    return self.lower_print(name, args);
                }
                _ => {}
            }
            // 裸的带参枚举构造器，如 Some(x)/Ok(x)/Err(e)
            if self.lookup_local(name).is_none()
                && self.lookup_param(name).is_none()
                && !self.ctx.functions.contains_key(name)
            {
                if let Some((enum_name, _)) = self.ctx.variant_lookup(name) {
                    return self.construct_enum(&enum_name, name, args);
                }
            }
        }

        // 方法调用：obj.method(args) -> Class__method(obj, args)
        if let HirExpression::Member(obj, method) = callee {
            // 枚举构造：EnumName.Variant(args)
            if let HirExpression::Variable(tyname) = obj.as_ref() {
                if self.ctx.enums.contains_key(tyname) {
                    return self.construct_enum(tyname, method, args);
                }
                // 模块限定调用：module::func(args)。module 不是值/类型，
                // 而 func 是已知的自由函数，按自由函数调用降级。
                let is_value = self.lookup_local(tyname).is_some()
                    || self.lookup_param(tyname).is_some()
                    || self.ctx.classes.contains_key(tyname)
                    || self.ctx.records.contains_key(tyname);
                if !is_value && self.ctx.functions.contains_key(method) {
                    let ret = self
                        .ctx
                        .functions
                        .get(method)
                        .cloned()
                        .unwrap_or(MirType::Unknown);
                    let lowered_args = args
                        .iter()
                        .map(|a| self.lower_expression(a))
                        .collect::<MirLowerResult<Vec<_>>>()?;
                    let dest = self.new_local(ret);
                    self.current_block.instructions.push(MirInstruction::Call {
                        dest: Some(dest),
                        func: MirOperand::Global(method.clone()),
                        args: lowered_args,
                    });
                    return Ok(MirOperand::Local(dest));
                }
            }

            if let Some(class) = self.class_of_expr(obj) {
                if self
                    .ctx
                    .classes
                    .get(&class)
                    .is_some_and(|c| c.methods.contains_key(method))
                {
                    let mangled = format!("{}__{}", class, method);
                    let ret = self
                        .ctx
                        .classes
                        .get(&class)
                        .and_then(|c| c.methods.get(method))
                        .cloned()
                        .unwrap_or(MirType::Unknown);
                    let obj_op = self.lower_expression(obj)?;
                    let mut lowered_args = vec![obj_op];
                    for a in args {
                        lowered_args.push(self.lower_expression(a)?);
                    }
                    let dest = self.new_local(ret);
                    self.current_block.instructions.push(MirInstruction::Call {
                        dest: Some(dest),
                        func: MirOperand::Global(mangled),
                        args: lowered_args,
                    });
                    return Ok(MirOperand::Local(dest));
                }
            }

            // UFCS：obj.method(args) -> method(obj, args)，当 method 是已知自由函数
            // （记录/枚举的方法以 self 为首参的自由函数形式实现）。
            if self.ctx.functions.contains_key(method) {
                let ret = self
                    .ctx
                    .functions
                    .get(method)
                    .cloned()
                    .unwrap_or(MirType::Unknown);
                let obj_op = self.lower_expression(obj)?;
                let mut lowered_args = vec![obj_op];
                for a in args {
                    lowered_args.push(self.lower_expression(a)?);
                }
                let dest = self.new_local(ret);
                self.current_block.instructions.push(MirInstruction::Call {
                    dest: Some(dest),
                    func: MirOperand::Global(method.clone()),
                    args: lowered_args,
                });
                return Ok(MirOperand::Local(dest));
            }
        }

        // 普通调用（含构造器：Variable(ClassName)）
        let func = self.lower_expression(callee)?;
        let lowered_args = args
            .iter()
            .map(|arg| self.lower_expression(arg))
            .collect::<MirLowerResult<Vec<_>>>()?;
        let ret = self.call_return_type(callee);
        let dest = self.new_local(ret);
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(dest),
            func,
            args: lowered_args,
        });
        Ok(MirOperand::Local(dest))
    }

    fn call_return_type(&self, callee: &HirExpression) -> MirType {
        if let HirExpression::Variable(name) = callee {
            if let Some(rt) = self.ctx.functions.get(name) {
                return rt.clone();
            }
            if self.ctx.classes.contains_key(name) {
                return MirType::Struct(name.clone(), Vec::new());
            }
        }
        MirType::Unknown
    }

    /// println/print 统一降级为 x_print / x_print_inline，对实参按静态类型装箱。
    fn lower_print(&mut self, name: &str, args: &[HirExpression]) -> MirLowerResult<MirOperand> {
        let newline = !matches!(name, "print_inline" | "eprint");
        let runtime = if newline { "x_print" } else { "x_print_inline" };

        if args.is_empty() {
            // 仅打印换行
            self.current_block.instructions.push(MirInstruction::Call {
                dest: None,
                func: MirOperand::Global("x_print_newline".to_string()),
                args: vec![],
            });
            return Ok(MirOperand::Constant(MirConstant::Unit));
        }

        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                // 多实参以空格分隔
                let sp =
                    self.box_value_str(MirOperand::Constant(MirConstant::String(" ".to_string())));
                self.current_block.instructions.push(MirInstruction::Call {
                    dest: None,
                    func: MirOperand::Global("x_print_inline".to_string()),
                    args: vec![sp],
                });
            }
            let boxed = self.box_for_print(arg)?;
            let is_last = i + 1 == args.len();
            let func = if is_last { runtime } else { "x_print_inline" };
            self.current_block.instructions.push(MirInstruction::Call {
                dest: None,
                func: MirOperand::Global(func.to_string()),
                args: vec![boxed],
            });
        }
        Ok(MirOperand::Constant(MirConstant::Unit))
    }

    /// 把一个实参装箱为 XValue 操作数（用于打印）
    fn box_for_print(&mut self, arg: &HirExpression) -> MirLowerResult<MirOperand> {
        let ty = self.type_of(arg);
        if is_xvalue(&ty) {
            return self.lower_expression(arg);
        }
        let op = self.lower_expression(arg)?;
        Ok(self.box_scalar(op, &ty))
    }

    /// 把标量操作数装箱为 XValue
    fn box_scalar(&mut self, op: MirOperand, ty: &MirType) -> MirOperand {
        let func = match ty {
            MirType::Float(_) => "x_from_double",
            MirType::Bool => "x_from_bool",
            MirType::String => "x_from_str",
            MirType::Char => "x_from_char",
            MirType::Struct(name, _) if name == "XValue" => return op,
            MirType::Struct(_, _) | MirType::Pointer(_) => "x_from_ptr",
            _ => "x_from_int",
        };
        let dest = self.new_local(xvalue_ty());
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(dest),
            func: MirOperand::Global(func.to_string()),
            args: vec![op],
        });
        MirOperand::Local(dest)
    }

    fn box_value_str(&mut self, op: MirOperand) -> MirOperand {
        let dest = self.new_local(xvalue_ty());
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(dest),
            func: MirOperand::Global("x_from_str".to_string()),
            args: vec![op],
        });
        MirOperand::Local(dest)
    }

    // --- 二元运算（含字符串拼接） -------------------------------------
    fn lower_binary(
        &mut self,
        op: &HirBinaryOp,
        lhs: &HirExpression,
        rhs: &HirExpression,
    ) -> MirLowerResult<MirOperand> {
        let result_ty = binary_result_type(op, &self.type_of(lhs), &self.type_of(rhs));

        // 字符串拼接：Concat，或 Add 且结果为字符串
        let is_concat = matches!(op, HirBinaryOp::Concat)
            || (matches!(op, HirBinaryOp::Add) && matches!(result_ty, MirType::String));
        if is_concat {
            let left_s = self.as_cstr(lhs)?;
            let right_s = self.as_cstr(rhs)?;
            let dest = self.new_local(MirType::String);
            self.current_block.instructions.push(MirInstruction::Call {
                dest: Some(dest),
                func: MirOperand::Global("x_str_concat".to_string()),
                args: vec![left_s, right_s],
            });
            return Ok(MirOperand::Local(dest));
        }

        let lhs_op = self.lower_expression(lhs)?;
        let rhs_op = self.lower_expression(rhs)?;
        let dest = self.new_local(result_ty);
        self.current_block
            .instructions
            .push(MirInstruction::BinaryOp {
                dest,
                op: lower_binary_op(op)?,
                left: lhs_op,
                right: rhs_op,
            });
        Ok(MirOperand::Local(dest))
    }

    /// 把表达式转换为 C 字符串(char*)操作数：字符串原样；其它先装箱再 x_to_str
    fn as_cstr(&mut self, e: &HirExpression) -> MirLowerResult<MirOperand> {
        let ty = self.type_of(e);
        if matches!(ty, MirType::String) {
            return self.lower_expression(e);
        }
        let boxed = if is_xvalue(&ty) {
            self.lower_expression(e)?
        } else {
            let op = self.lower_expression(e)?;
            self.box_scalar(op, &ty)
        };
        let dest = self.new_local(MirType::String);
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(dest),
            func: MirOperand::Global("x_to_str".to_string()),
            args: vec![boxed],
        });
        Ok(MirOperand::Local(dest))
    }

    // --- 赋值 ---------------------------------------------------------
    fn lower_assign(
        &mut self,
        target: &HirExpression,
        value: &HirExpression,
    ) -> MirLowerResult<MirOperand> {
        let value_op = self.lower_expression(value)?;
        let target_inner = match target {
            HirExpression::Typed(inner, _) => inner.as_ref(),
            other => other,
        };
        match target_inner {
            HirExpression::Variable(name) => {
                if let Some(local) = self.lookup_local(name) {
                    self.current_block
                        .instructions
                        .push(MirInstruction::Assign {
                            dest: local,
                            value: value_op.clone(),
                        });
                    Ok(MirOperand::Local(local))
                } else if let Some(param_idx) = self.lookup_param(name) {
                    let dest = self.new_local(MirType::Unknown);
                    self.current_block
                        .instructions
                        .push(MirInstruction::Assign {
                            dest,
                            value: MirOperand::Param(param_idx),
                        });
                    Ok(MirOperand::Local(dest))
                } else if self.is_global(name) {
                    self.current_block.instructions.push(MirInstruction::Store {
                        ptr: MirOperand::Global(name.clone()),
                        value: value_op.clone(),
                    });
                    Ok(MirOperand::Global(name.clone()))
                } else {
                    Err(MirLowerError::UndefinedVariable(name.clone()))
                }
            }
            HirExpression::Member(obj, field) => {
                let obj_op = self.lower_expression(obj)?;
                self.current_block
                    .instructions
                    .push(MirInstruction::SetField {
                        object: obj_op,
                        field: field.clone(),
                        value: value_op.clone(),
                    });
                Ok(value_op)
            }
            _ => {
                let _ = self.lower_expression(target)?;
                Ok(value_op)
            }
        }
    }

    // --- 集合构造 -----------------------------------------------------
    fn construct_list(&mut self, items: &[HirExpression]) -> MirLowerResult<MirOperand> {
        let list = self.new_local(xvalue_ty());
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(list),
            func: MirOperand::Global("x_list_new".to_string()),
            args: vec![],
        });
        for item in items {
            let boxed = self.box_for_print(item)?;
            self.current_block.instructions.push(MirInstruction::Call {
                dest: None,
                func: MirOperand::Global("x_list_push".to_string()),
                args: vec![MirOperand::Local(list), boxed],
            });
        }
        Ok(MirOperand::Local(list))
    }

    fn construct_map(
        &mut self,
        entries: &[(HirExpression, HirExpression)],
    ) -> MirLowerResult<MirOperand> {
        let map = self.new_local(xvalue_ty());
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(map),
            func: MirOperand::Global("x_map_new".to_string()),
            args: vec![],
        });
        for (k, v) in entries {
            let key = self.box_for_print(k)?;
            let val = self.box_for_print(v)?;
            self.current_block.instructions.push(MirInstruction::Call {
                dest: None,
                func: MirOperand::Global("x_map_put".to_string()),
                args: vec![MirOperand::Local(map), key, val],
            });
        }
        Ok(MirOperand::Local(map))
    }

    fn construct_record(
        &mut self,
        name: &str,
        fields: &[(String, HirExpression)],
    ) -> MirLowerResult<MirOperand> {
        let nfields = self
            .ctx
            .records
            .get(name)
            .map(|f| f.len())
            .unwrap_or(fields.len())
            .max(1);
        let obj = self.new_local(MirType::Struct(name.to_string(), Vec::new()));
        self.current_block.instructions.push(MirInstruction::Alloc {
            dest: obj,
            ty: MirType::Struct(name.to_string(), Vec::new()),
            size: nfields * 8,
        });
        for (fname, value) in fields {
            let v = self.lower_expression(value)?;
            self.current_block
                .instructions
                .push(MirInstruction::SetField {
                    object: MirOperand::Local(obj),
                    field: fname.clone(),
                    value: v,
                });
        }
        Ok(MirOperand::Local(obj))
    }

    /// 构造枚举值：malloc, tag=<idx>, payloadN=box(arg)
    fn construct_enum(
        &mut self,
        enum_name: &str,
        variant: &str,
        args: &[HirExpression],
    ) -> MirLowerResult<MirOperand> {
        let tag = self.ctx.enum_tag(enum_name, variant).unwrap_or(0);
        let nfields = 1 + self
            .ctx
            .enums
            .get(enum_name)
            .map(|e| e.max_payload)
            .unwrap_or(args.len());
        let obj = self.new_local(MirType::Struct(enum_name.to_string(), Vec::new()));
        self.current_block.instructions.push(MirInstruction::Alloc {
            dest: obj,
            ty: MirType::Struct(enum_name.to_string(), Vec::new()),
            size: nfields * 8,
        });
        self.current_block
            .instructions
            .push(MirInstruction::SetField {
                object: MirOperand::Local(obj),
                field: "tag".to_string(),
                value: MirOperand::Constant(MirConstant::Int(tag as i64)),
            });
        for (i, arg) in args.iter().enumerate() {
            let boxed = self.box_for_print(arg)?;
            self.current_block
                .instructions
                .push(MirInstruction::SetField {
                    object: MirOperand::Local(obj),
                    field: format!("payload{}", i),
                    value: boxed,
                });
        }
        Ok(MirOperand::Local(obj))
    }

    // --- match / when is ---------------------------------------------
    fn lower_match(
        &mut self,
        discriminant: &HirExpression,
        cases: Vec<NormCase>,
    ) -> MirLowerResult<Option<MirOperand>> {
        // 识别枚举类型
        let enum_name = self.enum_of_expr(discriminant);

        if let Some(enum_name) = enum_name {
            return self.lower_enum_match(discriminant, &enum_name, cases);
        }

        // 非枚举：保守地求值并顺序执行（用于 prelude 中的其它 match 死代码）
        let _ = self.lower_expression(discriminant)?;
        for case in &cases {
            self.push_scope();
            match &case.kind {
                NormPat::Enum { bindings, .. } => {
                    for (name, _) in bindings {
                        let local = self.new_local(MirType::Unknown);
                        self.bind_local(name.clone(), local);
                    }
                }
                NormPat::Bind(name) => {
                    let local = self.new_local(MirType::Unknown);
                    self.bind_local(name.clone(), local);
                }
                _ => {}
            }
            if let Some(g) = &case.guard {
                let _ = self.lower_expression(g)?;
            }
            for stmt in &case.body.statements {
                self.lower_statement(stmt)?;
            }
            self.pop_scope();
        }
        Ok(None)
    }

    fn lower_enum_match(
        &mut self,
        discriminant: &HirExpression,
        enum_name: &str,
        cases: Vec<NormCase>,
    ) -> MirLowerResult<Option<MirOperand>> {
        // 判别式的具体类型实参（如 Result<Int, ErrorStack> -> [Int, ErrorStack]），
        // 用于把泛型负载投影为真实类型。
        let type_args: Vec<MirType> = match self.type_of(discriminant) {
            MirType::Struct(name, args) if name == enum_name => args,
            _ => Vec::new(),
        };

        let scrut_op = self.lower_expression(discriminant)?;
        let scrut_local =
            self.materialize(scrut_op, MirType::Struct(enum_name.to_string(), Vec::new()));

        // tag = scrut.tag
        let tag_local = self.new_local(MirType::Int(64));
        self.current_block
            .instructions
            .push(MirInstruction::FieldAccess {
                dest: tag_local,
                object: MirOperand::Local(scrut_local),
                field: "tag".to_string(),
            });

        let merge_id = self.alloc_block_id();
        let default_id = self.alloc_block_id();
        // match 作为表达式时，各分支的尾值写入同一个结果局部，merge 块读取它。
        let result_local = self.new_local(MirType::Unknown);
        let mut switch_cases: Vec<(MirConstant, usize)> = Vec::new();

        struct ArmPlan {
            block_id: usize,
            variant: Option<String>,
            bindings: Vec<(String, usize)>,
            whole_binding: Option<String>,
            guard: Option<HirExpression>,
            body: HirBlock,
        }
        let mut arms: Vec<ArmPlan> = Vec::new();
        let mut default_arm: Option<ArmPlan> = None;

        for case in cases {
            match case.kind {
                NormPat::Enum { variant, bindings } => {
                    let tag = self.ctx.enum_tag(enum_name, &variant).unwrap_or(0) as i64;
                    let block_id = self.alloc_block_id();
                    switch_cases.push((MirConstant::Int(tag), block_id));
                    arms.push(ArmPlan {
                        block_id,
                        variant: Some(variant),
                        bindings,
                        whole_binding: None,
                        guard: case.guard,
                        body: case.body,
                    });
                }
                NormPat::Wildcard | NormPat::Bind(_) => {
                    if default_arm.is_none() {
                        let whole_binding = match case.kind {
                            NormPat::Bind(n) => Some(n),
                            _ => None,
                        };
                        default_arm = Some(ArmPlan {
                            block_id: default_id,
                            variant: None,
                            bindings: Vec::new(),
                            whole_binding,
                            guard: case.guard,
                            body: case.body,
                        });
                    }
                }
                NormPat::Other => {}
            }
        }

        self.current_block.terminator = MirTerminator::Switch {
            value: MirOperand::Local(tag_local),
            cases: switch_cases,
            default: default_id,
        };

        // 生成每个具名分支
        for arm in &arms {
            self.switch_to_block(arm.block_id);
            self.push_scope();
            let payload_types = arm
                .variant
                .as_ref()
                .map(|v| self.payload_types(enum_name, v, &type_args))
                .unwrap_or_default();
            for (name, idx) in &arm.bindings {
                let payload_local = self.new_local(xvalue_ty());
                self.current_block
                    .instructions
                    .push(MirInstruction::FieldAccess {
                        dest: payload_local,
                        object: MirOperand::Local(scrut_local),
                        field: format!("payload{}", idx),
                    });
                // 把装箱的负载投影为其真实类型（用于字段访问/方法/转换）。
                let pty = payload_types.get(*idx).cloned().unwrap_or_else(xvalue_ty);
                let (bound_local, bound_ty) = self.unbox_payload(payload_local, &pty);
                self.bind_local(name.clone(), bound_local);
                self.var_types.insert(name.clone(), bound_ty);
            }
            if let Some(g) = &arm.guard {
                let _ = self.lower_expression(g)?;
            }
            let mut tail: Option<MirOperand> = None;
            for stmt in &arm.body.statements {
                tail = self.lower_statement(stmt)?;
            }
            if let Some(v) = tail {
                if self.block_open() {
                    self.current_block
                        .instructions
                        .push(MirInstruction::Assign {
                            dest: result_local,
                            value: v,
                        });
                }
            }
            self.pop_scope();
            self.close_open_with_branch(merge_id);
        }

        // 默认块
        self.switch_to_block(default_id);
        if let Some(arm) = &default_arm {
            self.push_scope();
            if let Some(name) = &arm.whole_binding {
                self.bind_local(name.clone(), scrut_local);
                self.var_types.insert(
                    name.clone(),
                    MirType::Struct(enum_name.to_string(), Vec::new()),
                );
            }
            if let Some(g) = &arm.guard {
                let _ = self.lower_expression(g)?;
            }
            let mut tail: Option<MirOperand> = None;
            for stmt in &arm.body.statements {
                tail = self.lower_statement(stmt)?;
            }
            if let Some(v) = tail {
                if self.block_open() {
                    self.current_block
                        .instructions
                        .push(MirInstruction::Assign {
                            dest: result_local,
                            value: v,
                        });
                }
            }
            self.pop_scope();
        }
        self.close_open_with_branch(merge_id);
        self.switch_to_block(merge_id);
        Ok(Some(MirOperand::Local(result_local)))
    }

    // --- 类型查询助手 -------------------------------------------------
    fn type_of(&self, expr: &HirExpression) -> MirType {
        infer_expr_type(expr, self.ctx, Some(&self.var_types))
    }

    fn member_type(&self, obj: &HirExpression, field: &str) -> MirType {
        if let Some(class) = self.class_of_expr(obj) {
            if let Some(info) = self.ctx.classes.get(&class) {
                if let Some((_, t)) = info.fields.iter().find(|(n, _)| n == field) {
                    return t.clone();
                }
            }
            if let Some(fields) = self.ctx.records.get(&class) {
                if let Some((_, t)) = fields.iter().find(|(n, _)| n == field) {
                    return t.clone();
                }
            }
        }
        MirType::Unknown
    }

    /// 解析某枚举变体的负载真实类型：把声明中的类型参数（如 T/E）替换为
    /// 判别式的具体类型实参。
    fn payload_types(&self, enum_name: &str, variant: &str, type_args: &[MirType]) -> Vec<MirType> {
        let Some(info) = self.ctx.enums.get(enum_name) else {
            return Vec::new();
        };
        let Some(decl_tys) = info.payloads.get(variant) else {
            return Vec::new();
        };
        decl_tys
            .iter()
            .map(|t| {
                if let HirType::Generic(g) = t {
                    if let Some(pos) = info.type_params.iter().position(|p| p == g) {
                        if let Some(ta) = type_args.get(pos) {
                            return ta.clone();
                        }
                    }
                    // 无法解析的泛型参数：保持装箱（XValue），让运行时按实际标签处理，
                    // 避免误用 x_as_ptr 把整数/布尔当指针拆箱。
                    return xvalue_ty();
                }
                lower_type(t)
            })
            .collect()
    }

    /// 把装箱的 XValue 负载拆箱为其真实表示；返回 (局部, 类型)。
    fn unbox_payload(&mut self, boxed: MirLocalId, pty: &MirType) -> (MirLocalId, MirType) {
        let (func, ret_ty) = match pty {
            MirType::Int(_) => ("x_as_int", MirType::Int(64)),
            MirType::Bool => ("x_as_bool", MirType::Bool),
            MirType::Float(_) => ("x_as_double", MirType::Float(64)),
            MirType::Char => ("x_as_int", MirType::Char),
            MirType::String => ("x_as_str", MirType::String),
            MirType::Struct(name, _) if name != "XValue" => ("x_as_ptr", pty.clone()),
            MirType::Pointer(_) => ("x_as_ptr", pty.clone()),
            // 已是 XValue / 未知：保持装箱。
            _ => return (boxed, xvalue_ty()),
        };
        let dest = self.new_local(ret_ty.clone());
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(dest),
            func: MirOperand::Global(func.to_string()),
            args: vec![MirOperand::Local(boxed)],
        });
        (dest, ret_ty)
    }

    fn class_of_expr(&self, expr: &HirExpression) -> Option<String> {
        match self.type_of(expr) {
            MirType::Struct(name, _) if name != "XValue" && name != "tuple" => Some(name),
            _ => None,
        }
    }

    fn enum_of_expr(&self, expr: &HirExpression) -> Option<String> {
        // 优先：Typed 注解为 Generic(EnumName) / 类型为 Named(EnumName)
        if let HirExpression::Typed(_, ty) = expr {
            if let Some(name) = hir_type_name(ty) {
                if self.ctx.enums.contains_key(&name) {
                    return Some(name);
                }
            }
        }
        match self.type_of(expr) {
            MirType::Struct(name, _) if self.ctx.enums.contains_key(&name) => Some(name),
            _ => None,
        }
    }

    /// 把操作数固化为局部变量（若已是局部则复用）
    fn materialize(&mut self, op: MirOperand, ty: MirType) -> MirLocalId {
        match op {
            MirOperand::Local(id) => id,
            other => {
                let id = self.new_local(ty);
                self.current_block
                    .instructions
                    .push(MirInstruction::Assign {
                        dest: id,
                        value: other,
                    });
                id
            }
        }
    }

    // --- 作用域/局部 --------------------------------------------------
    fn bind_pattern(&mut self, pattern: &HirPattern) -> MirLowerResult<()> {
        match pattern {
            HirPattern::Wildcard | HirPattern::Literal(_) => {}
            HirPattern::Variable(name) => {
                let local = self.new_local(MirType::Unknown);
                self.bind_local(name.clone(), local);
            }
            HirPattern::Array(items) | HirPattern::Tuple(items) => {
                for item in items {
                    self.bind_pattern(item)?;
                }
            }
            HirPattern::Dictionary(entries) => {
                for (k, v) in entries {
                    self.bind_pattern(k)?;
                    self.bind_pattern(v)?;
                }
            }
            HirPattern::Record(_, fields) => {
                for (_, pattern) in fields {
                    self.bind_pattern(pattern)?;
                }
            }
            HirPattern::Or(lhs, rhs) => {
                self.bind_pattern(lhs)?;
                self.bind_pattern(rhs)?;
            }
            HirPattern::EnumConstructor(_, _, args) => {
                for arg in args {
                    self.bind_pattern(arg)?;
                }
            }
        }
        Ok(())
    }

    fn new_local(&mut self, ty: MirType) -> MirLocalId {
        let id = self.next_local;
        self.next_local += 1;
        self.function.locals.insert(id, ty);
        id
    }

    fn bind_local(&mut self, name: String, local: MirLocalId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), local);
        }
        self.function.name_to_local.insert(name, local);
    }

    fn lookup_local(&self, name: &str) -> Option<MirLocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .filter(|&id| id != MirLocalId::MAX)
    }

    fn is_global(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .map(|id| id == MirLocalId::MAX)
            .unwrap_or(false)
            || self.ctx.globals.contains_key(name)
    }

    fn lookup_param(&self, name: &str) -> Option<usize> {
        self.function
            .parameters
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.index)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        let _ = self.scopes.pop();
    }
}

// ====================================================================
// 自由函数：类型推断与映射
// ====================================================================

/// 归一化的 match 分支模式（统一 HirPattern 与 parser Pattern 两种来源）
enum NormPat {
    /// 枚举构造器：变体名 + (绑定名, payload 下标)
    Enum {
        variant: String,
        bindings: Vec<(String, usize)>,
    },
    /// 顶层变量绑定（整体绑定，等价于通配并绑定）
    Bind(String),
    /// 通配
    Wildcard,
    /// 其它（字面量等，暂不支持精确分派）
    Other,
}

/// 归一化的 match 分支
struct NormCase {
    kind: NormPat,
    guard: Option<HirExpression>,
    body: HirBlock,
}

/// 从 HirPattern 归一化
fn norm_from_hir(p: &HirPattern) -> NormPat {
    match p {
        HirPattern::EnumConstructor(_, variant, fields) => {
            let mut bindings = Vec::new();
            for (i, f) in fields.iter().enumerate() {
                if let HirPattern::Variable(name) = f {
                    bindings.push((name.clone(), i));
                }
            }
            NormPat::Enum {
                variant: variant.clone(),
                bindings,
            }
        }
        HirPattern::Wildcard => NormPat::Wildcard,
        HirPattern::Variable(name) => NormPat::Bind(name.clone()),
        _ => NormPat::Other,
    }
}

/// 从 parser Pattern 归一化
fn norm_from_parser(p: &Pattern) -> NormPat {
    match p {
        Pattern::EnumConstructor(_, variant, fields) => {
            let mut bindings = Vec::new();
            for (i, f) in fields.iter().enumerate() {
                if let Pattern::Variable(name) = f {
                    bindings.push((name.clone(), i));
                }
            }
            NormPat::Enum {
                variant: variant.clone(),
                bindings,
            }
        }
        Pattern::Wildcard => NormPat::Wildcard,
        Pattern::Variable(name) => NormPat::Bind(name.clone()),
        _ => NormPat::Other,
    }
}

fn hir_type_name(ty: &HirType) -> Option<String> {
    match ty {
        HirType::Generic(name) => Some(name.clone()),
        HirType::Record(name, _) => Some(name.clone()),
        HirType::Union(name, _) => Some(name.clone()),
        HirType::TypeConstructor(name, _) => Some(name.clone()),
        _ => None,
    }
}

/// 表达式类型推断（ctx + 可选的局部类型表）
fn infer_expr_type(
    expr: &HirExpression,
    ctx: &TypeCtx,
    locals: Option<&HashMap<String, MirType>>,
) -> MirType {
    match expr {
        HirExpression::Literal(lit) => match lit {
            HirLiteral::Integer(_) => MirType::Int(64),
            HirLiteral::Float(_) => MirType::Float(64),
            HirLiteral::Boolean(_) => MirType::Bool,
            HirLiteral::String(_) => MirType::String,
            HirLiteral::Char(_) => MirType::Char,
            HirLiteral::Unit => MirType::Unit,
            HirLiteral::None => xvalue_ty(),
        },
        HirExpression::Variable(name) => locals
            .and_then(|m| m.get(name).cloned())
            .or_else(|| ctx.globals.get(name).cloned())
            .unwrap_or(MirType::Unknown),
        HirExpression::Typed(inner, ty) => {
            let from_ty = lower_type_opt(ty);
            if let Some(t) = from_ty {
                if !matches!(t, MirType::Unknown) {
                    return t;
                }
            }
            infer_expr_type(inner, ctx, locals)
        }
        HirExpression::Cast(inner, ty) => {
            use x_parser::ast::Type as AstType;
            match ty {
                AstType::String => MirType::String,
                AstType::Int | AstType::UnsignedInt => MirType::Int(64),
                AstType::Float => MirType::Float(64),
                AstType::Bool => MirType::Bool,
                _ => infer_expr_type(inner, ctx, locals),
            }
        }
        HirExpression::Binary(op, l, r) => binary_result_type(
            op,
            &infer_expr_type(l, ctx, locals),
            &infer_expr_type(r, ctx, locals),
        ),
        HirExpression::Unary(op, e) => match op {
            HirUnaryOp::Not => MirType::Bool,
            _ => infer_expr_type(e, ctx, locals),
        },
        HirExpression::Member(obj, field) => {
            // 枚举命名空间成员（EnumName.Variant）→ 该枚举类型
            if let HirExpression::Variable(tyname) = obj.as_ref() {
                if ctx.enums.contains_key(tyname) {
                    return MirType::Struct(tyname.clone(), Vec::new());
                }
            }
            let obj_ty = infer_expr_type(obj, ctx, locals);
            if let MirType::Struct(name, _) = obj_ty {
                if let Some(info) = ctx.classes.get(&name) {
                    if let Some((_, t)) = info.fields.iter().find(|(n, _)| n == field) {
                        return t.clone();
                    }
                }
                if let Some(fields) = ctx.records.get(&name) {
                    if let Some((_, t)) = fields.iter().find(|(n, _)| n == field) {
                        return t.clone();
                    }
                }
            }
            MirType::Unknown
        }
        HirExpression::Call(callee, _) => match callee.as_ref() {
            HirExpression::Variable(name) => {
                if let Some(rt) = ctx.functions.get(name) {
                    rt.clone()
                } else if ctx.classes.contains_key(name) {
                    MirType::Struct(name.clone(), Vec::new())
                } else {
                    MirType::Unknown
                }
            }
            HirExpression::Member(obj, method) => {
                // 枚举构造
                if let HirExpression::Variable(tyname) = obj.as_ref() {
                    if ctx.enums.contains_key(tyname) {
                        return MirType::Struct(tyname.clone(), Vec::new());
                    }
                }
                let obj_ty = infer_expr_type(obj, ctx, locals);
                if let MirType::Struct(name, _) = &obj_ty {
                    if let Some(info) = ctx.classes.get(name) {
                        if let Some(t) = info.methods.get(method) {
                            return t.clone();
                        }
                    }
                }
                // UFCS：obj.method() -> 自由函数 method 的返回类型
                if let Some(rt) = ctx.functions.get(method) {
                    return rt.clone();
                }
                MirType::Unknown
            }
            _ => MirType::Unknown,
        },
        HirExpression::Array(_) | HirExpression::Dictionary(_) | HirExpression::Tuple(_) => {
            xvalue_ty()
        }
        HirExpression::Record(name, _) => MirType::Struct(name.clone(), Vec::new()),
        HirExpression::If(_, t, _) => infer_expr_type(t, ctx, locals),
        HirExpression::Assign(_, v) => infer_expr_type(v, ctx, locals),
        HirExpression::Block(_) => MirType::Unknown,
        HirExpression::TryPropagate(e) | HirExpression::Await(e) | HirExpression::Given(_, e) => {
            infer_expr_type(e, ctx, locals)
        }
        _ => MirType::Unknown,
    }
}

/// 二元运算结果类型
fn binary_result_type(op: &HirBinaryOp, l: &MirType, r: &MirType) -> MirType {
    use HirBinaryOp::*;
    match op {
        Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual | And | Or => MirType::Bool,
        Concat => MirType::String,
        Add => {
            if matches!(l, MirType::String) || matches!(r, MirType::String) {
                MirType::String
            } else if matches!(l, MirType::Float(_)) || matches!(r, MirType::Float(_)) {
                MirType::Float(64)
            } else {
                MirType::Int(64)
            }
        }
        Sub | Mul | Div | Mod | Pow => {
            if matches!(l, MirType::Float(_)) || matches!(r, MirType::Float(_)) {
                MirType::Float(64)
            } else {
                MirType::Int(64)
            }
        }
        BitAnd | BitOr | BitXor | LeftShift | RightShift => MirType::Int(64),
    }
}

fn is_simple_global_literal(lit: &HirLiteral) -> bool {
    // 字符串/字符全局初始化器走 main 中赋值（需要 .rodata 指针）；
    // 标量直接做静态初始化。
    matches!(
        lit,
        HirLiteral::Integer(_) | HirLiteral::Float(_) | HirLiteral::Boolean(_)
    )
}

fn lower_literal_to_constant(lit: &HirLiteral) -> MirConstant {
    match lit {
        HirLiteral::Integer(v) => MirConstant::Int(*v),
        HirLiteral::Float(v) => MirConstant::Float(*v),
        HirLiteral::Boolean(v) => MirConstant::Bool(*v),
        HirLiteral::String(v) => MirConstant::String(v.clone()),
        HirLiteral::Char(v) => MirConstant::Char(*v),
        HirLiteral::Unit => MirConstant::Unit,
        HirLiteral::None => MirConstant::Null,
    }
}

/// 字段的内存表示类型：统一为 8 字节宽度，避免后端 8 字节存取破坏紧凑布局。
fn field_repr_ty(ty: &HirType) -> MirType {
    repr_of(lower_type(ty))
}

/// 把语义类型折叠为内存布局表示（标量统一为 8 字节槽）。
fn repr_of(ty: MirType) -> MirType {
    match ty {
        MirType::Float(_) => MirType::Float(64),
        MirType::Bool => MirType::Int(64),
        MirType::Char => MirType::Int(64),
        MirType::Int(_) => MirType::Int(64),
        MirType::String => MirType::String,
        other => other,
    }
}

fn lower_type_opt(ty: &HirType) -> Option<MirType> {
    let t = lower_type(ty);
    if matches!(t, MirType::Unknown) {
        None
    } else {
        Some(t)
    }
}

fn lower_type(ty: &HirType) -> MirType {
    match ty {
        HirType::Int => MirType::Int(64),
        HirType::UnsignedInt => MirType::Int(64),
        HirType::Float => MirType::Float(64),
        HirType::Bool => MirType::Bool,
        HirType::String | HirType::CString => MirType::String,
        HirType::Char | HirType::CChar => MirType::Char,
        HirType::Unit | HirType::Void | HirType::Never => MirType::Unit,

        // 集合在运行时统一表示为装箱的 XValue（x_list_* / x_map_*）
        HirType::Array(_) => xvalue_ty(),
        HirType::Dictionary(_, _) => xvalue_ty(),
        HirType::Record(name, _) => MirType::Struct(name.clone(), Vec::new()),
        HirType::Union(name, _) => MirType::Struct(name.clone(), Vec::new()),
        HirType::Tuple(types) => {
            MirType::Struct("tuple".to_string(), types.iter().map(lower_type).collect())
        }

        HirType::Function(params, ret) => MirType::Function(
            params.iter().map(lower_type).collect(),
            Box::new(lower_type(ret)),
        ),
        HirType::Async(inner) => lower_type(inner),

        // 命名（可能带类型参数）的类型，如 Result<T,E>/Option<T>/用户记录。
        // 表示为命名结构体，使枚举/记录识别（enum_of_expr 等）得以工作。
        // 对类型参数（如裸 T），下游会在 ctx 中查不到从而退化为未知，无副作用。
        HirType::TypeConstructor(name, args) => {
            MirType::Struct(name.clone(), args.iter().map(lower_type).collect())
        }
        HirType::Generic(name) => MirType::Struct(name.clone(), Vec::new()),

        HirType::TypeParam(_) | HirType::Dynamic | HirType::Unknown => MirType::Unknown,

        HirType::Reference(inner)
        | HirType::MutableReference(inner)
        | HirType::Pointer(inner)
        | HirType::ConstPointer(inner)
        | HirType::MutPointer(inner) => MirType::Pointer(Box::new(lower_type(inner))),

        HirType::CInt
        | HirType::CUInt
        | HirType::CLong
        | HirType::CULong
        | HirType::CLongLong
        | HirType::CULongLong
        | HirType::CSize => MirType::Int(64),

        HirType::CFloat | HirType::CDouble => MirType::Float(64),
    }
}

fn lower_binary_op(op: &HirBinaryOp) -> MirLowerResult<MirBinOp> {
    Ok(match op {
        HirBinaryOp::Add => MirBinOp::Add,
        HirBinaryOp::Sub => MirBinOp::Sub,
        HirBinaryOp::Mul => MirBinOp::Mul,
        HirBinaryOp::Div => MirBinOp::Div,
        HirBinaryOp::Mod => MirBinOp::Mod,
        HirBinaryOp::Equal => MirBinOp::Eq,
        HirBinaryOp::NotEqual => MirBinOp::Ne,
        HirBinaryOp::Less => MirBinOp::Lt,
        HirBinaryOp::LessEqual => MirBinOp::Le,
        HirBinaryOp::Greater => MirBinOp::Gt,
        HirBinaryOp::GreaterEqual => MirBinOp::Ge,
        HirBinaryOp::And => MirBinOp::And,
        HirBinaryOp::Or => MirBinOp::Or,
        HirBinaryOp::BitAnd => MirBinOp::BitAnd,
        HirBinaryOp::BitOr => MirBinOp::BitOr,
        HirBinaryOp::BitXor => MirBinOp::BitXor,
        HirBinaryOp::LeftShift => MirBinOp::Shl,
        HirBinaryOp::RightShift => MirBinOp::Shr,
        HirBinaryOp::Concat => {
            return Err(MirLowerError::Internal(
                "Concat 应已在 lower_binary 中处理".to_string(),
            ))
        }
        HirBinaryOp::Pow => MirBinOp::Mul,
    })
}

fn lower_unary_op(op: &HirUnaryOp) -> MirLowerResult<MirUnOp> {
    Ok(match op {
        HirUnaryOp::Negate => MirUnOp::Neg,
        HirUnaryOp::Not => MirUnOp::Not,
        HirUnaryOp::BitNot => MirUnOp::BitNot,
        HirUnaryOp::Await => MirUnOp::Neg, // 不应到达；保守占位
        HirUnaryOp::Reference => MirUnOp::Reference,
        HirUnaryOp::MutableReference => MirUnOp::MutableReference,
    })
}

fn default_return_value(ty: &MirType) -> Option<MirOperand> {
    match ty {
        MirType::Unit => None,
        MirType::Bool => Some(MirOperand::Constant(MirConstant::Bool(false))),
        MirType::Int(_) => Some(MirOperand::Constant(MirConstant::Int(0))),
        MirType::Float(_) => Some(MirOperand::Constant(MirConstant::Float(0.0))),
        MirType::String => Some(MirOperand::Constant(MirConstant::String(String::new()))),
        MirType::Char => Some(MirOperand::Constant(MirConstant::Char('\0'))),
        MirType::Pointer(_) => Some(MirOperand::Constant(MirConstant::Null)),
        MirType::Array(_, _)
        | MirType::Struct(_, _)
        | MirType::Function(_, _)
        | MirType::Unknown => Some(MirOperand::Constant(MirConstant::Unit)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_env() -> x_hir::HirTypeEnv {
        x_hir::HirTypeEnv {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
        }
    }

    #[test]
    fn lower_empty_hir_to_empty_module() {
        let hir = Hir {
            module_name: "main".to_string(),
            declarations: vec![],
            statements: vec![],
            type_env: empty_env(),
            perceus_info: x_hir::HirPerceusInfo::default(),
        };

        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        assert_eq!(mir.name, "main");
        assert!(mir.functions.is_empty());
        assert!(mir.globals.is_empty());
    }

    #[test]
    fn lower_toplevel_statement_creates_main() {
        let hir = Hir {
            module_name: "main".to_string(),
            declarations: vec![],
            statements: vec![HirStatement::Expression(HirExpression::Literal(
                HirLiteral::Integer(1),
            ))],
            type_env: empty_env(),
            perceus_info: x_hir::HirPerceusInfo::default(),
        };

        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        assert_eq!(mir.functions.len(), 1);
        assert_eq!(mir.functions[0].name, "main");
    }

    fn function_decl_hir(name: &str, body: Vec<HirStatement>) -> Hir {
        Hir {
            module_name: "main".to_string(),
            declarations: vec![HirDeclaration::Function(HirFunctionDecl {
                name: name.to_string(),
                type_params: Vec::new(),
                parameters: Vec::new(),
                return_type: HirType::Int,
                body: HirBlock { statements: body },
                is_async: false,
                effects: Vec::new(),
            })],
            statements: vec![],
            type_env: empty_env(),
            perceus_info: x_hir::HirPerceusInfo::default(),
        }
    }

    #[test]
    fn lower_if_statement_builds_cfg() {
        let cond = HirExpression::Binary(
            HirBinaryOp::Greater,
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(0))),
        );
        let if_stmt = HirStatement::If(x_hir::HirIfStatement {
            condition: cond,
            then_block: HirBlock {
                statements: vec![HirStatement::Return(Some(HirExpression::Literal(
                    HirLiteral::Integer(7),
                )))],
            },
            else_block: None,
        });
        let hir = function_decl_hir("f", vec![if_stmt]);

        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        let func = &mir.functions[0];
        assert!(func.blocks.len() >= 2);
        assert!(func
            .blocks
            .iter()
            .any(|b| matches!(b.terminator, MirTerminator::CondBranch { .. })));
    }

    #[test]
    fn lower_while_statement_builds_loop() {
        let cond = HirExpression::Binary(
            HirBinaryOp::Less,
            Box::new(HirExpression::Literal(HirLiteral::Integer(0))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(5))),
        );
        let while_stmt = HirStatement::While(x_hir::HirWhileStatement {
            condition: cond,
            body: HirBlock { statements: vec![] },
        });
        let hir = function_decl_hir("f", vec![while_stmt]);

        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        let func = &mir.functions[0];

        let header_ids: Vec<usize> = func
            .blocks
            .iter()
            .filter(|b| matches!(b.terminator, MirTerminator::CondBranch { .. }))
            .map(|b| b.id)
            .collect();
        assert!(!header_ids.is_empty());
        let has_back_edge = func.blocks.iter().any(|b| {
            matches!(&b.terminator, MirTerminator::Branch { target } if header_ids.contains(target))
        });
        assert!(has_back_edge);
    }
}
