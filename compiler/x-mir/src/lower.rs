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

use crate::mir::*;
use x_hir::{
    Hir, HirBinaryOp, HirBlock, HirDeclaration, HirExpression, HirFunctionDecl, HirLiteral,
    HirParameter, HirPattern, HirStatement, HirType, HirUnaryOp,
};

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
}

impl HirToMirLowerer {
    fn new(module_name: &str) -> Self {
        Self {
            module: MirModule {
                name: module_name.to_string(),
                functions: Vec::new(),
                globals: Vec::new(),
            },
        }
    }

    fn finish(self) -> MirModule {
        self.module
    }

    fn lower_declaration(&mut self, decl: &HirDeclaration) -> MirLowerResult<()> {
        match decl {
            HirDeclaration::Function(func) => {
                let mir_func = FunctionLowerer::lower_function(func)?;
                self.module.functions.push(mir_func);
            }
            HirDeclaration::Variable(var) => {
                let init = if let Some(expr) = &var.initializer {
                    match expr {
                        HirExpression::Literal(lit) => Some(lower_literal_to_constant(lit)),
                        _ => None,
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
                    is_extern: true,
                };
                self.module.functions.push(mir_func);
            }
            HirDeclaration::Class(_)
            | HirDeclaration::Trait(_)
            | HirDeclaration::Enum(_)
            | HirDeclaration::TypeAlias(_)
            | HirDeclaration::Module(_)
            | HirDeclaration::Import(_)
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
        let synthetic = HirFunctionDecl {
            name: "main".to_string(),
            parameters: Vec::<HirParameter>::new(),
            return_type: HirType::Int,
            body: HirBlock {
                statements: statements.to_vec(),
            },
            is_async: false,
            effects: Vec::new(),
        };

        let mut mir = FunctionLowerer::lower_function(&synthetic)?;

        // 如果 main 最终没有显式 return，则补一个 return 0
        if let Some(last) = mir.blocks.last_mut() {
            if !matches!(last.terminator, MirTerminator::Return { .. }) {
                last.terminator = MirTerminator::Return {
                    value: Some(MirOperand::Constant(MirConstant::Int(0))),
                };
            }
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
    fn lower_function(func: &HirFunctionDecl) -> MirLowerResult<MirFunction> {
        let mut lowerer = Self {
            function: MirFunction {
                name: func.name.clone(),
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
                is_extern: false,
            },
            current_block: MirBasicBlock {
                id: 0,
                instructions: Vec::new(),
                terminator: MirTerminator::Unreachable,
            },
            next_local: 0,
            scopes: vec![HashMap::new()],
        };

        // 参数映射为 Param(index)，无需放进 locals
        lowerer.lower_block(&func.body)?;

        // 若函数体没有显式 return，则根据返回类型补默认返回
        if matches!(lowerer.current_block.terminator, MirTerminator::Unreachable) {
            lowerer.current_block.terminator = MirTerminator::Return {
                value: default_return_value(&lowerer.function.return_type),
            };
        }

        lowerer.function.blocks.push(lowerer.current_block);
        Ok(lowerer.function)
    }

    fn lower_block(&mut self, block: &HirBlock) -> MirLowerResult<()> {
        self.push_scope();

        for stmt in &block.statements {
            self.lower_statement(stmt)?;
        }

        self.pop_scope();
        Ok(())
    }

    fn lower_statement(&mut self, stmt: &HirStatement) -> MirLowerResult<()> {
        match stmt {
            HirStatement::Expression(expr) => {
                let _ = self.lower_expression(expr)?;
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
            }
            HirStatement::Return(expr) => {
                let value = expr
                    .as_ref()
                    .map(|e| self.lower_expression(e))
                    .transpose()?;
                self.current_block.terminator = MirTerminator::Return { value };
            }
            HirStatement::If(if_stmt) => {
                let _ = self.lower_expression(&if_stmt.condition)?;
                self.lower_block(&if_stmt.then_block)?;
                if let Some(else_block) = &if_stmt.else_block {
                    self.lower_block(else_block)?;
                }
            }
            HirStatement::For(for_stmt) => {
                let _ = self.lower_expression(&for_stmt.iterator)?;
                self.bind_pattern(&for_stmt.pattern)?;
                self.lower_block(&for_stmt.body)?;
            }
            HirStatement::While(while_stmt) => {
                let _ = self.lower_expression(&while_stmt.condition)?;
                self.lower_block(&while_stmt.body)?;
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
            }
            HirStatement::Break | HirStatement::Continue => {
                // 当前最小 lowering 先保留线性结构，不拆 CFG。
            }
            HirStatement::Unsafe(block) => {
                self.lower_block(block)?;
            }
        }

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
                } else {
                    // 函数名/全局名暂时保守地作为字符串常量承载
                    Ok(MirOperand::Constant(MirConstant::String(name.clone())))
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
            HirExpression::Assign(target, value) => {
                let value_op = self.lower_expression(value)?;
                match target.as_ref() {
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
                    parameters: params.clone(),
                    return_type: HirType::Unknown,
                    body: body.clone(),
                    is_async: false,
                    effects: Vec::new(),
                };

                let lowered = FunctionLowerer::lower_function(&synthetic)?;
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
            HirExpression::Match(discriminant, _cases) => {
                // Lower the discriminant
                self.lower_expression(discriminant)?;
                // For now, just lower the discriminant - full pattern matching lowering to MIR is TODO
                Ok(MirOperand::Constant(MirConstant::Unit))
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
            scope.insert(name, local);
        }
    }

    fn lookup_local(&self, name: &str) -> Option<MirLocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
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

        HirType::Option(inner) => MirType::Pointer(Box::new(lower_type(inner))),
        HirType::Result(ok, _) => lower_type(ok),
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

        HirType::Pointer(inner) | HirType::ConstPointer(inner) => {
            MirType::Pointer(Box::new(lower_type(inner)))
        }

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
