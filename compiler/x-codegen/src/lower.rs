//! AST/HIR → X IR lowering
//! 将 X 语言的 AST 或 HIR 转换为 X IR

use crate::xir::*;
use x_parser::ast;
use x_parser::ast::{BinaryOp as AstBinaryOp, ExpressionKind, StatementKind, UnaryOp as AstUnaryOp};
use x_hir::{self, Hir, HirDeclaration, HirStatement, HirExpression, HirType, HirVariableDecl, HirFunctionDecl, HirBlock, HirPattern};

// Re-export Pattern for use in this module
use crate::xir::Pattern;

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

    // 添加运行时外部函数声明（如 printf）
    add_runtime_declarations(&mut program);

    // 收集所有类声明用于继承解析
    let class_map: std::collections::HashMap<String, &ast::ClassDecl> = ast_program
        .declarations
        .iter()
        .filter_map(|decl| {
            if let ast::Declaration::Class(class_decl) = decl {
                Some((class_decl.name.clone(), class_decl))
            } else {
                None
            }
        })
        .collect();

    // 处理每个声明
    for decl in &ast_program.declarations {
        match decl {
            ast::Declaration::Class(class_decl) => {
                // 生成类定义（带继承扁平化）
                let (class_def, vtable_opt) = lower_class_with_inheritance(class_decl, &class_map)?;
                program.add(Declaration::Class(class_def));

                // 如果有虚表，生成虚表
                if let Some(vtable) = vtable_opt {
                    program.add(Declaration::VTable(vtable));
                }

                // 生成方法函数
                for member in &class_decl.members {
                    if let ast::ClassMember::Method(method) = member {
                        let method_func = lower_class_method(&class_decl.name, method)?;
                        program.add(Declaration::Function(method_func));
                    }
                    if let ast::ClassMember::Constructor(constructor) = member {
                        let constructor_func =
                            lower_constructor(&class_decl.name, constructor)?;
                        program.add(Declaration::Function(constructor_func));
                    }
                }
            }
            _ => {
                if let Ok(xir_decl) = lower_declaration(decl) {
                    program.add(xir_decl);
                }
            }
        }
    }

    Ok(program)
}

/// 将类方法 lowering 为函数
fn lower_class_method(class_name: &str, method: &ast::FunctionDecl) -> LowerResult<Function> {
    let return_type = method
        .return_type
        .as_ref()
        .map_or(Type::Void, |t| lower_type(t).unwrap_or(Type::Void));

    let mut func = Function::new(
        &format!("{}_{}", class_name, method.name),
        return_type,
    );

    // 添加 self 参数
    func = func.param("self", Type::Named(class_name.to_string()));

    // 添加方法参数
    for param in &method.parameters {
        let param_type = param
            .type_annot
            .as_ref()
            .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
        func = func.param(&param.name, param_type);
    }

    // 处理方法体
    func.body = lower_block(&method.body)?;

    Ok(func)
}

/// 将构造函数 lowering 为工厂函数
fn lower_constructor(class_name: &str, constructor: &ast::ConstructorDecl) -> LowerResult<Function> {
    let mut func = Function::new(
        &format!("{}_new", class_name),
        Type::Named(class_name.to_string()),
    );

    // 添加构造函数参数
    for param in &constructor.parameters {
        let param_type = param
            .type_annot
            .as_ref()
            .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
        func = func.param(&param.name, param_type);
    }

    // 处理构造函数体
    func.body = lower_block(&constructor.body)?;

    Ok(func)
}

/// 添加运行时外部函数声明
/// X 语言没有标准库，这些是编译器运行时依赖的 C 函数
fn add_runtime_declarations(program: &mut Program) {
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
        ast::Declaration::Class(class_decl) => {
            // 将类 lowering 为结构体
            Ok(Declaration::Struct(lower_class_to_struct(class_decl)?))
        }
        ast::Declaration::Trait(_) => {
            // trait 不生成代码，只是类型检查用的接口定义
            Err(LowerError::UnsupportedFeature(
                "trait 声明不生成独立代码".to_string(),
            ))
        }
        _ => Err(LowerError::UnsupportedFeature(format!(
            "暂不支持的声明类型: {:?}",
            decl
        ))),
    }
}

/// 将类声明 lowering 为结构体定义
fn lower_class_to_struct(class_decl: &ast::ClassDecl) -> LowerResult<Struct> {
    let mut fields = Vec::new();

    for member in &class_decl.members {
        match member {
            ast::ClassMember::Field(field) => {
                let field_type = field
                    .type_annot
                    .as_ref()
                    .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
                fields.push(Field {
                    name: field.name.clone(),
                    type_: field_type,
                });
            }
            ast::ClassMember::Method(_) | ast::ClassMember::Constructor(_) => {
                // 方法和构造函数单独生成函数，不属于结构体字段
            }
        }
    }

    Ok(Struct {
        name: class_decl.name.clone(),
        fields,
    })
}

/// 将类声明 lowering 为 Class（带继承扁平化）
fn lower_class_with_inheritance(
    class_decl: &ast::ClassDecl,
    class_map: &std::collections::HashMap<String, &ast::ClassDecl>,
) -> LowerResult<(Class, Option<VTable>)> {
    let mut all_fields = Vec::new();
    let mut virtual_methods = Vec::new();
    let mut vtable_indices = Vec::new();

    // 收集父类字段（递归）
    if let Some(parent_name) = &class_decl.extends {
        collect_parent_fields(parent_name, class_map, &mut all_fields, &mut virtual_methods)?;
    }

    // 收集当前类的字段
    for member in &class_decl.members {
        match member {
            ast::ClassMember::Field(field) => {
                let field_type = field
                    .type_annot
                    .as_ref()
                    .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
                all_fields.push(Field {
                    name: field.name.clone(),
                    type_: field_type,
                });
            }
            ast::ClassMember::Method(method) => {
                // 检查是否是虚方法
                if method.modifiers.is_virtual {
                    let idx = virtual_methods.len();
                    virtual_methods.push((method.name.clone(), method.clone()));
                    vtable_indices.push((method.name.clone(), idx));
                }
            }
            ast::ClassMember::Constructor(_) => {}
        }
    }

    // 判断是否有虚方法
    let has_vtable = !virtual_methods.is_empty();

    // 生成虚表（如果有虚方法）
    let vtable_opt = if has_vtable {
        let entries = virtual_methods
            .iter()
            .map(|(name, method)| {
                let return_type = method
                    .return_type
                    .as_ref()
                    .map_or(Type::Void, |t| lower_type(t).unwrap_or(Type::Void));

                let mut param_types = vec![Type::Named(class_decl.name.clone())];
                for param in &method.parameters {
                    let pt = param
                        .type_annot
                        .as_ref()
                        .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
                    param_types.push(pt);
                }

                VTableEntry {
                    method_name: name.clone(),
                    function_type: VTableMethodType {
                        return_type,
                        param_types,
                    },
                }
            })
            .collect();

        Some(VTable {
            name: format!("{}_VTable", class_decl.name),
            class_name: class_decl.name.clone(),
            entries,
        })
    } else {
        None
    };

    Ok((
        Class {
            name: class_decl.name.clone(),
            extends: class_decl.extends.clone(),
            implements: class_decl.implements.clone(),
            fields: all_fields,
            vtable_indices,
            has_vtable,
        },
        vtable_opt,
    ))
}

/// 递归收集父类字段
fn collect_parent_fields(
    class_name: &str,
    class_map: &std::collections::HashMap<String, &ast::ClassDecl>,
    fields: &mut Vec<Field>,
    virtual_methods: &mut Vec<(String, ast::FunctionDecl)>,
) -> LowerResult<()> {
    if let Some(parent_class) = class_map.get(class_name) {
        // 先递归收集祖父类字段
        if let Some(grandparent_name) = &parent_class.extends {
            collect_parent_fields(grandparent_name, class_map, fields, virtual_methods)?;
        }

        // 收集父类字段
        for member in &parent_class.members {
            match member {
                ast::ClassMember::Field(field) => {
                    let field_type = field
                        .type_annot
                        .as_ref()
                        .map_or(Type::Int, |t| lower_type(t).unwrap_or(Type::Int));
                    fields.push(Field {
                        name: field.name.clone(),
                        type_: field_type,
                    });
                }
                ast::ClassMember::Method(method) => {
                    // 收集父类的虚方法
                    if method.modifiers.is_virtual {
                        virtual_methods.push((method.name.clone(), method.clone()));
                    }
                }
                ast::ClassMember::Constructor(_) => {}
            }
        }
    }
    Ok(())
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
    match &stmt.node {
        StatementKind::Expression(expr) => Ok(Statement::Expression(lower_expression(expr)?)),
        StatementKind::Variable(var_decl) => Ok(Statement::Variable(lower_local_var(var_decl)?)),
        StatementKind::Return(expr_opt) => Ok(Statement::Return(
            expr_opt.as_ref().map(|e| lower_expression(e)).transpose()?,
        )),
        StatementKind::If(if_stmt) => {
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
        StatementKind::While(while_stmt) => Ok(Statement::While(WhileStatement {
            condition: lower_expression(&while_stmt.condition)?,
            body: Box::new(Statement::Compound(lower_block(&while_stmt.body)?)),
        })),
        StatementKind::For(for_stmt) => {
            // For 循环 lowering：转换为迭代器循环或传统的索引循环
            let iterator = lower_expression(&for_stmt.iterator)?;
            let body = lower_block(&for_stmt.body)?;

            // 根据模式创建循环变量
            let loop_var = match &for_stmt.pattern {
                ast::Pattern::Variable(name) => name.clone(),
                ast::Pattern::Wildcard => "_".to_string(),
                _ => "_item".to_string(),
            };

            // 创建迭代器风格的 for 循环
            // 转换为 C 风格：使用索引迭代
            // for (int i = 0; i < len; i++) { ... }
            let index_var = format!("_{}_idx", loop_var);
            let len_var = format!("_{}_len", loop_var);

            // 创建初始化语句
            let init = Statement::Variable(Variable {
                name: index_var.clone(),
                type_: Type::Int,
                initializer: Some(Expression::Literal(Literal::Integer(0))),
                is_static: false,
                is_extern: false,
            });

            // 创建条件表达式
            let condition = Expression::Binary(
                BinaryOp::LessThan,
                Box::new(Expression::Variable(index_var.clone())),
                Box::new(Expression::Variable(len_var.clone())),
            );

            // 创建增量表达式
            let increment = Expression::Unary(
                UnaryOp::PreIncrement,
                Box::new(Expression::Variable(index_var.clone())),
            );

            // 创建循环体
            let mut for_body = Block::new();
            for_body.add(Statement::Variable(Variable {
                name: loop_var,
                type_: Type::Int, // 简化：假设元素类型为 Int
                initializer: Some(Expression::Index(
                    Box::new(iterator),
                    Box::new(Expression::Variable(index_var.clone())),
                )),
                is_static: false,
                is_extern: false,
            }));
            for stmt in body.statements {
                for_body.add(stmt);
            }

            Ok(Statement::For(ForStatement {
                initializer: Some(Box::new(init)),
                condition: Some(condition),
                increment: Some(increment),
                body: Box::new(Statement::Compound(for_body)),
            }))
        }
        StatementKind::Match(match_stmt) => {
            let scrutinee = lower_expression(&match_stmt.expression)?;
            let mut cases = Vec::new();

            for case in &match_stmt.cases {
                let pattern = lower_pattern(&case.pattern)?;
                let body = lower_block(&case.body)?;
                let guard = case.guard.as_ref()
                    .map(|g| lower_expression(g))
                    .transpose()?;
                cases.push(MatchCase {
                    pattern,
                    body,
                    guard,
                });
            }

            Ok(Statement::Match(MatchStatement {
                scrutinee,
                cases,
            }))
        }
        StatementKind::Try(try_stmt) => {
            let body = lower_block(&try_stmt.body)?;
            let mut catch_clauses = Vec::new();

            for cc in &try_stmt.catch_clauses {
                catch_clauses.push(CatchClause {
                    exception_type: cc.exception_type.clone(),
                    variable_name: cc.variable_name.clone(),
                    body: lower_block(&cc.body)?,
                });
            }

            let finally_block = try_stmt.finally_block.as_ref()
                .map(|b| lower_block(b))
                .transpose()?;

            Ok(Statement::Try(TryStatement {
                body,
                catch_clauses,
                finally_block,
            }))
        }
        StatementKind::Break => Ok(Statement::Break),
        StatementKind::Continue => Ok(Statement::Continue),
        StatementKind::DoWhile(d) => Ok(Statement::DoWhile(DoWhileStatement {
            body: Box::new(Statement::Compound(lower_block(&d.body)?)),
            condition: lower_expression(&d.condition)?,
        })),
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
    match &expr.node {
        ExpressionKind::Literal(lit) => lower_literal(lit),
        ExpressionKind::Variable(name) => {
            // 特殊处理 print 函数，映射到 printf
            if name == "print" {
                Ok(Expression::Variable("printf".to_string()))
            } else {
                Ok(Expression::Variable(name.clone()))
            }
        }
        ExpressionKind::Binary(op, left, right) => Ok(Expression::Binary(
            lower_binary_op(op)?,
            Box::new(lower_expression(left)?),
            Box::new(lower_expression(right)?),
        )),
        ExpressionKind::Unary(op, expr) => Ok(Expression::Unary(
            lower_unary_op(op)?,
            Box::new(lower_expression(expr)?),
        )),
        ExpressionKind::Call(callee, args) => {
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
                        if !matches!(&first_arg.node, ExpressionKind::Literal(ast::Literal::String(_))) {
                            // 对于非字符串参数，自动添加 "%d" 或类似格式
                            return handle_print_call(xir_args);
                        }
                    }
                }
            }

            Ok(Expression::Call(xir_callee, xir_args))
        }
        ExpressionKind::Assign(target, value) => Ok(Expression::Assign(
            Box::new(lower_expression(target)?),
            Box::new(lower_expression(value)?),
        )),
        ExpressionKind::Parenthesized(inner) => Ok(Expression::Parenthesized(Box::new(
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

/// lowering 模式
fn lower_pattern(pattern: &ast::Pattern) -> LowerResult<Pattern> {
    match pattern {
        ast::Pattern::Wildcard => Ok(Pattern::Wildcard),
        ast::Pattern::Variable(name) => Ok(Pattern::Variable(name.clone())),
        ast::Pattern::Literal(lit) => {
            let lowered_lit = match lit {
                ast::Literal::Integer(n) => Literal::Integer(*n),
                ast::Literal::Float(f) => Literal::Double(*f),
                ast::Literal::Boolean(b) => Literal::Bool(*b),
                ast::Literal::String(s) => Literal::String(s.clone()),
                ast::Literal::Char(c) => Literal::Char(*c),
                _ => Literal::Integer(0),
            };
            Ok(Pattern::Literal(lowered_lit))
        }
        ast::Pattern::Array(patterns) => {
            let lowered: Vec<Pattern> = patterns
                .iter()
                .map(lower_pattern)
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Constructor("Array".to_string(), lowered))
        }
        ast::Pattern::Tuple(patterns) => {
            let lowered: Vec<Pattern> = patterns
                .iter()
                .map(lower_pattern)
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Tuple(lowered))
        }
        ast::Pattern::Record(name, fields) => {
            let lowered_fields: Vec<(String, Pattern)> = fields
                .iter()
                .map(|(n, p)| Ok((n.clone(), lower_pattern(p)?)))
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Record(name.clone(), lowered_fields))
        }
        ast::Pattern::Or(left, right) => Ok(Pattern::Or(
            Box::new(lower_pattern(left)?),
            Box::new(lower_pattern(right)?),
        )),
        ast::Pattern::Dictionary(entries) => {
            let patterns: Vec<Pattern> = entries
                .iter()
                .map(|(_, p)| lower_pattern(p))
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Constructor("Dict".to_string(), patterns))
        }
        ast::Pattern::Guard(inner, _) => lower_pattern(inner),
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

// ============================================================================
// HIR → XIR Lowering
// ============================================================================

/// 将 HIR 程序 lowering 为 X IR 程序
pub fn lower_hir_program(hir: &Hir) -> LowerResult<Program> {
    let mut program = Program::new();

    // 添加运行时外部函数声明（如 printf）
    add_runtime_declarations(&mut program);

    // 处理每个声明
    for decl in &hir.declarations {
        let xir_decl = lower_hir_declaration(decl)?;
        program.add(xir_decl);
    }

    // 处理顶层语句（转换为全局初始化或 main 函数中的语句）
    if !hir.statements.is_empty() {
        // 创建一个 main 函数来包含顶层语句
        let mut main_func = Function::new("main", Type::Int);

        // 将顶层语句添加到 main 函数体中
        for stmt in &hir.statements {
            main_func.body.add(lower_hir_statement(stmt)?);
        }

        // 添加 return 0
        main_func.body.add(Statement::Return(Some(Expression::Literal(Literal::Integer(0)))));

        program.add(Declaration::Function(main_func));
    }

    Ok(program)
}

/// lowering HIR 声明
fn lower_hir_declaration(decl: &HirDeclaration) -> LowerResult<Declaration> {
    match decl {
        HirDeclaration::Function(func_decl) => {
            Ok(Declaration::Function(lower_hir_function(func_decl)?))
        }
        HirDeclaration::Variable(var_decl) => {
            Ok(Declaration::Global(lower_hir_global_var(var_decl)?))
        }
        HirDeclaration::TypeAlias(alias) => Ok(Declaration::TypeAlias(TypeAlias {
            name: alias.name.clone(),
            type_: lower_hir_type(&alias.ty)?,
        })),
        HirDeclaration::Class(class_decl) => {
            // 类声明：生成结构体和方法
            let mut decls = Vec::new();

            // 生成结构体类型定义
            // 目前简化处理：生成类型别名
            decls.push(Declaration::TypeAlias(TypeAlias {
                name: class_decl.name.clone(),
                type_: Type::Named(class_decl.name.clone()),
            }));

            // 生成方法函数
            for method in &class_decl.methods {
                decls.push(Declaration::Function(lower_hir_function(method)?));
            }

            // 返回第一个声明（简化处理）
            decls.into_iter().next().ok_or_else(|| {
                LowerError::UnsupportedFeature("空类声明".to_string())
            })
        }
        HirDeclaration::Trait(trait_decl) => {
            // Trait 声明：暂生成类型别名
            Ok(Declaration::TypeAlias(TypeAlias {
                name: trait_decl.name.clone(),
                type_: Type::Named(trait_decl.name.clone()),
            }))
        }
        HirDeclaration::Module(name) => {
            Ok(Declaration::TypeAlias(TypeAlias {
                name: name.clone(),
                type_: Type::Void,
            }))
        }
        HirDeclaration::Import(_) | HirDeclaration::Export(_) => {
            // 导入/导出声明在代码生成时处理
            Ok(Declaration::TypeAlias(TypeAlias {
                name: "_import_export".to_string(),
                type_: Type::Void,
            }))
        }
    }
}

/// lowering HIR 函数声明
fn lower_hir_function(func_decl: &HirFunctionDecl) -> LowerResult<Function> {
    let mut func = Function::new(
        &func_decl.name,
        lower_hir_type(&func_decl.return_type)?,
    );

    // 处理参数
    for param in &func_decl.parameters {
        func = func.param(&param.name, lower_hir_type(&param.ty)?);
    }

    // 处理函数体
    func.body = lower_hir_block(&func_decl.body)?;

    Ok(func)
}

/// lowering HIR 全局变量
fn lower_hir_global_var(var_decl: &HirVariableDecl) -> LowerResult<GlobalVar> {
    let type_ = lower_hir_type(&var_decl.ty)?;
    let initializer = var_decl
        .initializer
        .as_ref()
        .map(|e| lower_hir_expression(e))
        .transpose()?;

    Ok(GlobalVar {
        name: var_decl.name.clone(),
        type_,
        initializer,
        is_static: false,
    })
}

/// lowering HIR 类型
fn lower_hir_type(ty: &HirType) -> LowerResult<Type> {
    match ty {
        HirType::Int => Ok(Type::Int),
        HirType::Float => Ok(Type::Double),
        HirType::Bool => Ok(Type::Bool),
        HirType::String => Ok(Type::Pointer(Box::new(Type::Char))),
        HirType::Char => Ok(Type::Char),
        HirType::Unit => Ok(Type::Void),
        HirType::Never => Ok(Type::Void),
        HirType::Array(inner) => Ok(Type::Array(Box::new(lower_hir_type(inner)?), None)),
        HirType::Option(inner) => {
            // Option<T> 暂时 lowering 为 T*，用 NULL 表示 None
            Ok(Type::Pointer(Box::new(lower_hir_type(inner)?)))
        }
        HirType::Result(ok, _err) => {
            // Result<T, E> 暂时 lowering 为 T
            lower_hir_type(ok)
        }
        HirType::Generic(name) => Ok(Type::Named(name.clone())),
        HirType::TypeParam(name) => Ok(Type::Named(name.clone())),
        HirType::Function(params, ret) => {
            let param_types = params
                .iter()
                .map(lower_hir_type)
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Type::FunctionPointer(
                Box::new(lower_hir_type(ret)?),
                param_types,
            ))
        }
        HirType::Async(inner) => lower_hir_type(inner),
        HirType::Tuple(types) => {
            // 元组 lowering 为结构体
            Ok(Type::Named(format!("Tuple{}", types.len())))
        }
        HirType::Record(name, _fields) => Ok(Type::Named(name.clone())),
        HirType::Union(name, _variants) => Ok(Type::Named(name.clone())),
        HirType::Dictionary(key, value) => {
            Ok(Type::Pointer(Box::new(Type::Named(format!(
                "Map_{}_{}",
                lower_hir_type(key)?,
                lower_hir_type(value)?
            )))))
        }
        HirType::TypeConstructor(name, type_args) => {
            // Generic type application
            let args: Vec<Type> = type_args.iter()
                .map(|t| lower_hir_type(t))
                .collect::<Result<Vec<_>, _>>()?;
            // Create a mangled name for the instantiated type
            let args_str = args.iter()
                .map(|t| format!("{:?}", t))
                .collect::<Vec<_>>()
                .join("_");
            Ok(Type::Named(format!("{}_{}", name, args_str)))
        }
        HirType::Unknown => Ok(Type::Int), // Unknown 类型默认为 Int
        HirType::Dynamic => Ok(Type::Int), // Dynamic 类型默认为 Int（运行时处理）
    }
}

/// lowering HIR 语句块
fn lower_hir_block(block: &HirBlock) -> LowerResult<Block> {
    let mut xir_block = Block::new();

    for stmt in &block.statements {
        xir_block.add(lower_hir_statement(stmt)?);
    }

    Ok(xir_block)
}

/// lowering HIR 语句
fn lower_hir_statement(stmt: &HirStatement) -> LowerResult<Statement> {
    match stmt {
        HirStatement::Expression(expr) => Ok(Statement::Expression(lower_hir_expression(expr)?)),
        HirStatement::Variable(var_decl) => Ok(Statement::Variable(lower_hir_local_var(var_decl)?)),
        HirStatement::Return(expr_opt) => Ok(Statement::Return(
            expr_opt.as_ref().map(|e| lower_hir_expression(e)).transpose()?,
        )),
        HirStatement::If(if_stmt) => {
            let then_branch = Box::new(Statement::Compound(lower_hir_block(&if_stmt.then_block)?));
            let else_branch = if_stmt
                .else_block
                .as_ref()
                .map(|b| Ok::<_, LowerError>(Box::new(Statement::Compound(lower_hir_block(b)?))))
                .transpose()?;

            Ok(Statement::If(IfStatement {
                condition: lower_hir_expression(&if_stmt.condition)?,
                then_branch,
                else_branch,
            }))
        }
        HirStatement::While(while_stmt) => Ok(Statement::While(WhileStatement {
            condition: lower_hir_expression(&while_stmt.condition)?,
            body: Box::new(Statement::Compound(lower_hir_block(&while_stmt.body)?)),
        })),
        HirStatement::For(for_stmt) => {
            let iterator = lower_hir_expression(&for_stmt.iterator)?;
            let body = lower_hir_block(&for_stmt.body)?;

            let loop_var = match &for_stmt.pattern {
                HirPattern::Variable(name) => name.clone(),
                HirPattern::Wildcard => "_".to_string(),
                _ => "_item".to_string(),
            };

            let index_var = format!("_{}_idx", loop_var);

            let init = Statement::Variable(Variable {
                name: index_var.clone(),
                type_: Type::Int,
                initializer: Some(Expression::Literal(Literal::Integer(0))),
                is_static: false,
                is_extern: false,
            });

            let condition = Expression::Binary(
                BinaryOp::LessThan,
                Box::new(Expression::Variable(index_var.clone())),
                Box::new(Expression::Call(
                    Box::new(Expression::Variable("len".to_string())),
                    vec![iterator.clone()],
                )),
            );

            let increment = Expression::Unary(
                UnaryOp::PreIncrement,
                Box::new(Expression::Variable(index_var.clone())),
            );

            let mut for_body = Block::new();
            for_body.add(Statement::Variable(Variable {
                name: loop_var,
                type_: Type::Int,
                initializer: Some(Expression::Index(
                    Box::new(iterator),
                    Box::new(Expression::Variable(index_var.clone())),
                )),
                is_static: false,
                is_extern: false,
            }));
            for stmt in body.statements {
                for_body.add(stmt);
            }

            Ok(Statement::For(ForStatement {
                initializer: Some(Box::new(init)),
                condition: Some(condition),
                increment: Some(increment),
                body: Box::new(Statement::Compound(for_body)),
            }))
        }
        HirStatement::Match(match_stmt) => {
            let scrutinee = lower_hir_expression(&match_stmt.expression)?;
            let mut cases = Vec::new();

            for case in &match_stmt.cases {
                let pattern = lower_hir_pattern(&case.pattern)?;
                let body = lower_hir_block(&case.body)?;
                let guard = case.guard.as_ref()
                    .map(|g| lower_hir_expression(g))
                    .transpose()?;
                cases.push(MatchCase {
                    pattern,
                    body,
                    guard,
                });
            }

            Ok(Statement::Match(MatchStatement {
                scrutinee,
                cases,
            }))
        }
        HirStatement::Try(try_stmt) => {
            let body = lower_hir_block(&try_stmt.body)?;
            let mut catch_clauses = Vec::new();

            for cc in &try_stmt.catch_clauses {
                catch_clauses.push(CatchClause {
                    exception_type: cc.exception_type.clone(),
                    variable_name: cc.variable_name.clone(),
                    body: lower_hir_block(&cc.body)?,
                });
            }

            let finally_block = try_stmt.finally_block.as_ref()
                .map(|b| lower_hir_block(b))
                .transpose()?;

            Ok(Statement::Try(TryStatement {
                body,
                catch_clauses,
                finally_block,
            }))
        }
        HirStatement::Break => Ok(Statement::Break),
        HirStatement::Continue => Ok(Statement::Continue),
    }
}

/// lowering HIR 局部变量声明
fn lower_hir_local_var(var_decl: &HirVariableDecl) -> LowerResult<Variable> {
    let type_ = lower_hir_type(&var_decl.ty)?;
    let initializer = var_decl
        .initializer
        .as_ref()
        .map(|e| lower_hir_expression(e))
        .transpose()?;

    Ok(Variable {
        name: var_decl.name.clone(),
        type_,
        initializer,
        is_static: false,
        is_extern: false,
    })
}

/// lowering HIR 表达式
fn lower_hir_expression(expr: &HirExpression) -> LowerResult<Expression> {
    match expr {
        HirExpression::Literal(lit) => {
            match lit {
                x_hir::HirLiteral::Integer(n) => Ok(Expression::Literal(Literal::Integer(*n))),
                x_hir::HirLiteral::Float(f) => Ok(Expression::Literal(Literal::Double(*f))),
                x_hir::HirLiteral::Boolean(b) => Ok(Expression::Literal(Literal::Bool(*b))),
                x_hir::HirLiteral::String(s) => Ok(Expression::Literal(Literal::String(s.clone()))),
                x_hir::HirLiteral::Char(c) => Ok(Expression::Literal(Literal::Char(*c))),
                x_hir::HirLiteral::Unit => Ok(Expression::Literal(Literal::Integer(0))),
                x_hir::HirLiteral::None => Ok(Expression::Literal(Literal::NullPointer)),
            }
        }
        HirExpression::Variable(name) => {
            if name == "print" {
                Ok(Expression::Variable("printf".to_string()))
            } else {
                Ok(Expression::Variable(name.clone()))
            }
        }
        HirExpression::Binary(op, left, right) => {
            let xir_op = match op {
                x_hir::HirBinaryOp::Add => BinaryOp::Add,
                x_hir::HirBinaryOp::Sub => BinaryOp::Subtract,
                x_hir::HirBinaryOp::Mul => BinaryOp::Multiply,
                x_hir::HirBinaryOp::Div => BinaryOp::Divide,
                x_hir::HirBinaryOp::Mod => BinaryOp::Modulo,
                x_hir::HirBinaryOp::And => BinaryOp::LogicalAnd,
                x_hir::HirBinaryOp::Or => BinaryOp::LogicalOr,
                x_hir::HirBinaryOp::Equal => BinaryOp::Equal,
                x_hir::HirBinaryOp::NotEqual => BinaryOp::NotEqual,
                x_hir::HirBinaryOp::Less => BinaryOp::LessThan,
                x_hir::HirBinaryOp::LessEqual => BinaryOp::LessThanEqual,
                x_hir::HirBinaryOp::Greater => BinaryOp::GreaterThan,
                x_hir::HirBinaryOp::GreaterEqual => BinaryOp::GreaterThanEqual,
                _ => return Err(LowerError::UnsupportedFeature(format!("二元运算符: {:?}", op))),
            };
            Ok(Expression::Binary(
                xir_op,
                Box::new(lower_hir_expression(left)?),
                Box::new(lower_hir_expression(right)?),
            ))
        }
        HirExpression::Unary(op, expr) => {
            let xir_op = match op {
                x_hir::HirUnaryOp::Negate => UnaryOp::Minus,
                x_hir::HirUnaryOp::Not => UnaryOp::Not,
                x_hir::HirUnaryOp::BitNot => UnaryOp::BitNot,
                _ => return Err(LowerError::UnsupportedFeature(format!("一元运算符: {:?}", op))),
            };
            Ok(Expression::Unary(xir_op, Box::new(lower_hir_expression(expr)?)))
        }
        HirExpression::Call(callee, args) => {
            let xir_callee = Box::new(lower_hir_expression(callee)?);
            let mut xir_args = Vec::with_capacity(args.len());

            for arg in args {
                xir_args.push(lower_hir_expression(arg)?);
            }

            Ok(Expression::Call(xir_callee, xir_args))
        }
        HirExpression::Assign(target, value) => Ok(Expression::Assign(
            Box::new(lower_hir_expression(target)?),
            Box::new(lower_hir_expression(value)?),
        )),
        HirExpression::Member(obj, member) => Ok(Expression::Member(
            Box::new(lower_hir_expression(obj)?),
            member.clone(),
        )),
        HirExpression::Array(elements) => {
            // 数组字面量 lowering 为初始化列表
            let mut init_list = Vec::with_capacity(elements.len());
            for elem in elements {
                init_list.push(Initializer::Expression(lower_hir_expression(elem)?));
            }
            Ok(Expression::InitializerList(init_list))
        }
        HirExpression::If(cond, then_expr, else_expr) => {
            Ok(Expression::Ternary(
                Box::new(lower_hir_expression(cond)?),
                Box::new(lower_hir_expression(then_expr)?),
                Box::new(lower_hir_expression(else_expr)?),
            ))
        }
        HirExpression::Lambda(params, body) => {
            // Lambda 表达式 lowering 为函数指针
            let mut func = Function::new("_lambda", Type::Int);
            for param in params {
                func = func.param(&param.name, lower_hir_type(&param.ty)?);
            }
            func.body = lower_hir_block(body)?;
            // 注意：这里简化处理，实际应该生成一个唯一的函数名
            Ok(Expression::Variable("_lambda".to_string()))
        }
        HirExpression::Pipe(input, functions) => {
            // 管道表达式 lowering 为嵌套调用
            let mut result = lower_hir_expression(input)?;
            for func in functions {
                result = Expression::Call(
                    Box::new(lower_hir_expression(func)?),
                    vec![result],
                );
            }
            Ok(result)
        }
        HirExpression::Record(name, fields) => {
            // 记录表达式 lowering 为结构体初始化
            let mut init_list = Vec::new();
            for (field_name, field_expr) in fields {
                init_list.push(Initializer::Named(
                    field_name.clone(),
                    Box::new(Initializer::Expression(lower_hir_expression(field_expr)?)),
                ));
            }
            Ok(Expression::CompoundLiteral(Type::Named(name.clone()), init_list))
        }
        HirExpression::Range(start, end, _inclusive) => {
            // 范围表达式 lowering 为数组初始化
            Ok(Expression::InitializerList(vec![
                Initializer::Expression(lower_hir_expression(start)?),
                Initializer::Expression(lower_hir_expression(end)?),
            ]))
        }
        HirExpression::Dictionary(entries) => {
            // 字典表达式暂 lowering 为空
            let _ = entries;
            Ok(Expression::Literal(Literal::NullPointer))
        }
        HirExpression::Wait(_wait_type, exprs) => {
            // Wait 表达式暂 lowering 为第一个表达式
            if exprs.is_empty() {
                Ok(Expression::Literal(Literal::NullPointer))
            } else {
                lower_hir_expression(&exprs[0])
            }
        }
        HirExpression::Needs(_effect) => {
            // Needs 表达式 lowering 为空
            Ok(Expression::Literal(Literal::Integer(0)))
        }
        HirExpression::Given(_effect, expr) => {
            lower_hir_expression(expr)
        }
        HirExpression::Handle(_inner, _handlers) => {
            // Handle 表达式 lowering - 暂时返回 0 作为占位符
            Ok(Expression::Literal(Literal::Integer(0)))
        }
        HirExpression::Typed(inner, _ty) => {
            // 类型注解表达式 lowering 为内部表达式
            lower_hir_expression(inner)
        }
        HirExpression::TryPropagate(inner_expr) => {
            // ? 运算符：lowering 为条件检查
            let inner = lower_hir_expression(inner_expr)?;
            // 创建一个简单的表达式，实际实现需要错误处理支持
            Ok(inner)
        }
    }
}

/// lowering HIR 模式
fn lower_hir_pattern(pattern: &HirPattern) -> LowerResult<Pattern> {
    match pattern {
        HirPattern::Wildcard => Ok(Pattern::Wildcard),
        HirPattern::Variable(name) => Ok(Pattern::Variable(name.clone())),
        HirPattern::Literal(lit) => {
            let lowered_lit = match lit {
                x_hir::HirLiteral::Integer(n) => Literal::Integer(*n),
                x_hir::HirLiteral::Float(f) => Literal::Double(*f),
                x_hir::HirLiteral::Boolean(b) => Literal::Bool(*b),
                x_hir::HirLiteral::String(s) => Literal::String(s.clone()),
                x_hir::HirLiteral::Char(c) => Literal::Char(*c),
                _ => Literal::Integer(0),
            };
            Ok(Pattern::Literal(lowered_lit))
        }
        HirPattern::Array(patterns) => {
            let lowered: Vec<Pattern> = patterns
                .iter()
                .map(lower_hir_pattern)
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Constructor("Array".to_string(), lowered))
        }
        HirPattern::Tuple(patterns) => {
            let lowered: Vec<Pattern> = patterns
                .iter()
                .map(lower_hir_pattern)
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Tuple(lowered))
        }
        HirPattern::Record(name, fields) => {
            let lowered_fields: Vec<(String, Pattern)> = fields
                .iter()
                .map(|(n, p)| Ok((n.clone(), lower_hir_pattern(p)?)))
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Record(name.clone(), lowered_fields))
        }
        HirPattern::Or(left, right) => Ok(Pattern::Or(
            Box::new(lower_hir_pattern(left)?),
            Box::new(lower_hir_pattern(right)?),
        )),
        HirPattern::Dictionary(entries) => {
            let patterns: Vec<Pattern> = entries
                .iter()
                .map(|(_, p)| lower_hir_pattern(p))
                .collect::<LowerResult<Vec<_>>>()?;
            Ok(Pattern::Constructor("Dict".to_string(), patterns))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::MethodModifiers;

    #[test]
    fn test_lower_simple_function() {
        // 创建一个简单的 AST 函数并 lowering
        let ast_func = ast::FunctionDecl {
            span: Span::default(),
            name: "main".to_string(),
            type_parameters: vec![],
            parameters: vec![],
            return_type: Some(ast::Type::Int),
            effects: vec![],
            body: ast::Block { statements: vec![] },
            is_async: false,
            modifiers: MethodModifiers::default(),
        };

        let result = lower_function(&ast_func);
        assert!(result.is_ok());

        let func = result.unwrap();
        assert_eq!(func.name, "main");
        assert_eq!(func.return_type, Type::Int);
    }

    #[test]
    fn test_lower_hir_simple_program() {
        // 创建一个简单的 HIR 程序
        let hir = Hir {
            module_name: "main".to_string(),
            declarations: vec![],
            statements: vec![],
            type_env: x_hir::HirTypeEnv {
                variables: std::collections::HashMap::new(),
                functions: std::collections::HashMap::new(),
                types: std::collections::HashMap::new(),
            },
            perceus_info: x_hir::HirPerceusInfo::default(),
        };

        let result = lower_hir_program(&hir);
        assert!(result.is_ok());

        let program = result.unwrap();
        // 应该有标准库的外部函数声明
        assert!(!program.declarations.is_empty());
    }

    #[test]
    fn test_lower_hir_function() {
        // 创建一个简单的 HIR 函数
        let hir_func = HirFunctionDecl {
            name: "add".to_string(),
            parameters: vec![
                x_hir::HirParameter {
                    name: "a".to_string(),
                    ty: HirType::Int,
                    default: None,
                },
                x_hir::HirParameter {
                    name: "b".to_string(),
                    ty: HirType::Int,
                    default: None,
                },
            ],
            return_type: HirType::Int,
            body: x_hir::HirBlock { statements: vec![] },
            is_async: false,
            effects: vec![],
        };

        let result = lower_hir_function(&hir_func);
        assert!(result.is_ok());

        let func = result.unwrap();
        assert_eq!(func.name, "add");
        assert_eq!(func.return_type, Type::Int);
    }

    #[test]
    fn test_lower_hir_types() {
        assert_eq!(lower_hir_type(&HirType::Int).unwrap(), Type::Int);
        assert_eq!(lower_hir_type(&HirType::Float).unwrap(), Type::Double);
        assert_eq!(lower_hir_type(&HirType::Bool).unwrap(), Type::Bool);
        assert_eq!(lower_hir_type(&HirType::Char).unwrap(), Type::Char);
        assert_eq!(lower_hir_type(&HirType::Unit).unwrap(), Type::Void);
    }
}
