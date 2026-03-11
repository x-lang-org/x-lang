//! AST → X IR lowering
//! 将 X 语言的 AST 转换为 X IR

use crate::xir::*;
use x_parser::ast;
use x_parser::ast::{BinaryOp as AstBinaryOp, UnaryOp as AstUnaryOp};

/// Lowering 错误
#[derive(Debug, thiserror::Error)]
pub enum LowerError {
    #[error("不支持的特性: {0}")]
    UnsupportedFeature(String),
    #[error("类型错误: {0}")]
    TypeError(String),
    #[error("未定义的符号: {0}")]
    UndefinedSymbol(String),
}

pub type LowerResult<T> = Result<T, LowerError>;

/// 将 AST 程序 lowering 为 X IR 程序
pub fn lower_program(ast_program: &ast::Program) -> LowerResult<Program> {
    let mut program = Program::new();

    // 添加标准库的外部函数声明（如 printf）
    add_stdlib_declarations(&mut program);

    // 处理每个声明
    for decl in &ast_program.declarations {
        let xir_decl = lower_declaration(decl)?;
        program.add(xir_decl);
    }

    Ok(program)
}

/// 添加标准库外部函数声明
fn add_stdlib_declarations(program: &mut Program) {
    // printf: int printf(const char*, ...)
    program.add(Declaration::ExternFunction(ExternFunction {
        name: "printf".to_string(),
        return_type: Type::Int,
        parameters: vec![Type::Pointer(Box::new(Type::Char))],
    }));

    // malloc: void* malloc(size_t)
    program.add(Declaration::ExternFunction(ExternFunction {
        name: "malloc".to_string(),
        return_type: Type::Pointer(Box::new(Type::Void)),
        parameters: vec![Type::Size],
    }));

    // free: void free(void*)
    program.add(Declaration::ExternFunction(ExternFunction {
        name: "free".to_string(),
        return_type: Type::Void,
        parameters: vec![Type::Pointer(Box::new(Type::Void))],
    }));
}

/// lowering 单个声明
fn lower_declaration(decl: &ast::Declaration) -> LowerResult<Declaration> {
    match decl {
        ast::Declaration::Function(func_decl) => {
            Ok(Declaration::Function(lower_function(func_decl)?))
        }
        ast::Declaration::Variable(var_decl) => {
            Ok(Declaration::Global(lower_global_var(var_decl)?))
        }
        ast::Declaration::TypeAlias(alias) => Ok(Declaration::TypeAlias(TypeAlias {
            name: alias.name.clone(),
            type_: lower_type(&alias.type_)?,
        })),
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的声明类型: {:?}",
            decl
        ))),
    }
}

/// lowering 函数声明
fn lower_function(func_decl: &ast::FunctionDecl) -> LowerResult<Function> {
    let mut func = Function::new(
        &func_decl.name,
        func_decl
            .return_type
            .as_ref()
            .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int)),
    );

    // 处理参数
    for param in &func_decl.parameters {
        let param_type = param
            .type_annot
            .as_ref()
            .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
        func = func.param(&param.name, param_type);
    }

    // 处理函数体
    func.body = lower_block(&func_decl.body)?;

    Ok(func)
}

/// lowering 全局变量
fn lower_global_var(var_decl: &ast::VariableDecl) -> LowerResult<GlobalVar> {
    let type_ = var_decl
        .type_annot
        .as_ref()
        .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
    let initializer = var_decl
        .initializer
        .as_ref()
        .map(|e| lower_expression(e))
        .transpose()?;

    Ok(GlobalVar {
        name: var_decl.name.clone(),
        type_,
        initializer,
        is_static: false,
    })
}

/// lowering 类型
fn lower_type(ty: &ast::Type) -> LowerResult<Type> {
    match ty {
        ast::Type::Int => Ok(Type::Int),
        ast::Type::Float => Ok(Type::Double),
        ast::Type::Bool => Ok(Type::Bool),
        ast::Type::String => Ok(Type::Pointer(Box::new(Type::Char))),
        ast::Type::Char => Ok(Type::Char),
        ast::Type::Unit => Ok(Type::Void),
        ast::Type::Array(inner) => Ok(Type::Array(Box::new(lower_type(inner)?), None)),
        ast::Type::Option(inner) => {
            // Option<T> 暂时 lowering 为 T*，用 NULL 表示 None
            Ok(Type::Pointer(Box::new(lower_type(inner)?)))
        }
        ast::Type::Result(ok, _err) => {
            // Result<T, E> 暂时 lowering 为 T
            lower_type(ok)
        }
        ast::Type::Generic(name) | ast::Type::Var(name) => Ok(Type::Named(name.clone())),
        ast::Type::Function(params, ret) => {
            let param_types = params
                .iter()
                .map(|p| lower_type(p))
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Type::FunctionPointer(
                Box::new(lower_type(ret)?),
                param_types,
            ))
        }
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的类型: {:?}",
            ty
        ))),
    }
}

/// lowering 语句块
fn lower_block(block: &ast::Block) -> LowerResult<Block> {
    let mut xir_block = Block::new();

    for stmt in &block.statements {
        xir_block.add(lower_statement(stmt)?);
    }

    Ok(xir_block)
}

/// lowering 单个语句
fn lower_statement(stmt: &ast::Statement) -> LowerResult<Statement> {
    match stmt {
        ast::Statement::Expression(expr) => Ok(Statement::Expression(lower_expression(expr)?)),
        ast::Statement::Variable(var_decl) => Ok(Statement::Variable(lower_local_var(var_decl)?)),
        ast::Statement::Return(expr_opt) => Ok(Statement::Return(
            expr_opt.as_ref().map(|e| lower_expression(e)).transpose()?,
        )),
        ast::Statement::If(if_stmt) => {
            let then_branch = Box::new(Statement::Compound(lower_block(&if_stmt.then_block)?));
            let else_branch = if_stmt
                .else_block
                .as_ref()
                .map(|b| Ok::<_, LowerError>(Box::new(Statement::Compound(lower_block(b)?))))
                .transpose()?;

            Ok(Statement::If(IfStatement {
                condition: lower_expression(&if_stmt.condition)?,
                then_branch,
                else_branch,
            }))
        }
        ast::Statement::While(while_stmt) => Ok(Statement::While(WhileStatement {
            condition: lower_expression(&while_stmt.condition)?,
            body: Box::new(Statement::Compound(lower_block(&while_stmt.body)?)),
        })),
        ast::Statement::Break => Ok(Statement::Break),
        ast::Statement::Continue => Ok(Statement::Continue),
        ast::Statement::DoWhile(d) => Ok(Statement::DoWhile(DoWhileStatement {
            body: Box::new(Statement::Compound(lower_block(&d.body)?)),
            condition: lower_expression(&d.condition)?,
        })),
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的语句: {:?}",
            stmt
        ))),
    }
}

/// lowering 局部变量声明
fn lower_local_var(var_decl: &ast::VariableDecl) -> LowerResult<Variable> {
    let type_ = var_decl
        .type_annot
        .as_ref()
        .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
    let initializer = var_decl
        .initializer
        .as_ref()
        .map(|e| lower_expression(e))
        .transpose()?;

    Ok(Variable {
        name: var_decl.name.clone(),
        type_,
        initializer,
        is_static: false,
        is_extern: false,
    })
}

/// lowering 表达式
fn lower_expression(expr: &ast::Expression) -> LowerResult<Expression> {
    match expr {
        ast::Expression::Literal(lit) => lower_literal(lit),
        ast::Expression::Variable(name) => {
            // 特殊处理 print 函数，映射到 printf
            if name == "print" {
                Ok(Expression::Variable("printf".to_string()))
            } else {
                Ok(Expression::Variable(name.clone()))
            }
        }
        ast::Expression::Binary(op, left, right) => Ok(Expression::Binary(
            lower_binary_op(op)?,
            Box::new(lower_expression(left)?),
            Box::new(lower_expression(right)?),
        )),
        ast::Expression::Unary(op, expr) => Ok(Expression::Unary(
            lower_unary_op(op)?,
            Box::new(lower_expression(expr)?),
        )),
        ast::Expression::Call(callee, args) => {
            let xir_callee = Box::new(lower_expression(callee)?);
            let mut xir_args = Vec::with_capacity(args.len());

            for arg in args {
                xir_args.push(lower_expression(arg)?);
            }

            // 如果调用的是 print，需要特殊处理参数
            if let Expression::Variable(name) = xir_callee.as_ref() {
                if name == "printf" && !args.is_empty() {
                    // 如果第一个参数是字符串字面量，保持原样
                    // 如果不是，需要添加格式字符串
                    if let Some(first_arg) = args.first() {
                        if !matches!(first_arg, ast::Expression::Literal(ast::Literal::String(_))) {
                            // 对于非字符串参数，自动添加 "%d" 或类似格式
                            return handle_print_call(xir_args);
                        }
                    }
                }
            }

            Ok(Expression::Call(xir_callee, xir_args))
        }
        ast::Expression::Assign(target, value) => Ok(Expression::Assign(
            Box::new(lower_expression(target)?),
            Box::new(lower_expression(value)?),
        )),
        ast::Expression::Parenthesized(inner) => Ok(Expression::Parenthesized(Box::new(
            lower_expression(inner)?,
        ))),
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的表达式: {:?}",
            expr
        ))),
    }
}

/// 处理 print 函数调用，自动添加格式字符串
fn handle_print_call(mut args: Vec<Expression>) -> LowerResult<Expression> {
    if args.len() == 1 {
        // 单个参数，根据类型选择格式
        let fmt_str = Expression::Literal(Literal::String("%d\\n".to_string()));
        Ok(Expression::Call(
            Box::new(Expression::Variable("printf".to_string())),
            vec![fmt_str, args.remove(0)],
        ))
    } else {
        // 多个参数，先打印第一个（应该是字符串）
        Ok(Expression::Call(
            Box::new(Expression::Variable("printf".to_string())),
            args,
        ))
    }
}

/// lowering 字面量
fn lower_literal(lit: &ast::Literal) -> LowerResult<Expression> {
    match lit {
        ast::Literal::Integer(n) => Ok(Expression::Literal(Literal::Integer(*n))),
        ast::Literal::Float(f) => Ok(Expression::Literal(Literal::Double(*f))),
        ast::Literal::Boolean(b) => Ok(Expression::Literal(Literal::Bool(*b))),
        ast::Literal::String(s) => Ok(Expression::Literal(Literal::String(s.clone()))),
        ast::Literal::Char(c) => Ok(Expression::Literal(Literal::Char(*c))),
        ast::Literal::Null => Ok(Expression::Literal(Literal::NullPointer)),
        ast::Literal::None => Ok(Expression::Literal(Literal::NullPointer)),
        ast::Literal::Unit => Ok(Expression::Literal(Literal::Integer(0))),
    }
}

/// lowering 二元运算符
fn lower_binary_op(op: &AstBinaryOp) -> LowerResult<BinaryOp> {
    match op {
        AstBinaryOp::Add => Ok(BinaryOp::Add),
        AstBinaryOp::Sub => Ok(BinaryOp::Subtract),
        AstBinaryOp::Mul => Ok(BinaryOp::Multiply),
        AstBinaryOp::Div => Ok(BinaryOp::Divide),
        AstBinaryOp::Mod => Ok(BinaryOp::Modulo),
        AstBinaryOp::And => Ok(BinaryOp::LogicalAnd),
        AstBinaryOp::Or => Ok(BinaryOp::LogicalOr),
        AstBinaryOp::Equal => Ok(BinaryOp::Equal),
        AstBinaryOp::NotEqual => Ok(BinaryOp::NotEqual),
        AstBinaryOp::Less => Ok(BinaryOp::LessThan),
        AstBinaryOp::LessEqual => Ok(BinaryOp::LessThanEqual),
        AstBinaryOp::Greater => Ok(BinaryOp::GreaterThan),
        AstBinaryOp::GreaterEqual => Ok(BinaryOp::GreaterThanEqual),
        AstBinaryOp::BitAnd => Ok(BinaryOp::BitAnd),
        AstBinaryOp::BitOr => Ok(BinaryOp::BitOr),
        AstBinaryOp::BitXor => Ok(BinaryOp::BitXor),
        AstBinaryOp::LeftShift => Ok(BinaryOp::LeftShift),
        AstBinaryOp::RightShift => Ok(BinaryOp::RightShift),
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的二元运算符: {:?}",
            op
        ))),
    }
}

/// lowering 一元运算符
fn lower_unary_op(op: &AstUnaryOp) -> LowerResult<UnaryOp> {
    match op {
        AstUnaryOp::Negate => Ok(UnaryOp::Minus),
        AstUnaryOp::Not => Ok(UnaryOp::Not),
        AstUnaryOp::BitNot => Ok(UnaryOp::BitNot),
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的一元运算符: {:?}",
            op
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_simple_function() {
        // 创建一个简单的 AST 函数并 lowering
        let ast_func = ast::FunctionDecl {
            name: "main".to_string(),
            parameters: vec![],
            return_type: Some(ast::Type::Int),
            body: ast::Block { statements: vec![] },
            is_async: false,
        };

        let result = lower_function(&ast_func);
        assert!(result.is_ok());

        let func = result.unwrap();
        assert_eq!(func.name, "main");
        assert_eq!(func.return_type, Type::Int);
    }
}
