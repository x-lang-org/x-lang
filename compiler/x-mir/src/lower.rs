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

fn mir_stdlib_ufcs_name(recv: &MirType, method: &str) -> Option<&'static str> {
    match (recv, method) {
        (MirType::String, "length") => Some("string_length"),
        (MirType::String, "substring") => Some("string_substring"),
        (MirType::String, "contains") => Some("string_contains"),
        (MirType::String, "trim") => Some("string_trim"),
        (MirType::String, "split") => Some("string_split"),
        (MirType::String, "to_upper" | "toUpperCase") => Some("string_to_upper"),
        (MirType::String, "to_lower" | "toLowerCase") => Some("string_to_lower"),
        (MirType::Array(_, _), "length") => Some("array_length"),
        (MirType::Array(_, _), "push") => Some("array_push"),
        // XValue lists (heterogeneous runtime arrays)
        (MirType::Struct(name, _), "length") if name == "XValue" => Some("array_length"),
        (MirType::Struct(name, _), "push") if name == "XValue" => Some("array_push"),
        // Pointer to XValue (also an XValue handle)
        (MirType::Pointer(inner), "length") if matches!(inner.as_ref(), MirType::Struct(name, _) if name == "XValue") => Some("array_length"),
        (MirType::Pointer(inner), "push") if matches!(inner.as_ref(), MirType::Struct(name, _) if name == "XValue") => Some("array_push"),
        (MirType::Float(_), "sqrt") => Some("sqrt"),
        (MirType::Float(_), "floor") => Some("floor"),
        (MirType::Float(_), "ceil") => Some("ceil"),
        (MirType::Float(_), "abs") => Some("fabs"),
        (MirType::Float(_), "pow") => Some("pow"),
        (MirType::Int(_), "abs") => Some("abs"),
        _ => None,
    }
}

fn mir_types_compatible(a: &MirType, b: &MirType) -> bool {
    match (a, b) {
        (MirType::Unknown, _) | (_, MirType::Unknown) => true,
        (MirType::Unit, MirType::Unit) => true,
        (MirType::Bool, MirType::Bool) => true,
        (MirType::Char, MirType::Char) => true,
        (MirType::String, MirType::String) => true,
        (MirType::Int(_), MirType::Int(_)) => true,
        (MirType::Float(_), MirType::Float(_)) => true,
        (MirType::Pointer(_), MirType::Pointer(_)) => true,
        (MirType::Struct(a, _), MirType::Struct(b, _)) => a == b,
        _ => a == b,
    }
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
    /// Function name → return type.
    functions: HashMap<String, MirType>,
    /// Function name → parameter types (for boxing scalars into `any` / XValue).
    function_params: HashMap<String, Vec<MirType>>,
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
        // Pass 1: enums (so class fields / function signatures can treat them as heap ptrs).
        for decl in &hir.declarations {
            if let HirDeclaration::Enum(e) = decl {
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
        }

        // Pass 2: classes / records / functions (class+enum names → pointers).
        for decl in &hir.declarations {
            match decl {
                HirDeclaration::Function(f) => {
                    self.ctx
                        .functions
                        .insert(f.name.clone(), lower_user_type(&f.return_type, &self.ctx));
                    self.ctx.function_params.insert(
                        f.name.clone(),
                        f.parameters
                            .iter()
                            .map(|p| lower_user_type(&p.ty, &self.ctx))
                            .collect(),
                    );
                }
                HirDeclaration::ExternFunction(f) => {
                    self.ctx
                        .functions
                        .insert(f.name.clone(), lower_user_type(&f.return_type, &self.ctx));
                    self.ctx.function_params.insert(
                        f.name.clone(),
                        f.parameters
                            .iter()
                            .map(|p| lower_user_type(&p.ty, &self.ctx))
                            .collect(),
                    );
                }
                HirDeclaration::Class(c) => {
                    let fields = c
                        .fields
                        .iter()
                        .map(|fld| (fld.name.clone(), field_repr_ty(&fld.ty, &self.ctx)))
                        .collect::<Vec<_>>();
                    let mut methods = HashMap::new();
                    for m in &c.methods {
                        methods.insert(m.name.clone(), lower_user_type(&m.return_type, &self.ctx));
                    }
                    self.ctx
                        .classes
                        .insert(c.name.clone(), ClassInfo { fields, methods });
                }
                HirDeclaration::Record(r) => {
                    let fields = r
                        .fields
                        .iter()
                        .map(|(n, t)| (n.clone(), lower_user_type(t, &self.ctx)))
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
                    .unwrap_or_else(|| lower_user_type(&var.ty, &self.ctx));
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
                    extern_abi: var.extern_abi.clone(),
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

    /// 把 class 展开为：MirStruct + 合成构造器(Class__new) + 方法(Class__method)
    fn lower_class(&mut self, class: &HirClassDecl) -> MirLowerResult<()> {
        let info = self.ctx.classes.get(&class.name).cloned().ok_or_else(|| {
            MirLowerError::Internal(format!("类未在预扫描中登记: {}", class.name))
        })?;

        // 1) 结构体布局
        self.module.structs.push(MirStruct {
            name: class.name.clone(),
            fields: info.fields.clone(),
        });

        // 2) 构造器（Class__new，避免与类型名冲突）
        let ctor = class.constructors.first();
        let mir_ctor = self.synthesize_constructor(&class.name, &info, ctor)?;
        self.module.functions.push(mir_ctor);

        // 3) 方法（Class__method，首参为 self；lookup 时 this 别名到 self）
        for method in &class.methods {
            let mangled = format!("{}__{}", class.name, method.name);
            let mut decl = method.clone();
            decl.name = mangled;
            let self_ty = HirType::Pointer(Box::new(HirType::Record(class.name.clone(), Vec::new())));
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
        //   let self = alloc(C);  // this 与 self 同绑定；类实例以指针表示
        //   <ctor body, with this.field = ...>
        //   return self
        let class_ty = MirType::Struct(class_name.to_string(), Vec::new());
        let self_ty = MirType::Pointer(Box::new(class_ty.clone()));
        let mut lowerer = FunctionLowerer::new(
            MirFunction {
                name: format!("{}__new", class_name),
                type_params: Vec::new(),
                parameters: params
                    .iter()
                    .enumerate()
                    .map(|(index, p)| MirParameter {
                        name: p.name.clone(),
                        ty: lower_user_type(&p.ty, &self.ctx),
                        index,
                    })
                    .collect(),
                return_type: self_ty.clone(),
                blocks: Vec::new(),
                locals: HashMap::new(),
                name_to_local: HashMap::new(),
                is_extern: false,
            },
            &self.ctx,
        );

        // self/this = malloc(size)
        let self_local = lowerer.new_local(self_ty.clone());
        lowerer.bind_local("self".to_string(), self_local);
        lowerer.bind_local("this".to_string(), self_local);
        lowerer.var_types.insert("self".to_string(), self_ty.clone());
        lowerer.var_types.insert("this".to_string(), self_ty.clone());
        let size = (info.fields.len().max(1)) * 8;
        lowerer
            .current_block
            .instructions
            .push(MirInstruction::Alloc {
                dest: self_local,
                ty: class_ty,
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
                    ty: lower_user_type(&p.ty, ctx),
                    index,
                })
                .collect(),
            return_type: lower_user_type(&func.return_type, ctx),
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
                // Prefer the declared type when present. Initializer inference
                // of `null` is Unit/Unknown and must not erase `*T` annotations.
                let declared = lower_user_type(&var.ty, &self.ctx);
                let from_init = var
                    .initializer
                    .as_ref()
                    .map(|init| self.type_of(init))
                    .filter(|t| !matches!(t, MirType::Unknown | MirType::Unit));
                let ty = match (&declared, from_init) {
                    (MirType::Unknown | MirType::Unit, Some(t)) => t,
                    (d, _) if !matches!(d, MirType::Unknown) => declared,
                    (_, Some(t)) => t,
                    _ => declared,
                };
                let local = self.new_local(ty.clone());
                self.bind_local(var.name.clone(), local);
                self.var_types.insert(var.name.clone(), ty.clone());

                if let Some(init) = &var.initializer {
                    let value = match init {
                        // `null` is lowered as Unit in HIR; materialize as Null for pointers.
                        HirExpression::Literal(HirLiteral::Unit)
                            if matches!(ty, MirType::Pointer(_)) =>
                        {
                            MirOperand::Constant(MirConstant::Null)
                        }
                        _ => self.lower_expression(init)?,
                    };
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

                // If both arms end in an expression value, treat the if as a
                // value-producing expression (needed for `function f() -> T {
                //   if c { e1 } else { e2 } }` without an explicit return).
                // Only yield when arm types are compatible — statement-level
                // ifs often end in assignments of different types (bool vs int).
                let result_ty = {
                    if !matches!(self.function.return_type, MirType::Unit | MirType::Unknown) {
                        self.function.return_type.clone()
                    } else {
                        MirType::Unknown
                    }
                };
                let result = self.new_local(result_ty);

                self.current_block.terminator = MirTerminator::CondBranch {
                    cond,
                    then_block: then_id,
                    else_block: else_id,
                };
                self.switch_to_block(then_id);

                let then_val = self.lower_block(&if_stmt.then_block)?;
                let then_open = self.block_open();
                self.close_open_with_branch(merge_id);

                let mut else_val = None;
                let mut else_open = false;
                if let Some(else_block) = &if_stmt.else_block {
                    self.switch_to_block(else_id);
                    else_val = self.lower_block(else_block)?;
                    else_open = self.block_open();
                    self.close_open_with_branch(merge_id);
                }
                self.switch_to_block(merge_id);

                let yield_pair = match (&then_val, &else_val) {
                    (Some(t), Some(e)) if then_open && else_open => {
                        let tt = self.operand_mir_type(t);
                        let et = self.operand_mir_type(e);
                        if mir_types_compatible(&tt, &et) {
                            Some((t.clone(), e.clone(), tt))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some((tv, ev, ty)) = yield_pair {
                    if !matches!(ty, MirType::Unknown | MirType::Unit) {
                        self.function.locals.insert(result, ty);
                    }
                    self.append_assign_in_block(
                        then_id,
                        MirInstruction::Assign {
                            dest: result,
                            value: tv,
                        },
                    );
                    self.append_assign_in_block(
                        else_id,
                        MirInstruction::Assign {
                            dest: result,
                            value: ev,
                        },
                    );
                    Ok(Some(MirOperand::Local(result)))
                } else {
                    Ok(None)
                }
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

        // Determine the element type of the loop variable
        let elem_ty = match &iter_ty {
            MirType::Array(inner, _) => *inner.clone(),
            _ => xvalue_ty(), // XValue list - element is XValue
        };

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
            // Use the element type for the loop variable, not XValue
            self.var_types.insert(name.clone(), elem_ty);
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
                // Check if this is a known method call (like .length, .push, etc.)
                // and generate a Call instead of a FieldAccess.
                let obj_ty = self.type_of(object);
                if let Some(free_name) = mir_stdlib_ufcs_name(&obj_ty, field) {
                    if self.ctx.functions.contains_key(free_name) {
                        // Generate a Call to the free function
                        let args: Vec<HirExpression> = vec![*object.clone()];
                        return self.lower_call(
                            &HirExpression::Variable(free_name.to_string()),
                            &args,
                        );
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
                let dest_ty = match op {
                    HirUnaryOp::Reference | HirUnaryOp::MutableReference => {
                        let inner = match &operand {
                            MirOperand::Local(id) => self
                                .function
                                .locals
                                .get(id)
                                .cloned()
                                .unwrap_or_else(|| self.type_of(e)),
                            _ => self.type_of(e),
                        };
                        MirType::Pointer(Box::new(inner))
                    }
                    _ => self.type_of(e),
                };
                let dest = self.new_local(dest_ty);
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
                    AstType::Int | AstType::UnsignedInt(_) | AstType::IntSized(_) => Some(MirType::Int(64)),
                    AstType::Float => Some(MirType::Float(64)),
                    AstType::Bool => Some(MirType::Bool),
                    AstType::Char | AstType::CChar => Some(MirType::Char),
                    AstType::Generic(name) if self.ctx.classes.contains_key(name) => Some(
                        MirType::Pointer(Box::new(MirType::Struct(name.clone(), Vec::new()))),
                    ),
                    AstType::Generic(name) if self.ctx.enums.contains_key(name) => Some(
                        MirType::Pointer(Box::new(MirType::Struct(name.clone(), Vec::new()))),
                    ),
                    _ => None,
                };
                let src = self.type_of(e);
                let op = self.lower_expression(e)?;
                // 数值类型间转换（int<->float、宽度变化）发一条 Cast 指令；其余原样传递。
                let numeric =
                    |t: &MirType| matches!(t, MirType::Int(_) | MirType::Float(_) | MirType::Bool | MirType::Char);
                if let Some(target) = target {
                    if numeric(&src) && numeric(&target) && src != target {
                        let dest = self.new_local(target.clone());
                        self.current_block.instructions.push(MirInstruction::Cast {
                            dest,
                            value: op,
                            ty: target,
                        });
                        return Ok(MirOperand::Local(dest));
                    }
                    // XValue / opaque → class/enum pointer
                    if matches!(
                        &target,
                        MirType::Pointer(inner)
                            if matches!(inner.as_ref(), MirType::Struct(n, _) if n != "XValue")
                    ) {
                        let dest = self.new_local(target.clone());
                        self.current_block.instructions.push(MirInstruction::Call {
                            dest: Some(dest),
                            func: MirOperand::Global("x_as_ptr".to_string()),
                            args: vec![op],
                        });
                        return Ok(MirOperand::Local(dest));
                    }
                    // XValue → float/int/bool/char unbox
                    if !numeric(&src) {
                        let (func, ret) = match &target {
                            MirType::Float(_) => ("x_as_double", target.clone()),
                            MirType::Int(_) => ("x_as_int", target.clone()),
                            MirType::Char => ("x_as_int", MirType::Char),
                            MirType::Bool => ("x_as_bool", MirType::Int(64)),
                            _ => {
                                return Ok(op);
                            }
                        };
                        let dest = self.new_local(ret);
                        self.current_block.instructions.push(MirInstruction::Call {
                            dest: Some(dest),
                            func: MirOperand::Global(func.to_string()),
                            args: vec![op],
                        });
                        if matches!(target, MirType::Bool) {
                            let bdest = self.new_local(MirType::Bool);
                            self.current_block.instructions.push(MirInstruction::BinaryOp {
                                dest: bdest,
                                op: MirBinOp::Ne,
                                left: MirOperand::Local(dest),
                                right: MirOperand::Constant(MirConstant::Int(0)),
                            });
                            return Ok(MirOperand::Local(bdest));
                        }
                        return Ok(MirOperand::Local(dest));
                    }
                }
                Ok(op)
            }
            HirExpression::Assign(target, value) => self.lower_assign(target, value),
            HirExpression::If(cond, then_expr, else_expr) => {
                let then_ty = self.type_of(then_expr);
                let else_ty = self.type_of(else_expr);
                let result_ty = if !matches!(then_ty, MirType::Unknown | MirType::Unit) {
                    then_ty
                } else if !matches!(else_ty, MirType::Unknown | MirType::Unit) {
                    else_ty
                } else if !matches!(
                    self.function.return_type,
                    MirType::Unit | MirType::Unknown
                ) {
                    self.function.return_type.clone()
                } else {
                    MirType::Unknown
                };
                let result = self.new_local(result_ty);
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
                    let params = self.params_of(method);
                    let mut lowered_args = Vec::with_capacity(args.len());
                    for (i, a) in args.iter().enumerate() {
                        lowered_args.push(self.lower_arg_for_param(a, params.get(i).cloned())?);
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

            // UFCS：obj.method(args) → free_fn(obj, args)
            // Prefer prelude aliases (length → string_length) based on receiver type.
            let ufcs_name = {
                let recv_ty = self.type_of(obj);
                mir_stdlib_ufcs_name(&recv_ty, method)
                    .map(|s| s.to_string())
                    .or_else(|| {
                        if self.ctx.functions.contains_key(method) {
                            Some(method.clone())
                        } else {
                            None
                        }
                    })
            };
            if let Some(free_name) = ufcs_name {
                if self.ctx.functions.contains_key(&free_name) {
                    let ret = self
                        .ctx
                        .functions
                        .get(&free_name)
                        .cloned()
                        .unwrap_or(MirType::Unknown);
                    let params = self.params_of(&free_name);
                    let obj_op = self.lower_expression(obj)?;
                    // UFCS inserts the receiver as the first formal parameter.
                    let mut lowered_args = vec![obj_op];
                    for (i, a) in args.iter().enumerate() {
                        // `array_push(list, item)` stores boxed XValues.
                        if free_name == "array_push" && i == 0 {
                            lowered_args.push(self.box_for_print(a)?);
                        } else {
                            // Arg i maps to formal parameter i+1 (after self).
                            lowered_args.push(
                                self.lower_arg_for_param(a, params.get(i + 1).cloned())?,
                            );
                        }
                    }
                    let dest = self.new_local(ret);
                    self.current_block.instructions.push(MirInstruction::Call {
                        dest: Some(dest),
                        func: MirOperand::Global(free_name),
                        args: lowered_args,
                    });
                    return Ok(MirOperand::Local(dest));
                }
            }
        }

        // 普通调用（含构造器：Variable(ClassName) → Class__new）
        let (func, runtime_name) = if let HirExpression::Variable(name) = callee {
            if self.lookup_local(name).is_none()
                && self.lookup_param(name).is_none()
                && self.ctx.classes.contains_key(name)
            {
                (MirOperand::Global(format!("{}__new", name)), "")
            } else {
                (self.lower_expression(callee)?, name.as_str())
            }
        } else {
            (self.lower_expression(callee)?, "")
        };

        // Dictionary indexing: dict[key] with string/non-int key → x_map_get.
        if runtime_name == "__index__" && args.len() == 2 {
            let idx_ty = self.type_of(&args[1]);
            if matches!(idx_ty, MirType::String)
                || !matches!(idx_ty, MirType::Int(_) | MirType::Unknown | MirType::Char)
            {
                let map = self.lower_expression(&args[0])?;
                let key = self.box_for_print(&args[1])?;
                let dest = self.new_local(xvalue_ty());
                self.current_block.instructions.push(MirInstruction::Call {
                    dest: Some(dest),
                    func: MirOperand::Global("x_map_get".to_string()),
                    args: vec![map, key],
                });
                return Ok(MirOperand::Local(dest));
            }
        }

        // Representation FFI: list push stores boxed XValues. Prelude
        // `array_push` / runtime `x_list_push` both need a boxed item.
        let lowered_args = if (runtime_name == "x_list_push" || runtime_name == "array_push")
            && args.len() == 2
        {
            vec![
                self.lower_expression(&args[0])?,
                self.box_for_print(&args[1])?,
            ]
        } else if runtime_name == "__index__" && args.len() == 2 {
            // List/string index: box collection if needed; keep int index.
            vec![
                self.lower_expression(&args[0])?,
                self.lower_expression(&args[1])?,
            ]
        } else {
            let params = if runtime_name.is_empty() {
                Vec::new()
            } else {
                self.params_of(runtime_name)
            };
            let mut out = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                out.push(self.lower_arg_for_param(arg, params.get(i).cloned())?);
            }
            out
        };
        let ret = if runtime_name == "x_list_push" || runtime_name == "array_push" {
            MirType::Unit
        } else {
            self.call_return_type(callee)
        };
        let dest = if matches!(ret, MirType::Unit) {
            None
        } else {
            Some(self.new_local(ret))
        };
        self.current_block.instructions.push(MirInstruction::Call {
            dest,
            func,
            args: lowered_args,
        });
        Ok(match dest {
            Some(d) => MirOperand::Local(d),
            None => MirOperand::Constant(MirConstant::Unit),
        })
    }

    fn call_return_type(&self, callee: &HirExpression) -> MirType {
        if let HirExpression::Variable(name) = callee {
            if name == "__index__" {
                return xvalue_ty();
            }
            if let Some(rt) = self.ctx.functions.get(name) {
                return rt.clone();
            }
            if self.ctx.classes.contains_key(name) {
                return MirType::Pointer(Box::new(MirType::Struct(name.clone(), Vec::new())));
            }
        }
        MirType::Unknown
    }

    fn operand_mir_type(&self, op: &MirOperand) -> MirType {
        match op {
            MirOperand::Local(id) => self
                .function
                .locals
                .get(id)
                .cloned()
                .unwrap_or(MirType::Unknown),
            MirOperand::Param(idx) => self
                .function
                .parameters
                .get(*idx)
                .map(|p| p.ty.clone())
                .unwrap_or(MirType::Unknown),
            MirOperand::Constant(c) => match c {
                MirConstant::Int(_) => MirType::Int(64),
                MirConstant::Float(_) => MirType::Float(64),
                MirConstant::Bool(_) => MirType::Bool,
                MirConstant::Char(_) => MirType::Char,
                MirConstant::String(_) => MirType::String,
                MirConstant::Null => MirType::Unknown,
                MirConstant::Unit => MirType::Unit,
            },
            MirOperand::Global(_) => MirType::Unknown,
        }
    }

    fn append_assign_in_block(&mut self, block_id: usize, instr: MirInstruction) {
        if self.current_block.id == block_id {
            self.current_block.instructions.push(instr);
            return;
        }
        if let Some(block) = self.function.blocks.iter_mut().find(|b| b.id == block_id) {
            block.instructions.push(instr);
        }
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

    /// Lower a call argument, boxing scalars when the formal parameter is `any` / XValue.
    fn lower_arg_for_param(
        &mut self,
        arg: &HirExpression,
        param_ty: Option<MirType>,
    ) -> MirLowerResult<MirOperand> {
        if param_ty.as_ref().is_some_and(is_xvalue) {
            return self.box_for_print(arg);
        }
        self.lower_expression(arg)
    }

    /// Parameter types for a free function (empty if unknown).
    fn params_of(&self, name: &str) -> Vec<MirType> {
        self.ctx
            .function_params
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    /// 把标量操作数装箱为 XValue
    fn box_scalar(&mut self, op: MirOperand, ty: &MirType) -> MirOperand {
        let func = match ty {
            MirType::Float(_) => "x_from_double",
            MirType::Bool => "x_from_bool",
            MirType::String => "x_from_str",
            MirType::Char => "x_from_char",
            MirType::Struct(name, _) if name == "XValue" => return op,
            // Pointer to XValue is already boxed - don't box again
            MirType::Pointer(inner) if matches!(inner.as_ref(), MirType::Struct(name, _) if name == "XValue") => return op,
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

    /// 把表达式转换为 C 字符串(char*)操作数：字符串原样；其它先装箱再转字符串。
    /// `__index__` 等已返回 XValue 时不得再经 `x_from_int`（会把句柄当整数）。
    fn as_cstr(&mut self, e: &HirExpression) -> MirLowerResult<MirOperand> {
        let ty = self.type_of(e);
        if matches!(ty, MirType::String) {
            return self.lower_expression(e);
        }
        let op = self.lower_expression(e)?;
        let op_ty = match &op {
            MirOperand::Local(id) => self
                .function
                .locals
                .get(id)
                .cloned()
                .unwrap_or_else(|| ty.clone()),
            _ => ty.clone(),
        };
        if matches!(op_ty, MirType::String) {
            return Ok(op);
        }
        let boxed = if is_xvalue(&op_ty) || is_xvalue(&ty) {
            op
        } else {
            self.box_scalar(op, &ty)
        };
        let dest = self.new_local(MirType::String);
        // XValue 字符串负载用 x_as_str；其它标签走格式化。
        let func = if is_xvalue(&op_ty) || is_xvalue(&ty) {
            "x_as_str"
        } else {
            "x_to_str"
        };
        self.current_block.instructions.push(MirInstruction::Call {
            dest: Some(dest),
            func: MirOperand::Global(func.to_string()),
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
            // arr[i] = v  →  __index__(arr, i) = v
            // dict[k] = v → x_map_put when key is a string (or non-int).
            HirExpression::Call(callee, args)
                if matches!(callee.as_ref(), HirExpression::Variable(n) if n == "__index__")
                    && args.len() == 2 =>
            {
                let coll = self.lower_expression(&args[0])?;
                let idx_ty = self.type_of(&args[1]);
                let value_ty = self.type_of(value);
                let op_ty = match &value_op {
                    MirOperand::Local(id) => self
                        .function
                        .locals
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| value_ty.clone()),
                    _ => value_ty.clone(),
                };
                let boxed = if is_xvalue(&op_ty) || is_xvalue(&value_ty) {
                    value_op.clone()
                } else {
                    self.box_scalar(value_op.clone(), &value_ty)
                };
                if matches!(idx_ty, MirType::String)
                    || !matches!(idx_ty, MirType::Int(_) | MirType::Unknown | MirType::Char)
                {
                    let key = self.box_for_print(&args[1])?;
                    self.current_block.instructions.push(MirInstruction::Call {
                        dest: None,
                        func: MirOperand::Global("x_map_put".to_string()),
                        args: vec![coll, key, boxed],
                    });
                } else {
                    let idx = self.lower_expression(&args[1])?;
                    self.current_block.instructions.push(MirInstruction::Call {
                        dest: None,
                        func: MirOperand::Global("x_list_set".to_string()),
                        args: vec![coll, idx, boxed],
                    });
                }
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
        let struct_ty = MirType::Struct(name.to_string(), Vec::new());
        let obj = self.new_local(MirType::Pointer(Box::new(struct_ty.clone())));
        self.current_block.instructions.push(MirInstruction::Alloc {
            dest: obj,
            ty: struct_ty,
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
        let struct_ty = MirType::Struct(enum_name.to_string(), Vec::new());
        let obj = self.new_local(MirType::Pointer(Box::new(struct_ty.clone())));
        self.current_block.instructions.push(MirInstruction::Alloc {
            dest: obj,
            ty: struct_ty,
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
        // 用于把泛型负载投影为真实类型。 Enums are heap pointers, so unwrap
        // Pointer(Struct(...)) as well as bare Struct.
        let type_args: Vec<MirType> = match self.type_of(discriminant) {
            MirType::Struct(name, args) if name == enum_name => args,
            MirType::Pointer(inner) => match inner.as_ref() {
                MirType::Struct(name, args) if name == enum_name => args.clone(),
                _ => Vec::new(),
            },
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
                    // Class/enum/record names are also Generic in HIR — not type params.
                    if self.ctx.classes.contains_key(g)
                        || self.ctx.enums.contains_key(g)
                        || self.ctx.records.contains_key(g)
                    {
                        return lower_user_type(t, &self.ctx);
                    }
                    if let Some(pos) = info.type_params.iter().position(|p| p == g) {
                        if let Some(ta) = type_args.get(pos) {
                            return ta.clone();
                        }
                    }
                    // 无法解析的泛型参数：保持装箱（XValue），让运行时按实际标签处理，
                    // 避免误用 x_as_ptr 把整数/布尔当指针拆箱。
                    return xvalue_ty();
                }
                lower_user_type(t, &self.ctx)
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
            MirType::Struct(name, _) if name != "XValue" => {
                ("x_as_ptr", MirType::Pointer(Box::new(pty.clone())))
            }
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
            MirType::Pointer(inner) => match *inner {
                MirType::Struct(name, _) if name != "XValue" && name != "tuple" => Some(name),
                _ => None,
            },
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
            MirType::Pointer(inner) => match *inner {
                MirType::Struct(name, _) if self.ctx.enums.contains_key(&name) => Some(name),
                _ => None,
            },
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
        // 方法体内 `this` 与 `self` 同义（对齐解释器）
        let key = if name == "this" { "self" } else { name };
        self.function
            .parameters
            .iter()
            .find(|p| p.name == key)
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
            HirLiteral::UnsignedInteger(_, _) => MirType::Int(64),
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
                AstType::Int | AstType::UnsignedInt(_) | AstType::IntSized(_) => MirType::Int(64),
                AstType::Float => MirType::Float(64),
                AstType::Bool => MirType::Bool,
                AstType::Char | AstType::CChar => MirType::Char,
                AstType::Generic(name) if ctx.classes.contains_key(name) => {
                    MirType::Pointer(Box::new(MirType::Struct(name.clone(), Vec::new())))
                }
                AstType::Generic(name) if ctx.enums.contains_key(name) => {
                    MirType::Pointer(Box::new(MirType::Struct(name.clone(), Vec::new())))
                }
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
                    return MirType::Pointer(Box::new(MirType::Struct(tyname.clone(), Vec::new())));
                }
            }
            let obj_ty = infer_expr_type(obj, ctx, locals);
            let struct_name = match &obj_ty {
                MirType::Struct(name, _) => Some(name.clone()),
                MirType::Pointer(inner) => match inner.as_ref() {
                    MirType::Struct(name, _) => Some(name.clone()),
                    _ => None,
                },
                _ => None,
            };
            if let Some(name) = struct_name {
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
                if name == "__index__" {
                    xvalue_ty()
                } else if let Some(rt) = ctx.functions.get(name) {
                    rt.clone()
                } else if ctx.classes.contains_key(name) {
                    MirType::Pointer(Box::new(MirType::Struct(name.clone(), Vec::new())))
                } else {
                    MirType::Unknown
                }
            }
            HirExpression::Member(obj, method) => {
                // 枚举构造
                if let HirExpression::Variable(tyname) = obj.as_ref() {
                    if ctx.enums.contains_key(tyname) {
                        return MirType::Pointer(Box::new(MirType::Struct(
                            tyname.clone(),
                            Vec::new(),
                        )));
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
                // UFCS：obj.method() -> prelude free function return type
                if let Some(free) = mir_stdlib_ufcs_name(&obj_ty, method) {
                    if let Some(rt) = ctx.functions.get(free) {
                        return rt.clone();
                    }
                }
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
        HirExpression::If(_, t, e) => {
            let tt = infer_expr_type(t, ctx, locals);
            if !matches!(tt, MirType::Unknown | MirType::Unit) {
                tt
            } else {
                infer_expr_type(e, ctx, locals)
            }
        }
        HirExpression::Assign(_, v) => infer_expr_type(v, ctx, locals),
        HirExpression::Block(block) => {
            for stmt in block.statements.iter().rev() {
                if let HirStatement::Expression(expr) = stmt {
                    return infer_expr_type(expr, ctx, locals);
                }
            }
            MirType::Unknown
        }
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
        HirLiteral::UnsignedInteger(v, _) => MirConstant::Int(*v as i64),
        HirLiteral::Float(v) => MirConstant::Float(*v),
        HirLiteral::Boolean(v) => MirConstant::Bool(*v),
        HirLiteral::String(v) => MirConstant::String(v.clone()),
        HirLiteral::Char(v) => MirConstant::Char(*v),
        HirLiteral::Unit => MirConstant::Unit,
        HirLiteral::None => MirConstant::Null,
    }
}

/// 字段的内存表示类型：统一为 8 字节宽度，避免后端 8 字节存取破坏紧凑布局。
fn field_repr_ty(ty: &HirType, ctx: &TypeCtx) -> MirType {
    repr_of(lower_user_type(ty, ctx))
}

/// 用户类/枚举/XValue 在运行时以堆指针表示（构造器 malloc）。
fn lower_user_type(ty: &HirType, ctx: &TypeCtx) -> MirType {
    let lowered = lower_type(ty);
    match &lowered {
        MirType::Struct(name, _)
            if ctx.classes.contains_key(name)
                || ctx.enums.contains_key(name)
                || name == "XValue" =>
        {
            MirType::Pointer(Box::new(lowered))
        }
        _ => lowered,
    }
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
        HirType::UnsignedInt(_) => MirType::Int(64),
        HirType::IntSized(_) => MirType::Int(64),
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

        HirType::TypeParam(_) | HirType::Unknown => MirType::Unknown,
        // Dynamic values use the boxed XValue representation (lists, maps, `any`).
        HirType::Dynamic => xvalue_ty(),

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
