// 高级中间表示库
//
// HIR (High-level Intermediate Representation) 是 X 语言编译器的中间表示层，
// 位于 AST 和代码生成之间。HIR 提供了：
// - 规范化的语法表示（消除语法糖）
// - 类型信息标注
// - 便于优化和代码生成的结构

use std::collections::HashMap;
use x_parser::ast::{self, BinaryOp, Literal, Type, UnaryOp};

/// HIR 根结构
#[derive(Debug, PartialEq, Clone)]
pub struct Hir {
    /// 模块名称（当前为单文件，默认 "main"）
    pub module_name: String,
    /// 顶层声明
    pub declarations: Vec<HirDeclaration>,
    /// 顶层语句
    pub statements: Vec<HirStatement>,
    /// 类型环境（符号表）
    pub type_env: HirTypeEnv,
}

/// 类型环境
#[derive(Debug, PartialEq, Clone)]
pub struct HirTypeEnv {
    /// 变量类型映射
    pub variables: HashMap<String, HirTypeInfo>,
    /// 函数类型映射
    pub functions: HashMap<String, HirFunctionInfo>,
    /// 类型定义
    pub types: HashMap<String, HirTypeDef>,
}

/// 类型信息
#[derive(Debug, PartialEq, Clone)]
pub struct HirTypeInfo {
    pub ty: HirType,
    pub is_mutable: bool,
    pub source: TypeInfoSource,
}

/// 类型信息来源
#[derive(Debug, PartialEq, Clone)]
pub enum TypeInfoSource {
    /// 显式类型注解
    Annotated,
    /// 类型推断
    Inferred,
    /// 默认类型
    Default,
}

/// 函数信息
#[derive(Debug, PartialEq, Clone)]
pub struct HirFunctionInfo {
    pub name: String,
    pub parameters: Vec<(String, HirType)>,
    pub return_type: HirType,
    pub is_async: bool,
}

/// 类型定义
#[derive(Debug, PartialEq, Clone)]
pub struct HirTypeDef {
    pub name: String,
    pub kind: HirTypeDefKind,
}

/// 类型定义种类
#[derive(Debug, PartialEq, Clone)]
pub enum HirTypeDefKind {
    /// 类型别名
    Alias(HirType),
    /// 记录类型
    Record(Vec<(String, HirType)>),
    /// 联合类型
    Union(Vec<HirType>),
}

/// HIR 声明
#[derive(Debug, PartialEq, Clone)]
pub enum HirDeclaration {
    /// 变量声明
    Variable(HirVariableDecl),
    /// 函数声明
    Function(HirFunctionDecl),
    /// 类声明
    Class(HirClassDecl),
    /// Trait 声明
    Trait(HirTraitDecl),
    /// 类型别名
    TypeAlias(HirTypeAlias),
    /// 模块声明
    Module(String),
    /// 导入声明
    Import(HirImportDecl),
    /// 导出声明
    Export(String),
}

/// 变量声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirVariableDecl {
    pub name: String,
    pub is_mutable: bool,
    pub ty: HirType,
    pub initializer: Option<HirExpression>,
}

/// 函数声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirFunctionDecl {
    pub name: String,
    pub parameters: Vec<HirParameter>,
    pub return_type: HirType,
    pub body: HirBlock,
    pub is_async: bool,
    /// 效果注解
    pub effects: Vec<String>,
}

/// 参数
#[derive(Debug, PartialEq, Clone)]
pub struct HirParameter {
    pub name: String,
    pub ty: HirType,
    pub default: Option<HirExpression>,
}

/// 类声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirClassDecl {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub fields: Vec<HirVariableDecl>,
    pub methods: Vec<HirFunctionDecl>,
}

/// Trait 声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirTraitDecl {
    pub name: String,
    pub methods: Vec<HirFunctionDecl>,
}

/// 类型别名
#[derive(Debug, PartialEq, Clone)]
pub struct HirTypeAlias {
    pub name: String,
    pub ty: HirType,
}

/// 导入声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirImportDecl {
    pub module_path: String,
    pub symbols: Vec<HirImportSymbol>,
}

/// 导入符号
#[derive(Debug, PartialEq, Clone)]
pub enum HirImportSymbol {
    All,
    Named(String, Option<String>), // (原名, 别名)
}

/// HIR 语句
#[derive(Debug, PartialEq, Clone)]
pub enum HirStatement {
    /// 表达式语句
    Expression(HirExpression),
    /// 变量声明
    Variable(HirVariableDecl),
    /// 返回语句
    Return(Option<HirExpression>),
    /// If 语句
    If(HirIfStatement),
    /// For 循环
    For(HirForStatement),
    /// While 循环
    While(HirWhileStatement),
    /// Match 语句
    Match(HirMatchStatement),
    /// Try 语句
    Try(HirTryStatement),
    /// Break
    Break,
    /// Continue
    Continue,
}

/// If 语句
#[derive(Debug, PartialEq, Clone)]
pub struct HirIfStatement {
    pub condition: HirExpression,
    pub then_block: HirBlock,
    pub else_block: Option<HirBlock>,
}

/// For 循环
#[derive(Debug, PartialEq, Clone)]
pub struct HirForStatement {
    pub pattern: HirPattern,
    pub iterator: HirExpression,
    pub body: HirBlock,
}

/// While 循环
#[derive(Debug, PartialEq, Clone)]
pub struct HirWhileStatement {
    pub condition: HirExpression,
    pub body: HirBlock,
}

/// Match 语句
#[derive(Debug, PartialEq, Clone)]
pub struct HirMatchStatement {
    pub expression: HirExpression,
    pub cases: Vec<HirMatchCase>,
}

/// Match case
#[derive(Debug, PartialEq, Clone)]
pub struct HirMatchCase {
    pub pattern: HirPattern,
    pub body: HirBlock,
    pub guard: Option<HirExpression>,
}

/// Try 语句
#[derive(Debug, PartialEq, Clone)]
pub struct HirTryStatement {
    pub body: HirBlock,
    pub catch_clauses: Vec<HirCatchClause>,
    pub finally_block: Option<HirBlock>,
}

/// Catch 子句
#[derive(Debug, PartialEq, Clone)]
pub struct HirCatchClause {
    pub exception_type: Option<String>,
    pub variable_name: Option<String>,
    pub body: HirBlock,
}

/// HIR 块
#[derive(Debug, PartialEq, Clone)]
pub struct HirBlock {
    pub statements: Vec<HirStatement>,
}

/// HIR 表达式
#[derive(Debug, PartialEq, Clone)]
pub enum HirExpression {
    /// 字面量
    Literal(HirLiteral),
    /// 变量引用
    Variable(String),
    /// 成员访问
    Member(Box<HirExpression>, String),
    /// 函数调用
    Call(Box<HirExpression>, Vec<HirExpression>),
    /// 二元运算
    Binary(HirBinaryOp, Box<HirExpression>, Box<HirExpression>),
    /// 一元运算
    Unary(HirUnaryOp, Box<HirExpression>),
    /// 赋值
    Assign(Box<HirExpression>, Box<HirExpression>),
    /// If 表达式
    If(Box<HirExpression>, Box<HirExpression>, Box<HirExpression>),
    /// Lambda
    Lambda(Vec<HirParameter>, HirBlock),
    /// 数组
    Array(Vec<HirExpression>),
    /// 字典
    Dictionary(Vec<(HirExpression, HirExpression)>),
    /// 记录
    Record(String, Vec<(String, HirExpression)>),
    /// 范围
    Range(Box<HirExpression>, Box<HirExpression>, bool),
    /// 管道
    Pipe(Box<HirExpression>, Vec<HirExpression>),
    /// 等待异步操作
    Wait(HirWaitType, Vec<HirExpression>),
    /// 需要效果
    Needs(String),
    /// 给定效果
    Given(String, Box<HirExpression>),
    /// 类型注解表达式
    Typed(Box<HirExpression>, HirType),
}

/// HIR 字面量
#[derive(Debug, PartialEq, Clone)]
pub enum HirLiteral {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Unit,
    None,
}

/// HIR 二元运算符
#[derive(Debug, PartialEq, Clone)]
pub enum HirBinaryOp {
    Add, Sub, Mul, Div, Mod, Pow,
    And, Or,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    BitAnd, BitOr, BitXor, LeftShift, RightShift,
    Concat,
}

/// HIR 一元运算符
#[derive(Debug, PartialEq, Clone)]
pub enum HirUnaryOp {
    Negate, Not, BitNot, Await,
}

/// HIR Wait 类型
#[derive(Debug, PartialEq, Clone)]
pub enum HirWaitType {
    Single,
    Together,
    Race,
    Timeout(Box<HirExpression>),
}

/// HIR 模式
#[derive(Debug, PartialEq, Clone)]
pub enum HirPattern {
    Wildcard,
    Variable(String),
    Literal(HirLiteral),
    Array(Vec<HirPattern>),
    Dictionary(Vec<(HirPattern, HirPattern)>),
    Record(String, Vec<(String, HirPattern)>),
    Tuple(Vec<HirPattern>),
    Or(Box<HirPattern>, Box<HirPattern>),
}

/// HIR 类型
#[derive(Debug, PartialEq, Clone)]
pub enum HirType {
    // 基本类型
    Int,
    Float,
    Bool,
    String,
    Char,
    Unit,
    Never,

    // 复合类型
    Array(Box<HirType>),
    Dictionary(Box<HirType>, Box<HirType>),
    Record(String, Vec<(String, Box<HirType>)>),
    Union(String, Vec<HirType>),
    Tuple(Vec<HirType>),

    // 高级类型
    Option(Box<HirType>),
    Result(Box<HirType>, Box<HirType>),
    Function(Vec<HirType>, Box<HirType>),
    Async(Box<HirType>),

    // 泛型类型
    Generic(String),
    TypeParam(String),

    // 未知类型（推断失败时使用）
    Unknown,
}

impl HirType {
    /// 从 AST Type 转换
    pub fn from_ast(ty: &Type) -> Self {
        match ty {
            Type::Int => HirType::Int,
            Type::Float => HirType::Float,
            Type::Bool => HirType::Bool,
            Type::String => HirType::String,
            Type::Char => HirType::Char,
            Type::Unit => HirType::Unit,
            Type::Never => HirType::Never,
            Type::Array(inner) => HirType::Array(Box::new(HirType::from_ast(inner))),
            Type::Dictionary(k, v) => HirType::Dictionary(
                Box::new(HirType::from_ast(k)),
                Box::new(HirType::from_ast(v)),
            ),
            Type::Record(name, fields) => HirType::Record(
                name.clone(),
                fields.iter().map(|(n, t)| (n.clone(), Box::new(HirType::from_ast(t)))).collect(),
            ),
            Type::Union(name, variants) => HirType::Union(
                name.clone(),
                variants.iter().map(HirType::from_ast).collect(),
            ),
            Type::Tuple(types) => HirType::Tuple(types.iter().map(HirType::from_ast).collect()),
            Type::Option(inner) => HirType::Option(Box::new(HirType::from_ast(inner))),
            Type::Result(ok, err) => HirType::Result(
                Box::new(HirType::from_ast(ok)),
                Box::new(HirType::from_ast(err)),
            ),
            Type::Function(params, ret) => HirType::Function(
                params.iter().map(|p| HirType::from_ast(p)).collect(),
                Box::new(HirType::from_ast(ret)),
            ),
            Type::Async(inner) => HirType::Async(Box::new(HirType::from_ast(inner))),
            Type::Generic(name) => HirType::Generic(name.clone()),
            Type::TypeParam(name) => HirType::TypeParam(name.clone()),
            Type::Var(name) => HirType::Generic(name.clone()),
        }
    }
}

/// 高级中间表示错误
#[derive(thiserror::Error, Debug)]
pub enum HirError {
    #[error("转换错误: {0}")]
    ConversionError(String),

    #[error("未定义的变量: {0}")]
    UndefinedVariable(String),

    #[error("未定义的函数: {0}")]
    UndefinedFunction(String),

    #[error("重复声明: {0}")]
    DuplicateDeclaration(String),

    #[error("类型错误: {0}")]
    TypeError(String),
}

/// HIR 转换器
pub struct HirConverter {
    /// 当前作用域变量
    variables: HashMap<String, HirType>,
    /// 函数签名
    functions: HashMap<String, HirFunctionInfo>,
}

impl HirConverter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// 转换程序
    pub fn convert_program(&mut self, program: &ast::Program) -> Result<Hir, HirError> {
        let mut declarations = Vec::new();
        let mut statements = Vec::new();

        // 转换声明
        for decl in &program.declarations {
            declarations.push(self.convert_declaration(decl)?);
        }

        // 转换语句
        for stmt in &program.statements {
            statements.push(self.convert_statement(stmt)?);
        }

        // 构建类型环境
        let type_env = HirTypeEnv {
            variables: self.variables.iter().map(|(k, v)| {
                (k.clone(), HirTypeInfo {
                    ty: v.clone(),
                    is_mutable: false,
                    source: TypeInfoSource::Inferred,
                })
            }).collect(),
            functions: self.functions.clone(),
            types: HashMap::new(),
        };

        Ok(Hir {
            module_name: "main".to_string(),
            declarations,
            statements,
            type_env,
        })
    }

    /// 转换声明
    fn convert_declaration(&mut self, decl: &ast::Declaration) -> Result<HirDeclaration, HirError> {
        match decl {
            ast::Declaration::Variable(var_decl) => {
                let hir_decl = self.convert_variable_decl(var_decl)?;
                self.variables.insert(hir_decl.name.clone(), hir_decl.ty.clone());
                Ok(HirDeclaration::Variable(hir_decl))
            }
            ast::Declaration::Function(func_decl) => {
                let hir_decl = self.convert_function_decl(func_decl)?;
                self.functions.insert(hir_decl.name.clone(), HirFunctionInfo {
                    name: hir_decl.name.clone(),
                    parameters: hir_decl.parameters.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                    return_type: hir_decl.return_type.clone(),
                    is_async: hir_decl.is_async,
                });
                Ok(HirDeclaration::Function(hir_decl))
            }
            ast::Declaration::Class(class_decl) => {
                Ok(HirDeclaration::Class(HirClassDecl {
                    name: class_decl.name.clone(),
                    extends: class_decl.extends.clone(),
                    implements: class_decl.implements.clone(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                }))
            }
            ast::Declaration::Trait(trait_decl) => {
                Ok(HirDeclaration::Trait(HirTraitDecl {
                    name: trait_decl.name.clone(),
                    methods: Vec::new(),
                }))
            }
            ast::Declaration::TypeAlias(type_alias) => {
                Ok(HirDeclaration::TypeAlias(HirTypeAlias {
                    name: type_alias.name.clone(),
                    ty: HirType::from_ast(&type_alias.type_),
                }))
            }
            ast::Declaration::Module(module_decl) => {
                Ok(HirDeclaration::Module(module_decl.name.clone()))
            }
            ast::Declaration::Import(import_decl) => {
                Ok(HirDeclaration::Import(HirImportDecl {
                    module_path: import_decl.module_path.clone(),
                    symbols: import_decl.symbols.iter().map(|s| match s {
                        ast::ImportSymbol::All => HirImportSymbol::All,
                        ast::ImportSymbol::Named(name, alias) => HirImportSymbol::Named(name.clone(), alias.clone()),
                    }).collect(),
                }))
            }
            ast::Declaration::Export(export_decl) => {
                Ok(HirDeclaration::Export(export_decl.symbol.clone()))
            }
        }
    }

    /// 转换变量声明
    fn convert_variable_decl(&mut self, var_decl: &ast::VariableDecl) -> Result<HirVariableDecl, HirError> {
        let ty = if let Some(type_annot) = &var_decl.type_annot {
            HirType::from_ast(type_annot)
        } else if let Some(initializer) = &var_decl.initializer {
            self.infer_expression_type(initializer)
        } else {
            HirType::Unknown
        };

        let initializer = if let Some(init) = &var_decl.initializer {
            Some(self.convert_expression(init)?)
        } else {
            None
        };

        Ok(HirVariableDecl {
            name: var_decl.name.clone(),
            is_mutable: var_decl.is_mutable,
            ty,
            initializer,
        })
    }

    /// 转换函数声明
    fn convert_function_decl(&mut self, func_decl: &ast::FunctionDecl) -> Result<HirFunctionDecl, HirError> {
        // 保存当前作用域
        let outer_vars = self.variables.clone();

        // 转换参数
        let mut parameters = Vec::new();
        for param in &func_decl.parameters {
            let ty = if let Some(type_annot) = &param.type_annot {
                HirType::from_ast(type_annot)
            } else {
                HirType::Unknown
            };
            self.variables.insert(param.name.clone(), ty.clone());
            parameters.push(HirParameter {
                name: param.name.clone(),
                ty,
                default: if let Some(default) = &param.default {
                    Some(self.convert_expression(default)?)
                } else {
                    None
                },
            });
        }

        // 转换返回类型
        let return_type = if let Some(ret_type) = &func_decl.return_type {
            HirType::from_ast(ret_type)
        } else {
            HirType::Unit
        };

        // 转换函数体
        let body = self.convert_block(&func_decl.body)?;

        // 恢复作用域
        self.variables = outer_vars;

        Ok(HirFunctionDecl {
            name: func_decl.name.clone(),
            parameters,
            return_type,
            body,
            is_async: func_decl.is_async,
            effects: Vec::new(),
        })
    }

    /// 转换语句
    fn convert_statement(&mut self, stmt: &ast::Statement) -> Result<HirStatement, HirError> {
        match stmt {
            ast::Statement::Expression(expr) => {
                Ok(HirStatement::Expression(self.convert_expression(expr)?))
            }
            ast::Statement::Variable(var_decl) => {
                let hir_decl = self.convert_variable_decl(var_decl)?;
                self.variables.insert(hir_decl.name.clone(), hir_decl.ty.clone());
                Ok(HirStatement::Variable(hir_decl))
            }
            ast::Statement::Return(expr_opt) => {
                let hir_expr = if let Some(expr) = expr_opt {
                    Some(self.convert_expression(expr)?)
                } else {
                    None
                };
                Ok(HirStatement::Return(hir_expr))
            }
            ast::Statement::If(if_stmt) => {
                Ok(HirStatement::If(HirIfStatement {
                    condition: self.convert_expression(&if_stmt.condition)?,
                    then_block: self.convert_block(&if_stmt.then_block)?,
                    else_block: if let Some(else_block) = &if_stmt.else_block {
                        Some(self.convert_block(else_block)?)
                    } else {
                        None
                    },
                }))
            }
            ast::Statement::For(for_stmt) => {
                Ok(HirStatement::For(HirForStatement {
                    pattern: self.convert_pattern(&for_stmt.pattern),
                    iterator: self.convert_expression(&for_stmt.iterator)?,
                    body: self.convert_block(&for_stmt.body)?,
                }))
            }
            ast::Statement::While(while_stmt) => {
                Ok(HirStatement::While(HirWhileStatement {
                    condition: self.convert_expression(&while_stmt.condition)?,
                    body: self.convert_block(&while_stmt.body)?,
                }))
            }
            ast::Statement::Match(match_stmt) => {
                let mut cases = Vec::new();
                for case in &match_stmt.cases {
                    cases.push(HirMatchCase {
                        pattern: self.convert_pattern(&case.pattern),
                        body: self.convert_block(&case.body)?,
                        guard: if let Some(guard) = &case.guard {
                            Some(self.convert_expression(guard)?)
                        } else {
                            None
                        },
                    });
                }
                Ok(HirStatement::Match(HirMatchStatement {
                    expression: self.convert_expression(&match_stmt.expression)?,
                    cases,
                }))
            }
            ast::Statement::Try(try_stmt) => {
                let mut catch_clauses = Vec::new();
                for cc in &try_stmt.catch_clauses {
                    catch_clauses.push(HirCatchClause {
                        exception_type: cc.exception_type.clone(),
                        variable_name: cc.variable_name.clone(),
                        body: self.convert_block(&cc.body)?,
                    });
                }
                Ok(HirStatement::Try(HirTryStatement {
                    body: self.convert_block(&try_stmt.body)?,
                    catch_clauses,
                    finally_block: if let Some(finally) = &try_stmt.finally_block {
                        Some(self.convert_block(finally)?)
                    } else {
                        None
                    },
                }))
            }
            ast::Statement::Break => Ok(HirStatement::Break),
            ast::Statement::Continue => Ok(HirStatement::Continue),
            ast::Statement::DoWhile(dw) => {
                // DoWhile 转换为 While（语义等价，仅顺序不同）
                Ok(HirStatement::While(HirWhileStatement {
                    condition: self.convert_expression(&dw.condition)?,
                    body: self.convert_block(&dw.body)?,
                }))
            }
        }
    }

    /// 转换块
    fn convert_block(&mut self, block: &ast::Block) -> Result<HirBlock, HirError> {
        let mut statements = Vec::new();
        for stmt in &block.statements {
            statements.push(self.convert_statement(stmt)?);
        }
        Ok(HirBlock { statements })
    }

    /// 转换表达式
    fn convert_expression(&mut self, expr: &ast::Expression) -> Result<HirExpression, HirError> {
        match expr {
            ast::Expression::Literal(lit) => {
                Ok(HirExpression::Literal(self.convert_literal(lit)))
            }
            ast::Expression::Variable(name) => {
                Ok(HirExpression::Variable(name.clone()))
            }
            ast::Expression::Member(obj, member) => {
                Ok(HirExpression::Member(
                    Box::new(self.convert_expression(obj)?),
                    member.clone(),
                ))
            }
            ast::Expression::Call(callee, args) => {
                let mut hir_args = Vec::new();
                for arg in args {
                    hir_args.push(self.convert_expression(arg)?);
                }
                Ok(HirExpression::Call(
                    Box::new(self.convert_expression(callee)?),
                    hir_args,
                ))
            }
            ast::Expression::Binary(op, left, right) => {
                Ok(HirExpression::Binary(
                    self.convert_binary_op(op),
                    Box::new(self.convert_expression(left)?),
                    Box::new(self.convert_expression(right)?),
                ))
            }
            ast::Expression::Unary(op, expr) => {
                Ok(HirExpression::Unary(
                    self.convert_unary_op(op),
                    Box::new(self.convert_expression(expr)?),
                ))
            }
            ast::Expression::Assign(lhs, rhs) => {
                Ok(HirExpression::Assign(
                    Box::new(self.convert_expression(lhs)?),
                    Box::new(self.convert_expression(rhs)?),
                ))
            }
            ast::Expression::If(cond, then_expr, else_expr) => {
                Ok(HirExpression::If(
                    Box::new(self.convert_expression(cond)?),
                    Box::new(self.convert_expression(then_expr)?),
                    Box::new(self.convert_expression(else_expr)?),
                ))
            }
            ast::Expression::Lambda(params, body) => {
                // 保存当前作用域
                let outer_vars = self.variables.clone();

                let mut hir_params = Vec::new();
                for param in params {
                    let ty = if let Some(type_annot) = &param.type_annot {
                        HirType::from_ast(type_annot)
                    } else {
                        HirType::Unknown
                    };
                    self.variables.insert(param.name.clone(), ty.clone());
                    hir_params.push(HirParameter {
                        name: param.name.clone(),
                        ty,
                        default: None,
                    });
                }

                let hir_body = self.convert_block(body)?;

                // 恢复作用域
                self.variables = outer_vars;

                Ok(HirExpression::Lambda(hir_params, hir_body))
            }
            ast::Expression::Array(items) => {
                let mut hir_items = Vec::new();
                for item in items {
                    hir_items.push(self.convert_expression(item)?);
                }
                Ok(HirExpression::Array(hir_items))
            }
            ast::Expression::Dictionary(entries) => {
                let mut hir_entries = Vec::new();
                for (k, v) in entries {
                    hir_entries.push((
                        self.convert_expression(k)?,
                        self.convert_expression(v)?,
                    ));
                }
                Ok(HirExpression::Dictionary(hir_entries))
            }
            ast::Expression::Record(name, fields) => {
                let mut hir_fields = Vec::new();
                for (field_name, field_expr) in fields {
                    hir_fields.push((field_name.clone(), self.convert_expression(field_expr)?));
                }
                Ok(HirExpression::Record(name.clone(), hir_fields))
            }
            ast::Expression::Range(start, end, inclusive) => {
                Ok(HirExpression::Range(
                    Box::new(self.convert_expression(start)?),
                    Box::new(self.convert_expression(end)?),
                    *inclusive,
                ))
            }
            ast::Expression::Pipe(input, functions) => {
                let mut hir_funcs = Vec::new();
                for func in functions {
                    hir_funcs.push(self.convert_expression(func)?);
                }
                Ok(HirExpression::Pipe(
                    Box::new(self.convert_expression(input)?),
                    hir_funcs,
                ))
            }
            ast::Expression::Wait(wait_type, exprs) => {
                let hir_wait_type = match wait_type {
                    ast::WaitType::Single => HirWaitType::Single,
                    ast::WaitType::Together => HirWaitType::Together,
                    ast::WaitType::Race => HirWaitType::Race,
                    ast::WaitType::Timeout(timeout_expr) => {
                        HirWaitType::Timeout(Box::new(self.convert_expression(timeout_expr)?))
                    }
                };
                let mut hir_exprs = Vec::new();
                for expr in exprs {
                    hir_exprs.push(self.convert_expression(expr)?);
                }
                Ok(HirExpression::Wait(hir_wait_type, hir_exprs))
            }
            ast::Expression::Needs(effect_name) => {
                Ok(HirExpression::Needs(effect_name.clone()))
            }
            ast::Expression::Given(effect_name, expr) => {
                Ok(HirExpression::Given(
                    effect_name.clone(),
                    Box::new(self.convert_expression(expr)?),
                ))
            }
            ast::Expression::Parenthesized(inner) => {
                self.convert_expression(inner)
            }
        }
    }

    /// 转换字面量
    fn convert_literal(&self, lit: &Literal) -> HirLiteral {
        match lit {
            Literal::Integer(n) => HirLiteral::Integer(*n),
            Literal::Float(f) => HirLiteral::Float(*f),
            Literal::Boolean(b) => HirLiteral::Boolean(*b),
            Literal::String(s) => HirLiteral::String(s.clone()),
            Literal::Char(c) => HirLiteral::Char(*c),
            Literal::Null | Literal::Unit => HirLiteral::Unit,
            Literal::None => HirLiteral::None,
        }
    }

    /// 转换二元运算符
    fn convert_binary_op(&self, op: &BinaryOp) -> HirBinaryOp {
        match op {
            BinaryOp::Add => HirBinaryOp::Add,
            BinaryOp::Sub => HirBinaryOp::Sub,
            BinaryOp::Mul => HirBinaryOp::Mul,
            BinaryOp::Div => HirBinaryOp::Div,
            BinaryOp::Mod => HirBinaryOp::Mod,
            BinaryOp::Pow => HirBinaryOp::Pow,
            BinaryOp::And => HirBinaryOp::And,
            BinaryOp::Or => HirBinaryOp::Or,
            BinaryOp::Equal => HirBinaryOp::Equal,
            BinaryOp::NotEqual => HirBinaryOp::NotEqual,
            BinaryOp::Less => HirBinaryOp::Less,
            BinaryOp::LessEqual => HirBinaryOp::LessEqual,
            BinaryOp::Greater => HirBinaryOp::Greater,
            BinaryOp::GreaterEqual => HirBinaryOp::GreaterEqual,
            BinaryOp::BitAnd => HirBinaryOp::BitAnd,
            BinaryOp::BitOr => HirBinaryOp::BitOr,
            BinaryOp::BitXor => HirBinaryOp::BitXor,
            BinaryOp::LeftShift => HirBinaryOp::LeftShift,
            BinaryOp::RightShift => HirBinaryOp::RightShift,
            BinaryOp::Concat => HirBinaryOp::Concat,
            BinaryOp::RangeExclusive | BinaryOp::RangeInclusive => HirBinaryOp::Concat, // 这些应该在 Range 表达式中处理
        }
    }

    /// 转换一元运算符
    fn convert_unary_op(&self, op: &UnaryOp) -> HirUnaryOp {
        match op {
            UnaryOp::Negate => HirUnaryOp::Negate,
            UnaryOp::Not => HirUnaryOp::Not,
            UnaryOp::BitNot => HirUnaryOp::BitNot,
            UnaryOp::Wait => HirUnaryOp::Await,
        }
    }

    /// 转换模式
    fn convert_pattern(&self, pattern: &ast::Pattern) -> HirPattern {
        match pattern {
            ast::Pattern::Wildcard => HirPattern::Wildcard,
            ast::Pattern::Variable(name) => HirPattern::Variable(name.clone()),
            ast::Pattern::Literal(lit) => HirPattern::Literal(self.convert_literal(lit)),
            ast::Pattern::Array(patterns) => {
                HirPattern::Array(patterns.iter().map(|p| self.convert_pattern(p)).collect())
            }
            ast::Pattern::Dictionary(entries) => {
                HirPattern::Dictionary(entries.iter().map(|(k, v)| {
                    (self.convert_pattern(k), self.convert_pattern(v))
                }).collect())
            }
            ast::Pattern::Record(name, fields) => {
                HirPattern::Record(name.clone(), fields.iter().map(|(n, p)| {
                    (n.clone(), self.convert_pattern(p))
                }).collect())
            }
            ast::Pattern::Tuple(patterns) => {
                HirPattern::Tuple(patterns.iter().map(|p| self.convert_pattern(p)).collect())
            }
            ast::Pattern::Or(left, right) => {
                HirPattern::Or(
                    Box::new(self.convert_pattern(left)),
                    Box::new(self.convert_pattern(right)),
                )
            }
            ast::Pattern::Guard(pattern, _guard) => {
                // Guard 模式转换为普通模式（guard 在 match case 中单独处理）
                self.convert_pattern(pattern)
            }
        }
    }

    /// 推断表达式类型（简化版本）
    fn infer_expression_type(&self, expr: &ast::Expression) -> HirType {
        match expr {
            ast::Expression::Literal(lit) => match lit {
                Literal::Integer(_) => HirType::Int,
                Literal::Float(_) => HirType::Float,
                Literal::Boolean(_) => HirType::Bool,
                Literal::String(_) => HirType::String,
                Literal::Char(_) => HirType::Char,
                Literal::Null | Literal::Unit => HirType::Unit,
                Literal::None => HirType::Option(Box::new(HirType::Unknown)),
            },
            ast::Expression::Variable(name) => {
                self.variables.get(name).cloned().unwrap_or(HirType::Unknown)
            }
            ast::Expression::Array(items) => {
                if items.is_empty() {
                    HirType::Array(Box::new(HirType::Unknown))
                } else {
                    let inner_type = self.infer_expression_type(&items[0]);
                    HirType::Array(Box::new(inner_type))
                }
            }
            ast::Expression::Binary(op, left, _right) => {
                match op {
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Less |
                    BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual |
                    BinaryOp::And | BinaryOp::Or => HirType::Bool,
                    _ => self.infer_expression_type(left),
                }
            }
            ast::Expression::Unary(op, expr) => {
                match op {
                    UnaryOp::Not => HirType::Bool,
                    _ => self.infer_expression_type(expr),
                }
            }
            ast::Expression::If(_, then_expr, else_expr) => {
                let then_type = self.infer_expression_type(then_expr);
                let else_type = self.infer_expression_type(else_expr);
                if then_type == else_type {
                    then_type
                } else {
                    HirType::Unknown
                }
            }
            _ => HirType::Unknown,
        }
    }
}

/// 将抽象语法树转换为高级中间表示
pub fn ast_to_hir(program: &ast::Program) -> Result<Hir, HirError> {
    let mut converter = HirConverter::new();
    converter.convert_program(program)
}

/// 语法糖消除：将管道操作展开为嵌套的函数调用
///
/// 例如：`x |> f |> g` 展开为 `g(f(x))`
pub fn desugar_pipe(expr: HirExpression) -> HirExpression {
    match expr {
        HirExpression::Pipe(input, functions) => {
            let mut result = *input;
            for func in functions {
                result = HirExpression::Call(Box::new(func), vec![result]);
            }
            result
        }
        _ => expr,
    }
}

/// 对 HIR 表达式递归应用语法糖消除
pub fn desugar_expression(expr: HirExpression) -> HirExpression {
    match expr {
        HirExpression::Pipe(_, _) => desugar_pipe(expr),
        HirExpression::Call(callee, args) => {
            HirExpression::Call(
                Box::new(desugar_expression(*callee)),
                args.into_iter().map(desugar_expression).collect(),
            )
        }
        HirExpression::Binary(op, left, right) => {
            HirExpression::Binary(
                op,
                Box::new(desugar_expression(*left)),
                Box::new(desugar_expression(*right)),
            )
        }
        HirExpression::Unary(op, expr) => {
            HirExpression::Unary(op, Box::new(desugar_expression(*expr)))
        }
        HirExpression::If(cond, then_e, else_e) => {
            HirExpression::If(
                Box::new(desugar_expression(*cond)),
                Box::new(desugar_expression(*then_e)),
                Box::new(desugar_expression(*else_e)),
            )
        }
        HirExpression::Array(items) => {
            HirExpression::Array(items.into_iter().map(desugar_expression).collect())
        }
        HirExpression::Dictionary(entries) => {
            HirExpression::Dictionary(
                entries.into_iter()
                    .map(|(k, v)| (desugar_expression(k), desugar_expression(v)))
                    .collect(),
            )
        }
        HirExpression::Record(name, fields) => {
            HirExpression::Record(
                name,
                fields.into_iter()
                    .map(|(n, e)| (n, desugar_expression(e)))
                    .collect(),
            )
        }
        HirExpression::Range(start, end, inclusive) => {
            HirExpression::Range(
                Box::new(desugar_expression(*start)),
                Box::new(desugar_expression(*end)),
                inclusive,
            )
        }
        HirExpression::Wait(wait_type, exprs) => {
            HirExpression::Wait(wait_type, exprs.into_iter().map(desugar_expression).collect())
        }
        HirExpression::Given(name, expr) => {
            HirExpression::Given(name, Box::new(desugar_expression(*expr)))
        }
        HirExpression::Typed(expr, ty) => {
            HirExpression::Typed(Box::new(desugar_expression(*expr)), ty)
        }
        HirExpression::Assign(target, value) => {
            HirExpression::Assign(
                Box::new(desugar_expression(*target)),
                Box::new(desugar_expression(*value)),
            )
        }
        HirExpression::Member(obj, name) => {
            HirExpression::Member(Box::new(desugar_expression(*obj)), name)
        }
        HirExpression::Lambda(params, body) => {
            HirExpression::Lambda(params, desugar_block(body))
        }
        _ => expr,
    }
}

/// 对 HIR 块应用语法糖消除
pub fn desugar_block(block: HirBlock) -> HirBlock {
    HirBlock {
        statements: block.statements.into_iter().map(desugar_statement).collect(),
    }
}

/// 对 HIR 语句应用语法糖消除
pub fn desugar_statement(stmt: HirStatement) -> HirStatement {
    match stmt {
        HirStatement::Expression(expr) => {
            HirStatement::Expression(desugar_expression(expr))
        }
        HirStatement::Variable(var) => {
            HirStatement::Variable(HirVariableDecl {
                initializer: var.initializer.map(desugar_expression),
                ..var
            })
        }
        HirStatement::Return(expr) => {
            HirStatement::Return(expr.map(desugar_expression))
        }
        HirStatement::If(if_stmt) => {
            HirStatement::If(HirIfStatement {
                condition: desugar_expression(if_stmt.condition),
                then_block: desugar_block(if_stmt.then_block),
                else_block: if_stmt.else_block.map(desugar_block),
            })
        }
        HirStatement::For(for_stmt) => {
            HirStatement::For(HirForStatement {
                iterator: desugar_expression(for_stmt.iterator),
                body: desugar_block(for_stmt.body),
                ..for_stmt
            })
        }
        HirStatement::While(while_stmt) => {
            HirStatement::While(HirWhileStatement {
                condition: desugar_expression(while_stmt.condition),
                body: desugar_block(while_stmt.body),
            })
        }
        HirStatement::Match(match_stmt) => {
            HirStatement::Match(HirMatchStatement {
                expression: desugar_expression(match_stmt.expression),
                cases: match_stmt.cases.into_iter().map(|c| HirMatchCase {
                    body: desugar_block(c.body),
                    guard: c.guard.map(desugar_expression),
                    ..c
                }).collect(),
            })
        }
        HirStatement::Try(try_stmt) => {
            HirStatement::Try(HirTryStatement {
                body: desugar_block(try_stmt.body),
                catch_clauses: try_stmt.catch_clauses.into_iter().map(|cc| HirCatchClause {
                    body: desugar_block(cc.body),
                    ..cc
                }).collect(),
                finally_block: try_stmt.finally_block.map(desugar_block),
            })
        }
        _ => stmt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_to_hir_returns_ok_for_minimal_program() {
        let source = "let x = 1;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");
        assert!(!hir.declarations.is_empty() || !hir.statements.is_empty());
    }

    #[test]
    fn ast_to_hir_returns_ok_for_program_with_function() {
        let source = "function main() { println(\"hi\") }";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");
        assert!(!hir.declarations.is_empty());
    }

    #[test]
    fn hir_error_displays_message() {
        let e = HirError::ConversionError("test message".to_string());
        assert!(e.to_string().contains("转换错误"));
        assert!(e.to_string().contains("test message"));
    }

    #[test]
    fn hir_type_from_ast_converts_correctly() {
        assert_eq!(HirType::from_ast(&Type::Int), HirType::Int);
        assert_eq!(HirType::from_ast(&Type::String), HirType::String);
        assert_eq!(HirType::from_ast(&Type::Bool), HirType::Bool);
    }

    #[test]
    fn hir_converter_converts_variable_decl() {
        let source = "let x: Int = 42;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        // 变量声明可能在 declarations 或 statements 中
        let found = hir.declarations.iter().find_map(|d| {
            if let HirDeclaration::Variable(var_decl) = d {
                Some(var_decl)
            } else {
                None
            }
        }).or_else(|| hir.statements.iter().find_map(|s| {
            if let HirStatement::Variable(var_decl) = s {
                Some(var_decl)
            } else {
                None
            }
        }));

        if let Some(var_decl) = found {
            assert_eq!(var_decl.name, "x");
            assert_eq!(var_decl.ty, HirType::Int);
        } else {
            panic!("Expected Variable declaration or statement");
        }
    }

    #[test]
    fn hir_converter_converts_function_decl() {
        let source = "function add(a: Int, b: Int) -> Int { return a + b; }";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        assert_eq!(hir.declarations.len(), 1);
        if let HirDeclaration::Function(func_decl) = &hir.declarations[0] {
            assert_eq!(func_decl.name, "add");
            assert_eq!(func_decl.parameters.len(), 2);
            assert_eq!(func_decl.return_type, HirType::Int);
        } else {
            panic!("Expected Function declaration");
        }
    }

    #[test]
    fn desugar_pipe_converts_to_nested_calls() {
        // x |> f |> g 应该展开为 g(f(x))
        let input = HirExpression::Variable("x".to_string());
        let functions = vec![
            HirExpression::Variable("f".to_string()),
            HirExpression::Variable("g".to_string()),
        ];
        let pipe = HirExpression::Pipe(Box::new(input), functions);

        let result = desugar_pipe(pipe);

        // 结果应该是 g(f(x))
        match result {
            HirExpression::Call(callee, args) => {
                // 外层调用是 g
                assert!(matches!(&*callee, HirExpression::Variable(ref n) if n == "g"));
                assert_eq!(args.len(), 1);
                // 内层调用是 f(x)
                match &args[0] {
                    HirExpression::Call(inner_callee, inner_args) => {
                        assert!(matches!(&**inner_callee, HirExpression::Variable(ref n) if n == "f"));
                        assert_eq!(inner_args.len(), 1);
                        assert!(matches!(&inner_args[0], HirExpression::Variable(ref n) if n == "x"));
                    }
                    _ => panic!("Expected nested Call"),
                }
            }
            _ => panic!("Expected Call, got {:?}", result),
        }
    }

    #[test]
    fn desugar_expression_handles_nested_pipes() {
        // (x |> f) + (y |> g) 应该展开为 f(x) + g(y)
        let left_pipe = HirExpression::Pipe(
            Box::new(HirExpression::Variable("x".to_string())),
            vec![HirExpression::Variable("f".to_string())],
        );
        let right_pipe = HirExpression::Pipe(
            Box::new(HirExpression::Variable("y".to_string())),
            vec![HirExpression::Variable("g".to_string())],
        );
        let expr = HirExpression::Binary(
            HirBinaryOp::Add,
            Box::new(left_pipe),
            Box::new(right_pipe),
        );

        let result = desugar_expression(expr);

        match result {
            HirExpression::Binary(HirBinaryOp::Add, left, right) => {
                // 左边应该是 f(x)
                match &*left {
                    HirExpression::Call(callee, args) => {
                        assert!(matches!(&**callee, HirExpression::Variable(ref n) if n == "f"));
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected Call on left"),
                }
                // 右边应该是 g(y)
                match &*right {
                    HirExpression::Call(callee, args) => {
                        assert!(matches!(&**callee, HirExpression::Variable(ref n) if n == "g"));
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected Call on right"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }
}
