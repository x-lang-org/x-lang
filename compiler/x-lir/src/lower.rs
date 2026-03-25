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
            return_type: Type::Int,
            parameters: vec![Type::Pointer(Box::new(Type::Char))],
        },
        ExternFunction {
            name: "malloc".to_string(),
            return_type: Type::Pointer(Box::new(Type::Void)),
            parameters: vec![Type::Size],
        },
        ExternFunction {
            name: "free".to_string(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
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
    Ok(ExternFunction {
        name: func.name.clone(),
        return_type: lower_type(&func.return_type),
        parameters: func.parameters.iter().map(|p| lower_type(&p.ty)).collect(),
    })
}

fn lower_function(func: &MirFunction) -> LirLowerResult<Function> {
    let mut lir_func = Function::new(&func.name, lower_type(&func.return_type));

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
            body.add(Statement::Expression(Expression::Assign(
                Box::new(Expression::Dereference(Box::new(lower_operand(ptr)))),
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
            body.add(assign_local_stmt(*dest, lower_operand(src)));
        }
        MirInstruction::Drop { value } => {
            let expr = lower_operand(value);
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("free".to_string())),
                vec![Expression::Cast(
                    Type::Pointer(Box::new(Type::Void)),
                    Box::new(expr),
                )],
            )));
        }
        MirInstruction::Reuse { dest, src } => {
            body.add(assign_local_stmt(*dest, lower_operand(src)));
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
            globals: vec![],
            functions: vec![MirFunction {
                name: "main".to_string(),
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
}
