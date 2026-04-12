//! HIR → MIR lowering
//!
//! 该模块提供从 `x_hir::Hir` 到 `x_mir::MirModule` 的最小可用 lowering。
//! 当前目标是把新的编译架构真正接起来：
//!
//! AST -> HIR -> MIR -> LIR -> Backend
//!
//! 这里的 lowering 偏“结构保真”而非“优化正确性极致完整”：
//! - 为每个函数生成一个线性的入口基本块
//! - 顶层语句会被收集进合成的 `main` 函数
//! - 控制流结构暂时以线性指令 + 保守 terminator 表示
//! - 尚未做 SSA、CFG 拆块、Phi、复杂模式展开等中端优化
//!
//! 这足够支撑：
//! - CLI 的 `--emit mir`
//! - 后续 `MIR -> LIR` 降级
//! - 新架构下的流水线联通

use std::collections::HashMap;
use x_hir::HirImportSymbol;

use crate::mir::*;
use x_hir::{
    Hir, HirBinaryOp, HirBlock, HirDeclaration, HirExpression, HirFunctionDecl, HirLiteral,
    HirParameter, HirPattern, HirStatement, HirType, HirUnaryOp,
};
use x_parser::ast::{Literal, Pattern};

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

/// 将整个 HIR 程序 lowering 为 MIR 模块
pub fn lower_hir_to_mir(hir: &Hir) -> MirLowerResult<MirModule> {
    let mut lowerer = HirToMirLowerer::new(&hir.module_name);

    // 顶层声明
    for decl in &hir.declarations {
        lowerer.lower_declaration(decl)?;
    }

    // 顶层语句 -> 合成 main
    if !hir.statements.is_empty() {
        lowerer.lower_toplevel_statements_as_main(&hir.statements)?;
    }

    Ok(lowerer.finish())
}

/// 内部 lowering 状态
struct HirToMirLowerer {
    module: MirModule,
    /// 需要在 main 中初始化的全局变量（非字面量初始化器）
    globals_needing_init: Vec<(String, HirType, HirExpression)>,
}

impl HirToMirLowerer {
    fn new(module_name: &str) -> Self {
        Self {
            module: MirModule {
                name: module_name.to_string(),
                imports: Vec::new(),
                functions: Vec::new(),
                globals: Vec::new(),
            },
            globals_needing_init: Vec::new(),
        }
    }

    fn finish(self) -> MirModule {
        self.module
    }

    fn lower_declaration(&mut self, decl: &HirDeclaration) -> MirLowerResult<()> {
        match decl {
            HirDeclaration::Function(func) => {
                let mir_func = FunctionLowerer::lower_function(func, None)?;
                self.module.functions.push(mir_func);
            }
            HirDeclaration::Variable(var) => {
                let init = if let Some(expr) = &var.initializer {
                    match expr {
                        HirExpression::Literal(lit) => Some(lower_literal_to_constant(lit)),
                        _ => {
                            // 非字面量初始化器需要在 main 中赋值
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
                    ty: lower_type(&var.ty),
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
                // 将导入声明添加到 MIR 模块中供后续阶段使用
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
            HirDeclaration::Class(_)
            | HirDeclaration::Trait(_)
            | HirDeclaration::Enum(_)
            | HirDeclaration::Record(_)
            | HirDeclaration::Effect(_)
            | HirDeclaration::Implement
            | HirDeclaration::TypeAlias(_)
            | HirDeclaration::Newtype(_)
            | HirDeclaration::Module(_)
            | HirDeclaration::Export(_) => {
                // 当前阶段先不在 MIR 中显式建模这些高级声明。
                // 它们要么属于类型层信息，要么会在后续 lowering 中展开。
            }
        }

        Ok(())
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

        // 合并初始化语句和用户语句
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

        // Collect global variable names for the main function
        let global_names: Vec<String> =
            self.module.globals.iter().map(|g| g.name.clone()).collect();
        let mut mir = FunctionLowerer::lower_function(&synthetic, Some(&global_names))?;

        // 对于 main 函数，总是使用 return 0
        // 这样可以避免 println/print 等函数返回值被当作程序退出码
        if let Some(last) = mir.blocks.last_mut() {
            last.terminator = MirTerminator::Return {
                value: Some(MirOperand::Constant(MirConstant::Int(0))),
            };
        }

        self.module.functions.push(mir);
        Ok(())
    }
}

struct FunctionLowerer {
    function: MirFunction,
    current_block: MirBasicBlock,
    next_local: MirLocalId,
    scopes: Vec<HashMap<String, MirLocalId>>,
}

impl FunctionLowerer {
    fn lower_function(
        func: &HirFunctionDecl,
        global_names: Option<&[String]>,
    ) -> MirLowerResult<MirFunction> {
        let type_params = func
            .type_params
            .iter()
            .map(|name| TypeParameter { name: name.clone() })
            .collect();

        // Create initial scope with global variable names (special marker)
        let mut initial_scope = HashMap::new();
        if let Some(globals) = global_names {
            for name in globals {
                // Use special local ID -1 to indicate global variable
                initial_scope.insert(name.clone(), MirLocalId::MAX);
            }
        }

        let mut lowerer = Self {
            function: MirFunction {
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
            },
            current_block: MirBasicBlock {
                id: 0,
                instructions: Vec::new(),
                terminator: MirTerminator::Unreachable,
            },
            next_local: 0,
            scopes: vec![initial_scope],
        };

        // 参数映射为 Param(index)，无需放进 locals
        let last_expr_value = lowerer.lower_block(&func.body)?;

        // 若函数体没有显式 return，则根据返回类型补默认返回
        if matches!(lowerer.current_block.terminator, MirTerminator::Unreachable) {
            // 优先使用最后一条表达式语句的值作为返回值
            let return_value =
                last_expr_value.or_else(|| default_return_value(&lowerer.function.return_type));
            lowerer.current_block.terminator = MirTerminator::Return {
                value: return_value,
            };
        }

        lowerer.function.blocks.push(lowerer.current_block);
        Ok(lowerer.function)
    }

    /// 降低语句块，返回最后一条表达式语句的操作数（如果有）
    fn lower_block(&mut self, block: &HirBlock) -> MirLowerResult<Option<MirOperand>> {
        self.push_scope();

        let mut last_expr_value: Option<MirOperand> = None;

        for stmt in &block.statements {
            if let Some(value) = self.lower_statement(stmt)? {
                last_expr_value = Some(value);
            }
        }

        self.pop_scope();
        Ok(last_expr_value)
    }

    /// 降低语句，返回表达式的操作数（如果有）
    fn lower_statement(&mut self, stmt: &HirStatement) -> MirLowerResult<Option<MirOperand>> {
        match stmt {
            HirStatement::Expression(expr) => {
                // 返回表达式的值，供隐式 return 使用
                let value = self.lower_expression(expr)?;
                Ok(Some(value))
            }
            HirStatement::Variable(var) => {
                let local = self.new_local(lower_type(&var.ty));
                self.bind_local(var.name.clone(), local);

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
                let _ = self.lower_expression(&if_stmt.condition)?;
                self.lower_block(&if_stmt.then_block)?;
                if let Some(else_block) = &if_stmt.else_block {
                    self.lower_block(else_block)?;
                }
                Ok(None)
            }
            HirStatement::For(for_stmt) => {
                let _ = self.lower_expression(&for_stmt.iterator)?;
                self.bind_pattern(&for_stmt.pattern)?;
                self.lower_block(&for_stmt.body)?;
                Ok(None)
            }
            HirStatement::While(while_stmt) => {
                let _ = self.lower_expression(&while_stmt.condition)?;
                self.lower_block(&while_stmt.body)?;
                Ok(None)
            }
            HirStatement::Match(match_stmt) => {
                let _ = self.lower_expression(&match_stmt.expression)?;
                for case in &match_stmt.cases {
                    self.bind_pattern(&case.pattern)?;
                    if let Some(guard) = &case.guard {
                        let _ = self.lower_expression(guard)?;
                    }
                    self.lower_block(&case.body)?;
                }
                Ok(None)
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
            HirStatement::Break | HirStatement::Continue => {
                // 当前最小 lowering 先保留线性结构，不拆 CFG。
                Ok(None)
            }
            HirStatement::Unsafe(block) => self.lower_block(block),
            HirStatement::Defer(expr) => {
                // Defer 表达式在函数退出时执行，MIR 阶段暂不处理执行顺序
                let _ = self.lower_expression(expr)?;
                Ok(None)
            }
            HirStatement::Yield(_expr) => {
                // 生成器暂不支持
                Ok(None)
            }
            HirStatement::Loop(body) => {
                // 无限循环
                self.lower_block(body)?;
                Ok(None)
            }
            HirStatement::WhenGuard(condition, body) => {
                let cond = self.lower_expression(condition)?;
                let body_val = self.lower_expression(body)?;
                let dest = self.new_local(MirType::Unknown);
                self.current_block
                    .instructions
                    .push(MirInstruction::WhenGuard {
                        dest,
                        condition: cond,
                        body: body_val,
                    });
                Ok(Some(MirOperand::Local(dest)))
            }
        }
    }

    fn lower_expression(&mut self, expr: &HirExpression) -> MirLowerResult<MirOperand> {
        match expr {
            HirExpression::Literal(lit) => Ok(MirOperand::Constant(lower_literal_to_constant(lit))),
            HirExpression::Variable(name) => {
                if let Some(local) = self.lookup_local(name) {
                    Ok(MirOperand::Local(local))
                } else if let Some(param_index) = self.lookup_param(name) {
                    Ok(MirOperand::Param(param_index))
                } else {
                    // 函数名/全局名作为全局引用
                    Ok(MirOperand::Global(name.clone()))
                }
            }
            HirExpression::Member(object, field) => {
                let object_op = self.lower_expression(object)?;
                let dest = self.new_local(MirType::Unknown);
                self.current_block
                    .instructions
                    .push(MirInstruction::FieldAccess {
                        dest,
                        object: object_op,
                        field: field.clone(),
                    });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Call(callee, args) => {
                let func = self.lower_expression(callee)?;
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expression(arg))
                    .collect::<MirLowerResult<Vec<_>>>()?;

                let dest = self.new_local(MirType::Unknown);
                self.current_block.instructions.push(MirInstruction::Call {
                    dest: Some(dest),
                    func,
                    args: lowered_args,
                });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Binary(op, lhs, rhs) => {
                let lhs = self.lower_expression(lhs)?;
                let rhs = self.lower_expression(rhs)?;
                let dest = self.new_local(MirType::Unknown);

                self.current_block
                    .instructions
                    .push(MirInstruction::BinaryOp {
                        dest,
                        op: lower_binary_op(op)?,
                        left: lhs,
                        right: rhs,
                    });

                Ok(MirOperand::Local(dest))
            }
            HirExpression::Unary(op, expr) => {
                let operand = self.lower_expression(expr)?;
                let dest = self.new_local(MirType::Unknown);

                self.current_block
                    .instructions
                    .push(MirInstruction::UnaryOp {
                        dest,
                        op: lower_unary_op(op)?,
                        operand,
                    });

                Ok(MirOperand::Local(dest))
            }
            HirExpression::Cast(expr, _ty) => {
                // Cast just returns the same operand, type is already tracked
                let operand = self.lower_expression(expr)?;
                Ok(operand)
            }
            HirExpression::Assign(target, value) => {
                let value_op = self.lower_expression(value)?;
                // Extract the inner expression if it's wrapped in Typed
                let target_inner = match target.as_ref() {
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
                            self.current_block
                                .instructions
                                .push(MirInstruction::Assign {
                                    dest,
                                    value: value_op.clone(),
                                });
                            Ok(MirOperand::Local(dest))
                        } else if self.is_global(name) {
                            // Handle as global variable - emit a Store instruction
                            self.current_block.instructions.push(MirInstruction::Store {
                                ptr: MirOperand::Global(name.clone()),
                                value: value_op.clone(),
                            });
                            Ok(MirOperand::Global(name.clone()))
                        } else {
                            Err(MirLowerError::UndefinedVariable(name.clone()))
                        }
                    }
                    _ => {
                        // 更复杂的 lvalue 当前先保守求值，不显式建模存储目标
                        let _ = self.lower_expression(target)?;
                        Ok(value_op)
                    }
                }
            }
            HirExpression::If(cond, then_expr, else_expr) => {
                let _ = self.lower_expression(cond)?;
                let _ = self.lower_expression(then_expr)?;
                self.lower_expression(else_expr)
            }
            HirExpression::Lambda(params, body) => {
                let synthetic_name = format!("_lambda_{}", self.function.name);
                let synthetic = HirFunctionDecl {
                    name: synthetic_name.clone(),
                    type_params: Vec::new(),
                    parameters: params.clone(),
                    return_type: HirType::Unknown,
                    body: body.clone(),
                    is_async: false,
                    effects: Vec::new(),
                };

                let lowered = FunctionLowerer::lower_function(&synthetic, None)?;
                // 将 lambda 直接附加到模块级别做不到，因为这里只看到函数级上下文。
                // 所以这里退化成函数名常量，供后续阶段识别。
                let _ = lowered;
                Ok(MirOperand::Constant(MirConstant::String(synthetic_name)))
            }
            HirExpression::Array(items) => {
                let dest = self.new_local(MirType::Array(Box::new(MirType::Unknown), items.len()));
                self.current_block.instructions.push(MirInstruction::Alloc {
                    dest,
                    ty: MirType::Array(Box::new(MirType::Unknown), items.len()),
                    size: items.len(),
                });
                for item in items {
                    let _ = self.lower_expression(item)?;
                }
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Tuple(items) => {
                // 元组在 MIR 中表示为数组
                let dest = self.new_local(MirType::Array(Box::new(MirType::Unknown), items.len()));
                self.current_block.instructions.push(MirInstruction::Alloc {
                    dest,
                    ty: MirType::Array(Box::new(MirType::Unknown), items.len()),
                    size: items.len(),
                });
                for item in items {
                    let _ = self.lower_expression(item)?;
                }
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Dictionary(entries) => {
                let dest = self.new_local(MirType::Unknown);
                self.current_block.instructions.push(MirInstruction::Alloc {
                    dest,
                    ty: MirType::Unknown,
                    size: entries.len(),
                });
                for (k, v) in entries {
                    let _ = self.lower_expression(k)?;
                    let _ = self.lower_expression(v)?;
                }
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Record(_, fields) => {
                let dest = self.new_local(MirType::Unknown);
                self.current_block.instructions.push(MirInstruction::Alloc {
                    dest,
                    ty: MirType::Unknown,
                    size: fields.len(),
                });
                for (_, value) in fields {
                    let _ = self.lower_expression(value)?;
                }
                Ok(MirOperand::Local(dest))
            }
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
                // Lower discriminant to a local variable
                let discr_local = match self.lower_expression(discriminant)? {
                    MirOperand::Local(id) => {
                        // Already in a local, reuse it
                        id
                    }
                    operand => {
                        // Create new local and assign
                        let local_id = self.new_local(MirType::Bool);
                        self.current_block
                            .instructions
                            .push(MirInstruction::Assign {
                                dest: local_id,
                                value: operand,
                            });
                        local_id
                    }
                };

                // Create a block for each case and a merge block at the end
                let start_block_id = self.current_block.id;
                let mut case_blocks = Vec::new();
                let mut next_block_id = self.function.blocks.len() + 1;

                // For each case: create a new basic block, pattern matching and jump to merge
                for (_pattern, guard, body) in cases {
                    let case_block_id = next_block_id;
                    next_block_id += 1;

                    // Save current block - we'll come back after adding the case block
                    let prev_current_block = std::mem::replace(
                        &mut self.current_block,
                        MirBasicBlock {
                            id: case_block_id,
                            instructions: Vec::new(),
                            terminator: MirTerminator::Unreachable,
                        },
                    );

                    // If we have a guard, add the guard check and conditional jump
                    if let Some(guard_expr) = guard {
                        let _guard_val = self.lower_expression(guard_expr)?;
                        // TODO: full guard handling - for now just proceed
                    }

                    // Lower all statements in the case body
                    for stmt in &body.statements {
                        self.lower_statement(stmt)?;
                    }

                    // Add the case block to the function
                    self.function
                        .blocks
                        .push(std::mem::take(&mut self.current_block));
                    case_blocks.push(case_block_id);
                    self.current_block = prev_current_block;
                }

                // Current block (entry) after discriminant becomes the dispatch block
                // We emit a conditional branch for each case
                // TODO: for simple boolean match (if desugaring), this is two cases, can be handled as if-else
                // For now, generate sequential comparison and branching
                let discr_local_ref = discr_local;

                let _current_id = start_block_id;

                // For each case, compare and branch
                for (idx, (pattern, _, _)) in cases.iter().enumerate() {
                    let case_block_id = case_blocks[idx];

                    match pattern {
                        // Simple literal pattern (boolean for if desugaring)
                        Pattern::Literal(Literal::Boolean(expected)) => {
                            // Compare discriminant == expected
                            let cmp_result = self.new_local(MirType::Bool);
                            self.current_block
                                .instructions
                                .push(MirInstruction::BinaryOp {
                                    op: MirBinOp::Eq,
                                    left: MirOperand::Local(discr_local_ref),
                                    right: MirOperand::Constant(MirConstant::Bool(*expected)),
                                    dest: cmp_result,
                                });

                            // Branch to case block if equal
                            self.current_block.terminator = MirTerminator::CondBranch {
                                cond: MirOperand::Local(cmp_result),
                                then_block: case_block_id,
                                else_block: next_block_id,
                            };

                            self.function
                                .blocks
                                .push(std::mem::take(&mut self.current_block));
                            self.current_block = MirBasicBlock {
                                id: next_block_id,
                                instructions: Vec::new(),
                                terminator: MirTerminator::Unreachable,
                            };
                            next_block_id += 1;
                        }
                        _ => {
                            // For complex patterns, fall back - not implemented yet
                            // Continue to next case
                        }
                    }
                }

                // The last block is unreachable (should have covered all cases)
                self.current_block.terminator = MirTerminator::Unreachable;
                self.function
                    .blocks
                    .push(std::mem::take(&mut self.current_block));

                // Return the result from the merge - the last block actually holds the result
                // For now, create a result local and return it
                let result_local = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(result_local))
            }
            HirExpression::Await(expr) => {
                // Await 异步等待，返回内部表达式结果类型
                self.lower_expression(expr)
            }
            HirExpression::OptionalChain(base, _member) => {
                // 可选链暂简化
                let _ = self.lower_expression(base)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            HirExpression::NullCoalescing(left, right) => {
                // 空合并：计算两个操作数，返回结果
                let _ = self.lower_expression(left)?;
                let _ = self.lower_expression(right)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            HirExpression::WhenGuard(condition, body) => {
                let cond = self.lower_expression(condition)?;
                let body_val = self.lower_expression(body)?;
                let dest = self.new_local(MirType::Unknown);
                self.current_block
                    .instructions
                    .push(MirInstruction::WhenGuard {
                        dest,
                        condition: cond,
                        body: body_val,
                    });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Block(block) => {
                self.lower_block(block)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
        }
    }

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
        // Also store in function-level mapping for Perceus to use
        self.function.name_to_local.insert(name, local);
    }

    fn lookup_local(&self, name: &str) -> Option<MirLocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .filter(|&id| id != MirLocalId::MAX) // MirLocalId::MAX indicates global variable
    }

    fn is_global(&self, name: &str) -> bool {
        let result = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .map(|id| id == MirLocalId::MAX);
        result.unwrap_or(false)
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

fn lower_type(ty: &HirType) -> MirType {
    match ty {
        HirType::Int => MirType::Int(32),
        HirType::UnsignedInt => MirType::Int(32),
        HirType::Float => MirType::Float(64),
        HirType::Bool => MirType::Bool,
        HirType::String | HirType::CString => MirType::String,
        HirType::Char | HirType::CChar => MirType::Char,
        HirType::Unit | HirType::Void | HirType::Never => MirType::Unit,

        HirType::Array(inner) => MirType::Array(Box::new(lower_type(inner)), 0),
        HirType::Dictionary(_, _) => MirType::Pointer(Box::new(MirType::Unknown)),
        HirType::Record(name, _) => MirType::Struct(name.clone(), Vec::new()),
        HirType::Union(name, _) => MirType::Struct(name.clone(), Vec::new()),
        HirType::Tuple(types) => {
            MirType::Struct("tuple".to_string(), types.iter().map(lower_type).collect())
        }

        // Option and Result are now TypeConstructor - handled by the TypeConstructor case below
        HirType::Function(params, ret) => MirType::Function(
            params.iter().map(lower_type).collect(),
            Box::new(lower_type(ret)),
        ),
        HirType::Async(inner) => lower_type(inner),

        HirType::Generic(_)
        | HirType::TypeParam(_)
        | HirType::TypeConstructor(_, _)
        | HirType::Dynamic
        | HirType::Unknown => MirType::Unknown,

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
            return Err(MirLowerError::UnsupportedFeature(
                "字符串拼接尚未专门 lowering 到 MIR".to_string(),
            ))
        }
        HirBinaryOp::Pow => {
            return Err(MirLowerError::UnsupportedFeature(
                "Pow 尚未 lowering 到 MIR".to_string(),
            ))
        }
    })
}

fn lower_unary_op(op: &HirUnaryOp) -> MirLowerResult<MirUnOp> {
    Ok(match op {
        HirUnaryOp::Negate => MirUnOp::Neg,
        HirUnaryOp::Not => MirUnOp::Not,
        HirUnaryOp::BitNot => MirUnOp::BitNot,
        HirUnaryOp::Await => {
            return Err(MirLowerError::UnsupportedFeature(
                "Await 尚未 lowering 到专用 MIR 指令".to_string(),
            ))
        }
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

    #[test]
    fn lower_empty_hir_to_empty_module() {
        let hir = Hir {
            module_name: "main".to_string(),
            declarations: vec![],
            statements: vec![],
            type_env: x_hir::HirTypeEnv {
                variables: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
            },
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
            type_env: x_hir::HirTypeEnv {
                variables: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
            },
            perceus_info: x_hir::HirPerceusInfo::default(),
        };

        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        assert_eq!(mir.functions.len(), 1);
        assert_eq!(mir.functions[0].name, "main");
    }
}
