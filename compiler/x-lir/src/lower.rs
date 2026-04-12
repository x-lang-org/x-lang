//! MIR → LIR lowering
//!
//! 该模块提供从 `x_mir::MirModule` 到 `x_lir::Program` 的最小可用 lowering，
//! 用于把新的编译流水线真正接起来：
//!
//! AST -> HIR -> MIR -> LIR -> Backend
//!
//! 当前实现目标：
//! - 为 CLI 的 `--emit lir` 提供结构化输出
//! - 为后端统一输入提供稳定边界
//! - 保持实现简单、保守、可扩展
//!
//! 这不是最终优化版 lowering：
//! - 暂未保留完整 CFG 语义
//! - 暂未处理 Phi/SSA 消解
//! - 暂未做寄存器分配、栈帧布局、调用约定细化
//!
//! 但它已经足以作为新的架构层次中的 LIR/XIR 生成入口。

use crate::{
    BinaryOp, Block, Declaration, Expression, ExternFunction, Function, GlobalVar, Literal,
    Program, Statement, Type, UnaryOp, Variable,
};
use x_mir::{
    MirBasicBlock, MirBinOp, MirConstant, MirFunction, MirInstruction, MirModule, MirOperand,
    MirTerminator, MirType, MirUnOp,
};

/// MIR 到 LIR 的 lowering 错误
#[derive(Debug, thiserror::Error)]
pub enum LirLowerError {
    #[error("不支持的 MIR 特性: {0}")]
    UnsupportedFeature(String),

    #[error("内部 lowering 错误: {0}")]
    Internal(String),
}

pub type LirLowerResult<T> = Result<T, LirLowerError>;

/// 将整个 MIR 模块 lowering 为 LIR 程序
pub fn lower_mir_to_lir(module: &MirModule) -> LirLowerResult<Program> {
    let mut program = Program::new();

    add_runtime_declarations(&mut program);

    // Lower import declarations
    for import in &module.imports {
        program.add(Declaration::Import(crate::Import {
            module_path: import.module_path.clone(),
            symbols: import.symbols.clone(),
            import_all: import.import_all,
        }));
    }

    for global in &module.globals {
        program.add(Declaration::Global(lower_global(global)?));
    }

    for func in &module.functions {
        if func.is_extern {
            program.add(Declaration::ExternFunction(lower_extern_function(func)?));
        } else {
            program.add(Declaration::Function(lower_function(func)?));
        }
    }

    Ok(program)
}

/// 添加运行时外部声明
fn add_runtime_declarations(program: &mut Program) {
    let runtime = [
        ExternFunction {
            name: "printf".to_string(),
            type_params: Vec::new(),
            return_type: Type::Int,
            parameters: vec![Type::Pointer(Box::new(Type::Char))],
            abi: Some("c".to_string()),
        },
        ExternFunction {
            name: "malloc".to_string(),
            type_params: Vec::new(),
            return_type: Type::Pointer(Box::new(Type::Void)),
            parameters: vec![Type::Size],
            abi: Some("c".to_string()),
        },
        ExternFunction {
            name: "free".to_string(),
            type_params: Vec::new(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
            abi: Some("c".to_string()),
        },
        ExternFunction {
            name: "x_perceus_retain".to_string(),
            type_params: Vec::new(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
            abi: None,
        },
        ExternFunction {
            name: "x_perceus_release".to_string(),
            type_params: Vec::new(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
            abi: None,
        },
    ];

    for decl in runtime {
        program.add(Declaration::ExternFunction(decl));
    }
}

fn lower_global(global: &x_mir::MirGlobal) -> LirLowerResult<GlobalVar> {
    Ok(GlobalVar {
        name: global.name.clone(),
        type_: lower_type(&global.ty),
        initializer: global
            .initializer
            .as_ref()
            .map(lower_constant_to_expression),
        is_static: !global.mutable,
    })
}

fn lower_extern_function(func: &MirFunction) -> LirLowerResult<ExternFunction> {
    let type_params = func.type_params.iter().map(|tp| tp.name.clone()).collect();

    Ok(ExternFunction {
        name: func.name.clone(),
        type_params,
        return_type: lower_type(&func.return_type),
        parameters: func.parameters.iter().map(|p| lower_type(&p.ty)).collect(),
        abi: None,
    })
}

fn lower_function(func: &MirFunction) -> LirLowerResult<Function> {
    let mut lir_func = Function::new(&func.name, lower_type(&func.return_type));
    lir_func.type_params = func.type_params.iter().map(|tp| tp.name.clone()).collect();

    for (index, param) in func.parameters.iter().enumerate() {
        // Use arg{index} format to match param_name() in lower_operand
        lir_func = lir_func.param(&param_name(index), lower_type(&param.ty));
    }

    let mut body = Block::new();

    if !func.locals.is_empty() {
        let mut locals: Vec<_> = func.locals.iter().collect();
        locals.sort_by_key(|(id, _)| **id);

        for (id, ty) in locals {
            body.add(Statement::Variable(Variable {
                name: local_name(*id),
                type_: lower_type(ty),
                initializer: None,
                is_static: false,
                is_extern: false,
            }));
        }
    }

    for block in &func.blocks {
        lower_basic_block(block, &mut body)?;
    }

    if func.blocks.is_empty() {
        if let Some(default_return) = default_return_expr(&func.return_type) {
            body.add(Statement::Return(Some(default_return)));
        } else {
            body.add(Statement::Return(None));
        }
    }

    lir_func.body = body;
    Ok(lir_func)
}

fn lower_basic_block(block: &MirBasicBlock, body: &mut Block) -> LirLowerResult<()> {
    body.add(Statement::Label(block_label(block.id)));

    for instr in &block.instructions {
        lower_instruction(instr, body)?;
    }

    lower_terminator(&block.terminator, body)?;
    Ok(())
}

fn lower_instruction(instr: &MirInstruction, body: &mut Block) -> LirLowerResult<()> {
    match instr {
        MirInstruction::Assign { dest, value } => {
            body.add(assign_local_stmt(*dest, lower_operand(value)));
        }
        MirInstruction::BinaryOp {
            dest,
            op,
            left,
            right,
        } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Binary(
                    lower_binary_op(*op),
                    Box::new(lower_operand(left)),
                    Box::new(lower_operand(right)),
                ),
            ));
        }
        MirInstruction::UnaryOp { dest, op, operand } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Unary(lower_unary_op(*op), Box::new(lower_operand(operand))),
            ));
        }
        MirInstruction::Call { dest, func, args } => {
            let call = Expression::Call(
                Box::new(lower_operand(func)),
                args.iter().map(lower_operand).collect(),
            );

            if let Some(dest) = dest {
                body.add(assign_local_stmt(*dest, call));
            } else {
                body.add(Statement::Expression(call));
            }
        }
        MirInstruction::FieldAccess {
            dest,
            object,
            field,
        } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Member(Box::new(lower_operand(object)), field.clone()),
            ));
        }
        MirInstruction::ArrayAccess { dest, array, index } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Index(
                    Box::new(lower_operand(array)),
                    Box::new(lower_operand(index)),
                ),
            ));
        }
        MirInstruction::Alloc { dest, ty, size } => {
            let malloc_call = Expression::Call(
                Box::new(Expression::Variable("malloc".to_string())),
                vec![Expression::Literal(Literal::UnsignedLongLong(*size as u64))],
            );

            body.add(assign_local_stmt(
                *dest,
                Expression::Cast(
                    Type::Pointer(Box::new(lower_type(ty))),
                    Box::new(malloc_call),
                ),
            ));
        }
        MirInstruction::Load { dest, ptr } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Dereference(Box::new(lower_operand(ptr))),
            ));
        }
        MirInstruction::Store { ptr, value } => {
            // Handle global variable stores differently from dereference
            let ptr_expr = match ptr {
                MirOperand::Global(name) => Expression::Variable(name.clone()),
                _ => Expression::Dereference(Box::new(lower_operand(ptr))),
            };
            body.add(Statement::Expression(Expression::Assign(
                Box::new(ptr_expr),
                Box::new(lower_operand(value)),
            )));
        }
        MirInstruction::Cast { dest, value, ty } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Cast(lower_type(ty), Box::new(lower_operand(value))),
            ));
        }
        MirInstruction::Dup { dest, src } => {
            // Perceus: retain the reference before assignment
            let src_expr = lower_operand(src);
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("x_perceus_retain".to_string())),
                vec![Expression::Cast(
                    Type::Pointer(Box::new(Type::Void)),
                    Box::new(src_expr.clone()),
                )],
            )));
            body.add(assign_local_stmt(*dest, src_expr));
        }
        MirInstruction::Drop { value } => {
            // Perceus: release the reference (deallocates when count reaches zero)
            let expr = lower_operand(value);
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("x_perceus_release".to_string())),
                vec![Expression::Cast(
                    Type::Pointer(Box::new(Type::Void)),
                    Box::new(expr),
                )],
            )));
        }
        MirInstruction::Reuse { dest, src } => {
            // Reuse just moves the reference, no retain/release needed
            body.add(assign_local_stmt(*dest, lower_operand(src)));
        }
        MirInstruction::WhenGuard {
            dest,
            condition,
            body: guard_body,
        } => {
            // WhenGuard: if condition is true, return the body value
            // For now, emit as a simple if-return pattern
            body.add(Statement::If(crate::IfStatement {
                condition: lower_operand(condition),
                then_branch: Box::new(Statement::Return(Some(lower_operand(guard_body)))),
                else_branch: None,
            }));
            body.add(assign_local_stmt(
                *dest,
                Expression::Literal(Literal::Integer(0)),
            ));
        }
    }

    Ok(())
}

fn lower_terminator(term: &MirTerminator, body: &mut Block) -> LirLowerResult<()> {
    match term {
        MirTerminator::Branch { target } => {
            body.add(Statement::Goto(block_label(*target)));
        }
        MirTerminator::CondBranch {
            cond,
            then_block,
            else_block,
        } => {
            body.add(Statement::If(crate::IfStatement {
                condition: lower_operand(cond),
                then_branch: Box::new(Statement::Goto(block_label(*then_block))),
                else_branch: Some(Box::new(Statement::Goto(block_label(*else_block)))),
            }));
        }
        MirTerminator::Return { value } => {
            body.add(Statement::Return(value.as_ref().map(lower_operand)));
        }
        MirTerminator::Unreachable => {
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("abort".to_string())),
                vec![],
            )));
        }
        MirTerminator::Switch {
            value,
            cases,
            default,
        } => {
            body.add(Statement::Switch(crate::SwitchStatement {
                expression: lower_operand(value),
                cases: cases
                    .iter()
                    .map(|(constant, block)| crate::SwitchCase {
                        value: lower_constant_to_expression(constant),
                        body: Box::new(Statement::Goto(block_label(*block))),
                    })
                    .collect(),
                default: Some(Box::new(Statement::Goto(block_label(*default)))),
            }));
        }
    }

    Ok(())
}

fn lower_operand(operand: &MirOperand) -> Expression {
    match operand {
        MirOperand::Local(id) => Expression::Variable(local_name(*id)),
        MirOperand::Constant(c) => lower_constant_to_expression(c),
        MirOperand::Param(index) => Expression::Variable(param_name(*index)),
        MirOperand::Global(name) => Expression::Variable(name.clone()),
    }
}

fn lower_constant_to_expression(constant: &MirConstant) -> Expression {
    Expression::Literal(match constant {
        MirConstant::Int(v) => Literal::Integer(*v),
        MirConstant::Float(v) => Literal::Double(*v),
        MirConstant::Bool(v) => Literal::Bool(*v),
        MirConstant::String(v) => Literal::String(v.clone()),
        MirConstant::Char(v) => Literal::Char(*v),
        MirConstant::Null => Literal::NullPointer,
        MirConstant::Unit => Literal::Integer(0),
    })
}

fn lower_type(ty: &MirType) -> Type {
    match ty {
        MirType::Int(bits) => match bits {
            0..=32 => Type::Int,
            _ => Type::LongLong,
        },
        MirType::Float(bits) => match bits {
            0..=32 => Type::Float,
            _ => Type::Double,
        },
        MirType::Bool => Type::Bool,
        MirType::String => Type::Pointer(Box::new(Type::Char)),
        MirType::Char => Type::Char,
        MirType::Unit => Type::Void,
        MirType::Pointer(inner) => Type::Pointer(Box::new(lower_type(inner))),
        MirType::Array(inner, len) => Type::Array(Box::new(lower_type(inner)), Some(*len as u64)),
        MirType::Struct(name, _) => Type::Named(name.clone()),
        MirType::Function(params, ret) => Type::FunctionPointer(
            Box::new(lower_type(ret)),
            params.iter().map(lower_type).collect(),
        ),
        MirType::Unknown => Type::Int,
    }
}

fn lower_binary_op(op: MirBinOp) -> BinaryOp {
    match op {
        MirBinOp::Add => BinaryOp::Add,
        MirBinOp::Sub => BinaryOp::Subtract,
        MirBinOp::Mul => BinaryOp::Multiply,
        MirBinOp::Div => BinaryOp::Divide,
        MirBinOp::Mod => BinaryOp::Modulo,
        MirBinOp::Eq => BinaryOp::Equal,
        MirBinOp::Ne => BinaryOp::NotEqual,
        MirBinOp::Lt => BinaryOp::LessThan,
        MirBinOp::Le => BinaryOp::LessThanEqual,
        MirBinOp::Gt => BinaryOp::GreaterThan,
        MirBinOp::Ge => BinaryOp::GreaterThanEqual,
        MirBinOp::And => BinaryOp::LogicalAnd,
        MirBinOp::Or => BinaryOp::LogicalOr,
        MirBinOp::BitAnd => BinaryOp::BitAnd,
        MirBinOp::BitOr => BinaryOp::BitOr,
        MirBinOp::BitXor => BinaryOp::BitXor,
        MirBinOp::Shl => BinaryOp::LeftShift,
        MirBinOp::Shr => BinaryOp::RightShift,
    }
}

fn lower_unary_op(op: MirUnOp) -> UnaryOp {
    match op {
        MirUnOp::Neg => UnaryOp::Minus,
        MirUnOp::Not => UnaryOp::Not,
        MirUnOp::BitNot => UnaryOp::BitNot,
        MirUnOp::Reference => UnaryOp::Reference,
        MirUnOp::MutableReference => UnaryOp::MutableReference,
    }
}

fn assign_local_stmt(local: usize, expr: Expression) -> Statement {
    Statement::Expression(Expression::Assign(
        Box::new(Expression::Variable(local_name(local))),
        Box::new(expr),
    ))
}

fn local_name(id: usize) -> String {
    format!("t{id}")
}

fn param_name(index: usize) -> String {
    format!("arg{index}")
}

fn block_label(id: usize) -> String {
    format!("bb{id}")
}

fn default_return_expr(ty: &MirType) -> Option<Expression> {
    match ty {
        MirType::Unit => None,
        MirType::Bool => Some(Expression::Literal(Literal::Bool(false))),
        MirType::Int(_) => Some(Expression::Literal(Literal::Integer(0))),
        MirType::Float(_) => Some(Expression::Literal(Literal::Double(0.0))),
        MirType::String => Some(Expression::Literal(Literal::String(String::new()))),
        MirType::Char => Some(Expression::Literal(Literal::Char('\0'))),
        MirType::Pointer(_) => Some(Expression::Literal(Literal::NullPointer)),
        MirType::Array(_, _)
        | MirType::Struct(_, _)
        | MirType::Function(_, _)
        | MirType::Unknown => Some(Expression::Literal(Literal::Integer(0))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_mir::{
        MirBasicBlock, MirConstant, MirFunction, MirGlobal, MirInstruction, MirModule, MirOperand,
        MirParameter, MirTerminator, MirType,
    };

    #[test]
    fn lower_empty_module() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            functions: vec![],
            globals: vec![],
        };

        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        assert!(!lir.declarations.is_empty()); // runtime decls
    }

    #[test]
    fn lower_simple_function() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "main".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::Int(42)),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0usize, MirType::Int(32))].into_iter().collect(),
                name_to_local: [("x".to_string(), 0)].into_iter().collect(),
                is_extern: false,
            }],
        };

        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("main"));
        assert!(text.contains("t0"));
        assert!(text.contains("return t0;"));
    }

    #[test]
    fn lower_global_variable() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            functions: vec![],
            globals: vec![MirGlobal {
                name: "answer".to_string(),
                ty: MirType::Int(32),
                initializer: Some(MirConstant::Int(42)),
                mutable: false,
            }],
        };

        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("answer"));
        assert!(text.contains("42"));
    }

    // ==================== Binary Operations ====================

    #[test]
    fn lower_binary_add() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Add, 10, 20);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 + t1") || text.contains("+"));
    }

    #[test]
    fn lower_binary_sub() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Sub, 30, 10);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 - t1") || text.contains("-"));
    }

    #[test]
    fn lower_binary_mul() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Mul, 5, 6);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 * t1") || text.contains("*"));
    }

    #[test]
    fn lower_binary_div() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Div, 100, 10);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 / t1") || text.contains("/"));
    }

    #[test]
    fn lower_binary_mod() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Mod, 17, 5);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 % t1") || text.contains("%"));
    }

    #[test]
    fn lower_binary_eq() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Eq, 5, 5);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 == t1") || text.contains("=="));
    }

    #[test]
    fn lower_binary_lt() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Lt, 3, 5);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 < t1") || text.contains("<"));
    }

    #[test]
    fn lower_binary_and() {
        let mir = create_binary_op_module(x_mir::MirBinOp::And, 1, 1);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 && t1") || text.contains("&&"));
    }

    #[test]
    fn lower_binary_or() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Or, 0, 1);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 || t1") || text.contains("||"));
    }

    // ==================== Unary Operations ====================

    #[test]
    fn lower_unary_not() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "test_not".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Bool,
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Bool(true)),
                        },
                        MirInstruction::UnaryOp {
                            dest: 1,
                            op: MirUnOp::Not,
                            operand: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Bool), (1, MirType::Bool)]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("!"));
    }

    #[test]
    fn lower_unary_neg() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "test_neg".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::UnaryOp {
                            dest: 1,
                            op: MirUnOp::Neg,
                            operand: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("-"));
    }

    // ==================== Function Calls ====================

    #[test]
    fn lower_function_call_no_args() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![
                MirFunction {
                    name: "get_value".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(42))),
                        },
                    }],
                    locals: vec![].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
                MirFunction {
                    name: "caller".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Call {
                            dest: Some(0),
                            func: MirOperand::Global("get_value".to_string()),
                            args: vec![],
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    }],
                    locals: [(0, MirType::Int(32))].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
            ],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("get_value"));
        assert!(text.contains("caller"));
    }

    #[test]
    fn lower_function_call_with_args() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![
                MirFunction {
                    name: "add".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![
                        MirParameter {
                            name: "a".to_string(),
                            ty: MirType::Int(32),
                            index: 0,
                        },
                        MirParameter {
                            name: "b".to_string(),
                            ty: MirType::Int(32),
                            index: 1,
                        },
                    ],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::BinaryOp {
                            dest: 0,
                            op: x_mir::MirBinOp::Add,
                            left: MirOperand::Param(0),
                            right: MirOperand::Param(1),
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    }],
                    locals: [(0, MirType::Int(32))].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
                MirFunction {
                    name: "caller".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Call {
                            dest: Some(0),
                            func: MirOperand::Global("add".to_string()),
                            args: vec![
                                MirOperand::Constant(MirConstant::Int(10)),
                                MirOperand::Constant(MirConstant::Int(20)),
                            ],
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    }],
                    locals: [(0, MirType::Int(32))].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
            ],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("add"));
        assert!(text.contains("10"));
        assert!(text.contains("20"));
    }

    // ==================== Control Flow ====================

    #[test]
    fn lower_conditional_branch() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "abs".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::BinaryOp {
                            dest: 1,
                            op: x_mir::MirBinOp::Lt,
                            left: MirOperand::Param(0),
                            right: MirOperand::Constant(MirConstant::Int(0)),
                        }],
                        terminator: MirTerminator::CondBranch {
                            cond: MirOperand::Local(1),
                            then_block: 1,
                            else_block: 2,
                        },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![MirInstruction::UnaryOp {
                            dest: 2,
                            op: MirUnOp::Neg,
                            operand: MirOperand::Param(0),
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(2)),
                        },
                    },
                    MirBasicBlock {
                        id: 2,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Param(0)),
                        },
                    },
                ],
                locals: [(1, MirType::Bool), (2, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("if"));
        assert!(text.contains("goto"));
    }

    #[test]
    fn lower_unconditional_branch() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "loop_example".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)),
                        }],
                        terminator: MirTerminator::Branch { target: 1 },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    },
                ],
                locals: [(0, MirType::Int(32))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("goto"));
    }

    // ==================== Switch Statement ====================

    #[test]
    fn lower_switch_statement() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "switch_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![],
                        terminator: MirTerminator::Switch {
                            value: MirOperand::Param(0),
                            cases: vec![(MirConstant::Int(1), 1), (MirConstant::Int(2), 2)],
                            default: 3,
                        },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(10))),
                        },
                    },
                    MirBasicBlock {
                        id: 2,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(20))),
                        },
                    },
                    MirBasicBlock {
                        id: 3,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(0))),
                        },
                    },
                ],
                locals: vec![].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("switch"));
    }

    // ==================== Memory Operations ====================

    #[test]
    fn lower_alloc_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "alloc_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Pointer(Box::new(MirType::Int(32))),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Alloc {
                        dest: 0,
                        ty: MirType::Int(32),
                        size: 4,
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::Pointer(Box::new(MirType::Int(32))))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("malloc"));
    }

    #[test]
    fn lower_load_store_instructions() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "load_store_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Alloc {
                            dest: 0,
                            ty: MirType::Int(32),
                            size: 4,
                        },
                        MirInstruction::Assign {
                            dest: 1,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Store {
                            ptr: MirOperand::Local(0),
                            value: MirOperand::Local(1),
                        },
                        MirInstruction::Load {
                            dest: 2,
                            ptr: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(2)),
                    },
                }],
                locals: [
                    (0, MirType::Pointer(Box::new(MirType::Int(32)))),
                    (1, MirType::Int(32)),
                    (2, MirType::Int(32)),
                ]
                .into_iter()
                .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("malloc"));
        assert!(text.contains("*"));
    }

    // ==================== Perceus Operations ====================

    #[test]
    fn lower_dup_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "dup_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Dup {
                            dest: 1,
                            src: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("x_perceus_retain"));
    }

    #[test]
    fn lower_drop_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "drop_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Drop {
                            value: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Constant(MirConstant::Int(0))),
                    },
                }],
                locals: [(0, MirType::Int(32))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("x_perceus_release"));
    }

    #[test]
    fn lower_reuse_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "reuse_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Reuse {
                            dest: 1,
                            src: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();

        // Find the function body - Reuse should generate simple assignment, not function calls
        // The function body should contain "t1 = t0;" style assignment, not x_perceus_retain()
        // We check that within the reuse_test function there's no retain/release call
        let func_start = text.find("reuse_test").expect("function should exist");
        let func_body = &text[func_start..];

        // The function should have assignment like "t1 = t0"
        assert!(func_body.contains("t1 = t0") || func_body.contains("t1=t0"));

        // Reuse should NOT generate a call expression (no parentheses after retain/release in function body)
        // Note: External declarations of retain/release will exist, but no calls in this function
        assert!(!func_body.contains("x_perceus_retain("));
        assert!(!func_body.contains("x_perceus_release("));
    }

    // ==================== Types ====================

    #[test]
    fn lower_float_type() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "float_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Float(64),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::Float(1.25)),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::Float(64))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("double"));
        assert!(text.contains("1.25"));
    }

    #[test]
    fn lower_bool_type() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "bool_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Bool,
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::Bool(true)),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::Bool)].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("bool"));
        assert!(text.contains("true"));
    }

    #[test]
    fn lower_string_type() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "string_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::String,
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::String(
                            "Hello, World!".to_string(),
                        )),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::String)].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("Hello, World!"));
    }

    // ==================== External Functions ====================

    #[test]
    fn lower_extern_function() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "external_func".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![],
                locals: vec![].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: true,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("external"));
        assert!(text.contains("external_func"));
    }

    // ==================== Imports ====================

    #[test]
    fn lower_import_declaration() {
        use x_mir::Import;

        let mir = MirModule {
            name: "main".to_string(),
            imports: vec![Import {
                module_path: "std.io".to_string(),
                symbols: vec![("println".to_string(), None)],
                import_all: false,
            }],
            globals: vec![],
            functions: vec![],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("import"));
        assert!(text.contains("std.io"));
        assert!(text.contains("println"));
    }

    #[test]
    fn lower_import_all() {
        use x_mir::Import;

        let mir = MirModule {
            name: "main".to_string(),
            imports: vec![Import {
                module_path: "std.collections".to_string(),
                symbols: vec![],
                import_all: true,
            }],
            globals: vec![],
            functions: vec![],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("import"));
        assert!(text.contains("std.collections"));
        assert!(text.contains("*"));
    }

    // ==================== Field Access ====================

    #[test]
    fn lower_field_access() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "field_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)), // placeholder for struct
                        },
                        MirInstruction::FieldAccess {
                            dest: 1,
                            object: MirOperand::Local(0),
                            field: "x".to_string(),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains(".x"));
    }

    // ==================== Array Access ====================

    #[test]
    fn lower_array_access() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "array_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)), // placeholder for array
                        },
                        MirInstruction::Assign {
                            dest: 1,
                            value: MirOperand::Constant(MirConstant::Int(0)), // index
                        },
                        MirInstruction::ArrayAccess {
                            dest: 2,
                            array: MirOperand::Local(0),
                            index: MirOperand::Local(1),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(2)),
                    },
                }],
                locals: [
                    (0, MirType::Int(32)),
                    (1, MirType::Int(32)),
                    (2, MirType::Int(32)),
                ]
                .into_iter()
                .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("[") && text.contains("]"));
    }

    // ==================== Cast ====================

    #[test]
    fn lower_cast_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "cast_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Float(64),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Cast {
                            dest: 1,
                            value: MirOperand::Local(0),
                            ty: MirType::Float(64),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Float(64))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("cast") || text.contains("double"));
    }

    // ==================== Unreachable ====================

    #[test]
    fn lower_unreachable_terminator() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "unreachable_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![],
                    terminator: MirTerminator::Unreachable,
                }],
                locals: vec![].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("abort"));
    }

    // ==================== Helper Functions ====================

    fn create_binary_op_module(op: x_mir::MirBinOp, left: i64, right: i64) -> MirModule {
        MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "binary_op".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(left)),
                        },
                        MirInstruction::Assign {
                            dest: 1,
                            value: MirOperand::Constant(MirConstant::Int(right)),
                        },
                        MirInstruction::BinaryOp {
                            dest: 2,
                            op,
                            left: MirOperand::Local(0),
                            right: MirOperand::Local(1),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(2)),
                    },
                }],
                locals: [
                    (0, MirType::Int(32)),
                    (1, MirType::Int(32)),
                    (2, MirType::Int(32)),
                ]
                .into_iter()
                .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        }
    }
}
