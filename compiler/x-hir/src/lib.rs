// 高级中间表示库
//
// HIR (High-level Intermediate Representation) 是 X 语言编译器的中间表示层，
// 位于 AST 和代码生成之间。HIR 提供了：
// - 规范化的语法表示（消除语法糖）
// - 类型信息标注
// - 便于优化和代码生成的结构

use std::collections::HashMap;
use x_parser::ast::{self, BinaryOp, ExpressionKind, Literal, StatementKind, Type, UnaryOp};

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
    /// Perceus 所有权信息（由 analyze_ownership 填充）
    pub perceus_info: HirPerceusInfo,
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
    pub constructors: Vec<HirConstructorDecl>,
    pub is_abstract: bool,
    pub is_final: bool,
}

/// 构造函数声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirConstructorDecl {
    pub parameters: Vec<HirParameter>,
    pub body: HirBlock,
}

/// Trait 声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirTraitDecl {
    pub name: String,
    pub extends: Vec<String>,
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
    /// 效果处理：handle expr with { EffectName -> handler, ... }
    Handle(Box<HirExpression>, Vec<(String, HirExpression)>),
    /// 错误传播：expr?
    TryPropagate(Box<HirExpression>),
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
    UnsignedInt,
    Float,
    Bool,
    String,
    Char,
    Unit,
    Never,
    Dynamic,

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
    /// 类型构造器应用：List<Int>, Map<String, Int>
    TypeConstructor(String, Vec<HirType>),

    // 未知类型（推断失败时使用）
    Unknown,
}

impl HirType {
    /// 从 AST Type 转换
    pub fn from_ast(ty: &Type) -> Self {
        match ty {
            Type::Int => HirType::Int,
            Type::UnsignedInt => HirType::UnsignedInt,
            Type::Float => HirType::Float,
            Type::Bool => HirType::Bool,
            Type::String => HirType::String,
            Type::Char => HirType::Char,
            Type::Unit => HirType::Unit,
            Type::Never => HirType::Never,
            Type::Dynamic => HirType::Dynamic,
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
            Type::TypeConstructor(name, type_args) => HirType::TypeConstructor(
                name.clone(),
                type_args.iter().map(HirType::from_ast).collect(),
            ),
            Type::Var(name) => HirType::Generic(name.clone()),
            Type::Dynamic => HirType::Dynamic,
        }
    }
}

// ============================================================================
// 所有权信息（与 Perceus 集成）
// ============================================================================

/// 所有权状态
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HirOwnership {
    /// 拥有所有权（可以移动或消费）
    Owned,
    /// 借用（不可变引用）
    Borrowed,
    /// 可变借用
    BorrowedMut,
    /// 已移动
    Moved,
    /// 已复制（Copy 类型）
    Copied,
    /// 已释放
    Dropped,
}

/// 所有权行为（用于参数和返回值）
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HirOwnershipBehavior {
    /// 消费所有权
    Consume,
    /// 借用
    Borrow,
    /// 可变借用
    BorrowMut,
    /// 复制（Copy 类型）
    Copy,
}

/// 变量所有权信息
#[derive(Debug, PartialEq, Clone)]
pub struct HirOwnershipInfo {
    /// 变量名
    pub name: String,
    /// 所有权状态
    pub ownership: HirOwnership,
    /// 类型信息
    pub ty: HirType,
    /// 是否需要 drop
    pub needs_drop: bool,
}

/// 函数所有权签名
#[derive(Debug, PartialEq, Clone)]
pub struct HirFunctionOwnership {
    /// 函数名
    pub name: String,
    /// 参数所有权行为
    pub param_ownership: Vec<HirOwnershipBehavior>,
    /// 返回值所有权
    pub return_ownership: HirOwnership,
    /// 是否可能 panic（影响 drop 插入）
    pub may_panic: bool,
}

/// HIR 所有权注解（可附加到表达式或变量上）
#[derive(Debug, PartialEq, Clone)]
pub struct HirOwnershipAnnotation {
    /// 所有权信息
    pub ownership: HirOwnership,
    /// 来源变量（如果有）
    pub source_var: Option<String>,
    /// 目标变量（如果有）
    pub target_var: Option<String>,
    /// 是否隐式 dup
    pub implicit_dup: bool,
}

/// Perceus 分析结果（嵌入 HIR）
#[derive(Debug, PartialEq, Clone, Default)]
pub struct HirPerceusInfo {
    /// 变量所有权信息
    pub var_ownership: HashMap<String, HirOwnershipInfo>,
    /// 函数所有权签名
    pub function_signatures: HashMap<String, HirFunctionOwnership>,
    /// 需要 drop 的变量列表
    pub needs_drop: Vec<String>,
    /// 复用机会
    pub reuse_opportunities: Vec<HirReuseOpportunity>,
}

/// 复用机会
#[derive(Debug, PartialEq, Clone)]
pub struct HirReuseOpportunity {
    /// 源变量（将要 drop）
    pub source: String,
    /// 目标变量（将要 alloc）
    pub target: String,
}

impl HirOwnershipInfo {
    /// 创建新的所有权信息
    pub fn new(name: String, ownership: HirOwnership, ty: HirType) -> Self {
        let needs_drop = Self::type_needs_drop(&ty);
        Self {
            name,
            ownership,
            ty,
            needs_drop,
        }
    }

    /// 判断类型是否需要 drop
    fn type_needs_drop(ty: &HirType) -> bool {
        match ty {
            // Copy 类型不需要 drop
            HirType::Int | HirType::UnsignedInt | HirType::Float | HirType::Bool | HirType::Char | HirType::Unit => false,
            HirType::Never => false,
            // 所有权类型需要 drop
            HirType::String => true,
            HirType::Array(_) => true,
            HirType::Dictionary(_, _) => true,
            HirType::Record(_, _) => true,
            HirType::Option(inner) => Self::type_needs_drop(inner),
            HirType::Result(ok, err) => Self::type_needs_drop(ok) || Self::type_needs_drop(err),
            HirType::Tuple(types) => types.iter().any(Self::type_needs_drop),
            HirType::Function(_, _) => false, // 函数指针是 Copy
            HirType::Async(_) => true,
            HirType::Union(_, _) => true,
            HirType::Generic(_) => true, // 保守假设需要 drop
            HirType::TypeParam(_) => true,
            HirType::TypeConstructor(_, type_args) => {
                // 检查类型参数是否需要 drop
                type_args.iter().any(Self::type_needs_drop)
            }
            HirType::Unknown => true, // 保守假设
            HirType::Dynamic => true, // 保守假设
        }
    }
}

impl HirFunctionOwnership {
    /// 创建新的函数所有权签名
    pub fn new(name: String, param_count: usize, return_ownership: HirOwnership) -> Self {
        Self {
            name,
            param_ownership: vec![HirOwnershipBehavior::Consume; param_count],
            return_ownership,
            may_panic: false,
        }
    }

    /// 设置参数所有权行为
    pub fn with_param_ownership(mut self, index: usize, behavior: HirOwnershipBehavior) -> Self {
        if index < self.param_ownership.len() {
            self.param_ownership[index] = behavior;
        }
        self
    }
}

// ============================================================================
// HIR 扩展（带所有权注解的表达式）
// ============================================================================

/// 带所有权注解的表达式
#[derive(Debug, PartialEq, Clone)]
pub struct HirAnnotatedExpression {
    /// 原始表达式
    pub expr: HirExpression,
    /// 所有权注解
    pub ownership: Option<HirOwnershipAnnotation>,
}

/// 带所有权信息的变量声明
#[derive(Debug, PartialEq, Clone)]
pub struct HirAnnotatedVariableDecl {
    /// 原始变量声明
    pub decl: HirVariableDecl,
    /// 所有权信息
    pub ownership: Option<HirOwnershipInfo>,
}

/// 高级中间表示错误
#[derive(thiserror::Error, Debug)]
pub enum HirError {
    #[error("转换错误: {message}")]
    ConversionError {
        message: String,
    },

    #[error("未定义的变量: {name}")]
    UndefinedVariable {
        name: String,
    },

    #[error("未定义的函数: {name}")]
    UndefinedFunction {
        name: String,
    },

    #[error("重复声明: {name}")]
    DuplicateDeclaration {
        name: String,
    },

    #[error("类型错误: {message}")]
    TypeError {
        message: String,
    },

    #[error("未解析的引用: {name}")]
    UnresolvedReference {
        name: String,
    },

    #[error("无效的操作: {message}")]
    InvalidOperation {
        message: String,
    },

    #[error("语义错误: {message}")]
    SemanticError {
        message: String,
    },
}

impl HirError {
    /// 创建转换错误
    pub fn conversion(message: impl Into<String>) -> Self {
        HirError::ConversionError {
            message: message.into(),
        }
    }

    /// 创建未定义变量错误
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        HirError::UndefinedVariable {
            name: name.into(),
        }
    }

    /// 创建未定义函数错误
    pub fn undefined_function(name: impl Into<String>) -> Self {
        HirError::UndefinedFunction {
            name: name.into(),
        }
    }

    /// 创建重复声明错误
    pub fn duplicate_declaration(name: impl Into<String>) -> Self {
        HirError::DuplicateDeclaration {
            name: name.into(),
        }
    }

    /// 创建类型错误
    pub fn type_error(message: impl Into<String>) -> Self {
        HirError::TypeError {
            message: message.into(),
        }
    }
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
            perceus_info: HirPerceusInfo::default(),
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
                let mut fields = Vec::new();
                let mut methods = Vec::new();
                let mut constructors = Vec::new();

                for member in &class_decl.members {
                    match member {
                        ast::ClassMember::Field(field) => {
                            fields.push(self.convert_variable_decl(field)?);
                        }
                        ast::ClassMember::Method(method) => {
                            methods.push(self.convert_function_decl(method)?);
                        }
                        ast::ClassMember::Constructor(ctor) => {
                            constructors.push(self.convert_constructor(ctor)?);
                        }
                    }
                }

                Ok(HirDeclaration::Class(HirClassDecl {
                    name: class_decl.name.clone(),
                    extends: class_decl.extends.clone(),
                    implements: class_decl.implements.clone(),
                    fields,
                    methods,
                    constructors,
                    is_abstract: class_decl.modifiers.is_abstract,
                    is_final: class_decl.modifiers.is_final,
                }))
            }
            ast::Declaration::Trait(trait_decl) => {
                let mut methods = Vec::new();
                for method in &trait_decl.methods {
                    methods.push(self.convert_function_decl(method)?);
                }

                Ok(HirDeclaration::Trait(HirTraitDecl {
                    name: trait_decl.name.clone(),
                    extends: trait_decl.extends.clone(),
                    methods,
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

    /// 转换构造函数
    fn convert_constructor(&mut self, ctor: &ast::ConstructorDecl) -> Result<HirConstructorDecl, HirError> {
        // 保存当前作用域
        let outer_vars = self.variables.clone();

        // 转换参数
        let mut parameters = Vec::new();
        for param in &ctor.parameters {
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

        // 转换构造函数体
        let body = self.convert_block(&ctor.body)?;

        // 恢复作用域
        self.variables = outer_vars;

        Ok(HirConstructorDecl {
            parameters,
            body,
        })
    }

    /// 转换语句
    fn convert_statement(&mut self, stmt: &ast::Statement) -> Result<HirStatement, HirError> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                Ok(HirStatement::Expression(self.convert_expression(expr)?))
            }
            StatementKind::Variable(var_decl) => {
                let hir_decl = self.convert_variable_decl(var_decl)?;
                self.variables.insert(hir_decl.name.clone(), hir_decl.ty.clone());
                Ok(HirStatement::Variable(hir_decl))
            }
            StatementKind::Return(expr_opt) => {
                let hir_expr = if let Some(expr) = expr_opt {
                    Some(self.convert_expression(expr)?)
                } else {
                    None
                };
                Ok(HirStatement::Return(hir_expr))
            }
            StatementKind::If(if_stmt) => {
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
            StatementKind::For(for_stmt) => {
                Ok(HirStatement::For(HirForStatement {
                    pattern: self.convert_pattern(&for_stmt.pattern),
                    iterator: self.convert_expression(&for_stmt.iterator)?,
                    body: self.convert_block(&for_stmt.body)?,
                }))
            }
            StatementKind::While(while_stmt) => {
                Ok(HirStatement::While(HirWhileStatement {
                    condition: self.convert_expression(&while_stmt.condition)?,
                    body: self.convert_block(&while_stmt.body)?,
                }))
            }
            StatementKind::Match(match_stmt) => {
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
            StatementKind::Try(try_stmt) => {
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
            StatementKind::Break => Ok(HirStatement::Break),
            StatementKind::Continue => Ok(HirStatement::Continue),
            StatementKind::DoWhile(dw) => {
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
        match &expr.node {
            ExpressionKind::Literal(lit) => {
                Ok(HirExpression::Literal(self.convert_literal(lit)))
            }
            ExpressionKind::Variable(name) => {
                Ok(HirExpression::Variable(name.clone()))
            }
            ExpressionKind::Member(obj, member) => {
                Ok(HirExpression::Member(
                    Box::new(self.convert_expression(obj)?),
                    member.clone(),
                ))
            }
            ExpressionKind::Call(callee, args) => {
                let mut hir_args = Vec::new();
                for arg in args {
                    hir_args.push(self.convert_expression(arg)?);
                }
                Ok(HirExpression::Call(
                    Box::new(self.convert_expression(callee)?),
                    hir_args,
                ))
            }
            ExpressionKind::Binary(op, left, right) => {
                Ok(HirExpression::Binary(
                    self.convert_binary_op(op),
                    Box::new(self.convert_expression(left)?),
                    Box::new(self.convert_expression(right)?),
                ))
            }
            ExpressionKind::Unary(op, expr) => {
                Ok(HirExpression::Unary(
                    self.convert_unary_op(op),
                    Box::new(self.convert_expression(expr)?),
                ))
            }
            ExpressionKind::Assign(lhs, rhs) => {
                Ok(HirExpression::Assign(
                    Box::new(self.convert_expression(lhs)?),
                    Box::new(self.convert_expression(rhs)?),
                ))
            }
            ExpressionKind::If(cond, then_expr, else_expr) => {
                Ok(HirExpression::If(
                    Box::new(self.convert_expression(cond)?),
                    Box::new(self.convert_expression(then_expr)?),
                    Box::new(self.convert_expression(else_expr)?),
                ))
            }
            ExpressionKind::Lambda(params, body) => {
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
            ExpressionKind::Array(items) => {
                let mut hir_items = Vec::new();
                for item in items {
                    hir_items.push(self.convert_expression(item)?);
                }
                Ok(HirExpression::Array(hir_items))
            }
            ExpressionKind::Dictionary(entries) => {
                let mut hir_entries = Vec::new();
                for (k, v) in entries {
                    hir_entries.push((
                        self.convert_expression(k)?,
                        self.convert_expression(v)?,
                    ));
                }
                Ok(HirExpression::Dictionary(hir_entries))
            }
            ExpressionKind::Record(name, fields) => {
                let mut hir_fields = Vec::new();
                for (field_name, field_expr) in fields {
                    hir_fields.push((field_name.clone(), self.convert_expression(field_expr)?));
                }
                Ok(HirExpression::Record(name.clone(), hir_fields))
            }
            ExpressionKind::Range(start, end, inclusive) => {
                Ok(HirExpression::Range(
                    Box::new(self.convert_expression(start)?),
                    Box::new(self.convert_expression(end)?),
                    *inclusive,
                ))
            }
            ExpressionKind::Pipe(input, functions) => {
                let mut hir_funcs = Vec::new();
                for func in functions {
                    hir_funcs.push(self.convert_expression(func)?);
                }
                Ok(HirExpression::Pipe(
                    Box::new(self.convert_expression(input)?),
                    hir_funcs,
                ))
            }
            ExpressionKind::Wait(wait_type, exprs) => {
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
            ExpressionKind::Needs(effect_name) => {
                Ok(HirExpression::Needs(effect_name.clone()))
            }
            ExpressionKind::Given(effect_name, expr) => {
                Ok(HirExpression::Given(
                    effect_name.clone(),
                    Box::new(self.convert_expression(expr)?),
                ))
            }
            ExpressionKind::Handle(inner, handlers) => {
                let hir_inner = Box::new(self.convert_expression(inner)?);
                let mut hir_handlers = Vec::new();
                for (name, handler) in handlers {
                    hir_handlers.push((name.clone(), self.convert_expression(handler)?));
                }
                Ok(HirExpression::Handle(hir_inner, hir_handlers))
            }
            ExpressionKind::TryPropagate(inner_expr) => {
                Ok(HirExpression::TryPropagate(Box::new(self.convert_expression(inner_expr)?)))
            }
            ExpressionKind::Parenthesized(inner) => {
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
        match &expr.node {
            ExpressionKind::Literal(lit) => match lit {
                Literal::Integer(_) => HirType::Int,
                Literal::Float(_) => HirType::Float,
                Literal::Boolean(_) => HirType::Bool,
                Literal::String(_) => HirType::String,
                Literal::Char(_) => HirType::Char,
                Literal::Null | Literal::Unit => HirType::Unit,
                Literal::None => HirType::Option(Box::new(HirType::Unknown)),
            },
            ExpressionKind::Variable(name) => {
                self.variables.get(name).cloned().unwrap_or(HirType::Unknown)
            }
            ExpressionKind::Array(items) => {
                if items.is_empty() {
                    HirType::Array(Box::new(HirType::Unknown))
                } else {
                    let inner_type = self.infer_expression_type(&items[0]);
                    HirType::Array(Box::new(inner_type))
                }
            }
            ExpressionKind::Binary(op, left, _right) => {
                match op {
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Less |
                    BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual |
                    BinaryOp::And | BinaryOp::Or => HirType::Bool,
                    _ => self.infer_expression_type(left),
                }
            }
            ExpressionKind::Unary(op, expr) => {
                match op {
                    UnaryOp::Not => HirType::Bool,
                    _ => self.infer_expression_type(expr),
                }
            }
            ExpressionKind::If(_, then_expr, else_expr) => {
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
        HirExpression::Handle(inner, handlers) => {
            let desugared_inner = Box::new(desugar_expression(*inner));
            let desugared_handlers: Vec<(String, HirExpression)> = handlers
                .into_iter()
                .map(|(name, handler)| (name, desugar_expression(handler)))
                .collect();
            HirExpression::Handle(desugared_inner, desugared_handlers)
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

// ============================================================================
// 语义分析
// ============================================================================

/// 语义分析结果
#[derive(Debug, Clone)]
pub struct SemanticAnalysisResult {
    /// 已定义的变量
    pub variables: HashMap<String, HirType>,
    /// 已定义的函数
    pub functions: HashMap<String, HirFunctionInfo>,
    /// 未解析的变量引用
    pub unresolved_variables: Vec<String>,
    /// 未解析的函数引用
    pub unresolved_functions: Vec<String>,
    /// 作用域层级
    pub scope_depth: usize,
}

/// 对 HIR 进行语义分析
pub fn analyze_semantics(hir: &Hir) -> SemanticAnalysisResult {
    let mut result = SemanticAnalysisResult {
        variables: HashMap::new(),
        functions: HashMap::new(),
        unresolved_variables: Vec::new(),
        unresolved_functions: Vec::new(),
        scope_depth: 0,
    };

    // 收集所有声明
    for decl in &hir.declarations {
        analyze_declaration(decl, &mut result);
    }

    // 分析顶层语句
    for stmt in &hir.statements {
        analyze_statement(stmt, &mut result);
    }

    result
}

/// 分析声明
fn analyze_declaration(decl: &HirDeclaration, result: &mut SemanticAnalysisResult) {
    match decl {
        HirDeclaration::Variable(var) => {
            result.variables.insert(var.name.clone(), var.ty.clone());
            if let Some(init) = &var.initializer {
                analyze_expression(init, result);
            }
        }
        HirDeclaration::Function(func) => {
            result.functions.insert(func.name.clone(), HirFunctionInfo {
                name: func.name.clone(),
                parameters: func.parameters.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                return_type: func.return_type.clone(),
                is_async: func.is_async,
            });
            // 分析函数体
            result.scope_depth += 1;
            // 添加参数到作用域
            for param in &func.parameters {
                result.variables.insert(param.name.clone(), param.ty.clone());
            }
            analyze_block(&func.body, result);
            result.scope_depth -= 1;
        }
        HirDeclaration::Class(class) => {
            // 添加类字段
            for field in &class.fields {
                result.variables.insert(format!("{}.{}", class.name, field.name), field.ty.clone());
            }
            // 分析方法
            for method in &class.methods {
                result.functions.insert(method.name.clone(), HirFunctionInfo {
                    name: method.name.clone(),
                    parameters: method.parameters.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                    return_type: method.return_type.clone(),
                    is_async: method.is_async,
                });
            }
        }
        HirDeclaration::Trait(trait_decl) => {
            // 添加 trait 方法签名
            for method in &trait_decl.methods {
                result.functions.insert(method.name.clone(), HirFunctionInfo {
                    name: method.name.clone(),
                    parameters: method.parameters.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                    return_type: method.return_type.clone(),
                    is_async: method.is_async,
                });
            }
        }
        HirDeclaration::TypeAlias(alias) => {
            result.variables.insert(alias.name.clone(), alias.ty.clone());
        }
        _ => {}
    }
}

/// 分析语句
fn analyze_statement(stmt: &HirStatement, result: &mut SemanticAnalysisResult) {
    match stmt {
        HirStatement::Expression(expr) => {
            analyze_expression(expr, result);
        }
        HirStatement::Variable(var) => {
            result.variables.insert(var.name.clone(), var.ty.clone());
            if let Some(init) = &var.initializer {
                analyze_expression(init, result);
            }
        }
        HirStatement::Return(expr) => {
            if let Some(expr) = expr {
                analyze_expression(expr, result);
            }
        }
        HirStatement::If(if_stmt) => {
            analyze_expression(&if_stmt.condition, result);
            analyze_block(&if_stmt.then_block, result);
            if let Some(else_block) = &if_stmt.else_block {
                analyze_block(else_block, result);
            }
        }
        HirStatement::While(while_stmt) => {
            analyze_expression(&while_stmt.condition, result);
            analyze_block(&while_stmt.body, result);
        }
        HirStatement::For(for_stmt) => {
            analyze_expression(&for_stmt.iterator, result);
            analyze_block(&for_stmt.body, result);
        }
        HirStatement::Match(match_stmt) => {
            analyze_expression(&match_stmt.expression, result);
            for case in &match_stmt.cases {
                analyze_block(&case.body, result);
            }
        }
        HirStatement::Try(try_stmt) => {
            analyze_block(&try_stmt.body, result);
            for cc in &try_stmt.catch_clauses {
                analyze_block(&cc.body, result);
            }
            if let Some(finally) = &try_stmt.finally_block {
                analyze_block(finally, result);
            }
        }
        _ => {}
    }
}

/// 分析表达式
fn analyze_expression(expr: &HirExpression, result: &mut SemanticAnalysisResult) {
    match expr {
        HirExpression::Variable(name) => {
            // 检查变量是否已定义
            if !result.variables.contains_key(name) && !result.functions.contains_key(name) {
                result.unresolved_variables.push(name.clone());
            }
        }
        HirExpression::Binary(_, left, right) => {
            analyze_expression(left, result);
            analyze_expression(right, result);
        }
        HirExpression::Unary(_, expr) => {
            analyze_expression(expr, result);
        }
        HirExpression::Call(callee, args) => {
            // 检查函数是否已定义
            if let HirExpression::Variable(name) = callee.as_ref() {
                if !result.functions.contains_key(name) && !result.variables.contains_key(name) {
                    result.unresolved_functions.push(name.clone());
                }
            } else {
                analyze_expression(callee, result);
            }
            for arg in args {
                analyze_expression(arg, result);
            }
        }
        HirExpression::Member(obj, _member) => {
            analyze_expression(obj, result);
        }
        HirExpression::Array(elements) => {
            for elem in elements {
                analyze_expression(elem, result);
            }
        }
        HirExpression::If(cond, then_expr, else_expr) => {
            analyze_expression(cond, result);
            analyze_expression(then_expr, result);
            analyze_expression(else_expr, result);
        }
        HirExpression::Lambda(_params, body) => {
            analyze_block(body, result);
        }
        HirExpression::Pipe(input, funcs) => {
            analyze_expression(input, result);
            for func in funcs {
                analyze_expression(func, result);
            }
        }
        HirExpression::Record(_name, fields) => {
            for (_, expr) in fields {
                analyze_expression(expr, result);
            }
        }
        HirExpression::Range(start, end, _) => {
            analyze_expression(start, result);
            analyze_expression(end, result);
        }
        HirExpression::Dictionary(entries) => {
            for (k, v) in entries {
                analyze_expression(k, result);
                analyze_expression(v, result);
            }
        }
        HirExpression::Wait(_, exprs) => {
            for expr in exprs {
                analyze_expression(expr, result);
            }
        }
        HirExpression::Given(_, expr) => {
            analyze_expression(expr, result);
        }
        HirExpression::Handle(inner, handlers) => {
            analyze_expression(inner, result);
            for (_, handler) in handlers {
                analyze_expression(handler, result);
            }
        }
        HirExpression::Assign(target, value) => {
            analyze_expression(target, result);
            analyze_expression(value, result);
        }
        HirExpression::Typed(inner, _) => {
            analyze_expression(inner, result);
        }
        _ => {}
    }
}

/// 分析块
fn analyze_block(block: &HirBlock, result: &mut SemanticAnalysisResult) {
    result.scope_depth += 1;
    for stmt in &block.statements {
        analyze_statement(stmt, result);
    }
    result.scope_depth -= 1;
}

// ============================================================================
// 优化 Pass
// ============================================================================

/// 优化配置
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// 是否启用常量折叠
    pub constant_folding: bool,
    /// 是否启用死代码消除
    pub dead_code_elimination: bool,
    /// 是否启用函数内联
    pub inline_functions: bool,
    /// 内联函数的最大大小（语句数）
    pub inline_max_size: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            inline_functions: false, // 默认关闭，需要更多上下文信息
            inline_max_size: 10,
        }
    }
}

/// 对 HIR 应用优化
pub fn optimize_hir(mut hir: Hir, config: &OptimizationConfig) -> Hir {
    if config.constant_folding {
        hir = constant_fold_hir(hir);
    }
    if config.dead_code_elimination {
        hir = dead_code_eliminate_hir(hir);
    }
    hir
}

// ============================================================================
// 常量折叠
// ============================================================================

/// 对整个 HIR 应用常量折叠
pub fn constant_fold_hir(mut hir: Hir) -> Hir {
    hir.declarations = hir.declarations.into_iter().map(constant_fold_declaration).collect();
    hir.statements = hir.statements.into_iter().map(constant_fold_statement).collect();
    hir
}

/// 对声明应用常量折叠
fn constant_fold_declaration(decl: HirDeclaration) -> HirDeclaration {
    match decl {
        HirDeclaration::Variable(mut var) => {
            if let Some(init) = var.initializer {
                var.initializer = Some(constant_fold_expression(init));
            }
            HirDeclaration::Variable(var)
        }
        HirDeclaration::Function(mut func) => {
            func.body = constant_fold_block(func.body);
            HirDeclaration::Function(func)
        }
        HirDeclaration::Class(mut class) => {
            class.fields = class.fields.into_iter().map(|mut f| {
                if let Some(init) = f.initializer {
                    f.initializer = Some(constant_fold_expression(init));
                }
                f
            }).collect();
            class.methods = class.methods.into_iter().map(|mut m| {
                m.body = constant_fold_block(m.body);
                m
            }).collect();
            HirDeclaration::Class(class)
        }
        HirDeclaration::Trait(mut trait_decl) => {
            trait_decl.methods = trait_decl.methods.into_iter().map(|mut m| {
                m.body = constant_fold_block(m.body);
                m
            }).collect();
            HirDeclaration::Trait(trait_decl)
        }
        other => other,
    }
}

/// 对语句应用常量折叠
fn constant_fold_statement(stmt: HirStatement) -> HirStatement {
    match stmt {
        HirStatement::Expression(expr) => {
            HirStatement::Expression(constant_fold_expression(expr))
        }
        HirStatement::Variable(mut var) => {
            if let Some(init) = var.initializer {
                var.initializer = Some(constant_fold_expression(init));
            }
            HirStatement::Variable(var)
        }
        HirStatement::Return(expr) => {
            HirStatement::Return(expr.map(constant_fold_expression))
        }
        HirStatement::If(mut if_stmt) => {
            if_stmt.condition = constant_fold_expression(if_stmt.condition);
            if_stmt.then_block = constant_fold_block(if_stmt.then_block);
            if_stmt.else_block = if_stmt.else_block.map(constant_fold_block);

            // 如果条件是常量，可以进行简化
            if let HirExpression::Literal(HirLiteral::Boolean(true)) = if_stmt.condition {
                // 条件为 true，只保留 then 块
                return HirStatement::Expression(HirExpression::Literal(HirLiteral::Unit));
            } else if let HirExpression::Literal(HirLiteral::Boolean(false)) = if_stmt.condition {
                // 条件为 false，只保留 else 块（如果有）
                if let Some(else_block) = if_stmt.else_block {
                    // 将 else 块转换为表达式序列
                    if else_block.statements.len() == 1 {
                        return else_block.statements.into_iter().next().unwrap();
                    }
                }
                return HirStatement::Expression(HirExpression::Literal(HirLiteral::Unit));
            }

            HirStatement::If(if_stmt)
        }
        HirStatement::For(mut for_stmt) => {
            for_stmt.iterator = constant_fold_expression(for_stmt.iterator);
            for_stmt.body = constant_fold_block(for_stmt.body);
            HirStatement::For(for_stmt)
        }
        HirStatement::While(mut while_stmt) => {
            while_stmt.condition = constant_fold_expression(while_stmt.condition);
            while_stmt.body = constant_fold_block(while_stmt.body);

            // 如果条件为 false，整个循环可以删除
            if let HirExpression::Literal(HirLiteral::Boolean(false)) = while_stmt.condition {
                return HirStatement::Expression(HirExpression::Literal(HirLiteral::Unit));
            }

            HirStatement::While(while_stmt)
        }
        HirStatement::Match(mut match_stmt) => {
            match_stmt.expression = constant_fold_expression(match_stmt.expression);
            match_stmt.cases = match_stmt.cases.into_iter().map(|mut c| {
                c.body = constant_fold_block(c.body);
                c.guard = c.guard.map(constant_fold_expression);
                c
            }).collect();
            HirStatement::Match(match_stmt)
        }
        HirStatement::Try(mut try_stmt) => {
            try_stmt.body = constant_fold_block(try_stmt.body);
            try_stmt.catch_clauses = try_stmt.catch_clauses.into_iter().map(|mut cc| {
                cc.body = constant_fold_block(cc.body);
                cc
            }).collect();
            try_stmt.finally_block = try_stmt.finally_block.map(constant_fold_block);
            HirStatement::Try(try_stmt)
        }
        other => other,
    }
}

/// 对块应用常量折叠
fn constant_fold_block(block: HirBlock) -> HirBlock {
    HirBlock {
        statements: block.statements.into_iter().map(constant_fold_statement).collect(),
    }
}

/// 对表达式应用常量折叠
pub fn constant_fold_expression(expr: HirExpression) -> HirExpression {
    match expr {
        // 二元运算常量折叠
        HirExpression::Binary(op, left, right) => {
            let left = constant_fold_expression(*left);
            let right = constant_fold_expression(*right);

            // 尝试折叠二元运算
            if let (HirExpression::Literal(l), HirExpression::Literal(r)) = (&left, &right) {
                if let Some(result) = eval_binary_op(&op, l, r) {
                    return HirExpression::Literal(result);
                }
            }

            HirExpression::Binary(op, Box::new(left), Box::new(right))
        }

        // 一元运算常量折叠
        HirExpression::Unary(op, operand) => {
            let operand = constant_fold_expression(*operand);

            if let HirExpression::Literal(lit) = &operand {
                if let Some(result) = eval_unary_op(&op, lit) {
                    return HirExpression::Literal(result);
                }
            }

            HirExpression::Unary(op, Box::new(operand))
        }

        // If 表达式常量折叠
        HirExpression::If(cond, then_expr, else_expr) => {
            let cond = constant_fold_expression(*cond);
            let then_expr = constant_fold_expression(*then_expr);
            let else_expr = constant_fold_expression(*else_expr);

            // 如果条件是常量，直接选择分支
            match &cond {
                HirExpression::Literal(HirLiteral::Boolean(true)) => return then_expr,
                HirExpression::Literal(HirLiteral::Boolean(false)) => return else_expr,
                _ => {}
            }

            HirExpression::If(Box::new(cond), Box::new(then_expr), Box::new(else_expr))
        }

        // 其他表达式递归处理
        HirExpression::Call(callee, args) => {
            HirExpression::Call(
                Box::new(constant_fold_expression(*callee)),
                args.into_iter().map(constant_fold_expression).collect(),
            )
        }
        HirExpression::Member(obj, name) => {
            HirExpression::Member(Box::new(constant_fold_expression(*obj)), name)
        }
        HirExpression::Assign(target, value) => {
            HirExpression::Assign(
                Box::new(constant_fold_expression(*target)),
                Box::new(constant_fold_expression(*value)),
            )
        }
        HirExpression::Array(elements) => {
            HirExpression::Array(elements.into_iter().map(constant_fold_expression).collect())
        }
        HirExpression::Dictionary(entries) => {
            HirExpression::Dictionary(
                entries.into_iter().map(|(k, v)| {
                    (constant_fold_expression(k), constant_fold_expression(v))
                }).collect(),
            )
        }
        HirExpression::Record(name, fields) => {
            HirExpression::Record(
                name,
                fields.into_iter().map(|(n, v)| (n, constant_fold_expression(v))).collect(),
            )
        }
        HirExpression::Range(start, end, inclusive) => {
            HirExpression::Range(
                Box::new(constant_fold_expression(*start)),
                Box::new(constant_fold_expression(*end)),
                inclusive,
            )
        }
        HirExpression::Pipe(input, funcs) => {
            HirExpression::Pipe(
                Box::new(constant_fold_expression(*input)),
                funcs.into_iter().map(constant_fold_expression).collect(),
            )
        }
        HirExpression::Wait(wait_type, exprs) => {
            HirExpression::Wait(wait_type, exprs.into_iter().map(constant_fold_expression).collect())
        }
        HirExpression::Given(effect, expr) => {
            HirExpression::Given(effect, Box::new(constant_fold_expression(*expr)))
        }
        HirExpression::Handle(inner, handlers) => {
            let folded_inner = Box::new(constant_fold_expression(*inner));
            let folded_handlers: Vec<(String, HirExpression)> = handlers
                .into_iter()
                .map(|(name, handler)| (name, constant_fold_expression(handler)))
                .collect();
            HirExpression::Handle(folded_inner, folded_handlers)
        }
        HirExpression::Typed(expr, ty) => {
            HirExpression::Typed(Box::new(constant_fold_expression(*expr)), ty)
        }
        HirExpression::Lambda(params, body) => {
            HirExpression::Lambda(params, constant_fold_block(body))
        }
        other => other,
    }
}

/// 计算二元运算的结果
fn eval_binary_op(op: &HirBinaryOp, left: &HirLiteral, right: &HirLiteral) -> Option<HirLiteral> {
    match (op, left, right) {
        // 整数运算
        (HirBinaryOp::Add, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Integer(a.checked_add(*b)?))
        }
        (HirBinaryOp::Sub, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Integer(a.checked_sub(*b)?))
        }
        (HirBinaryOp::Mul, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Integer(a.checked_mul(*b)?))
        }
        (HirBinaryOp::Div, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            if *b != 0 {
                Some(HirLiteral::Integer(a.checked_div(*b)?))
            } else {
                None // 除以零不折叠
            }
        }
        (HirBinaryOp::Mod, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            if *b != 0 {
                Some(HirLiteral::Integer(a.checked_rem_euclid(*b)?))
            } else {
                None
            }
        }

        // 浮点运算
        (HirBinaryOp::Add, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(a + b))
        }
        (HirBinaryOp::Sub, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(a - b))
        }
        (HirBinaryOp::Mul, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(a * b))
        }
        (HirBinaryOp::Div, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(a / b))
        }
        (HirBinaryOp::Mod, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(a % b))
        }

        // 混合运算（整数和浮点）
        (HirBinaryOp::Add, HirLiteral::Integer(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(*a as f64 + b))
        }
        (HirBinaryOp::Add, HirLiteral::Float(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Float(a + *b as f64))
        }
        (HirBinaryOp::Sub, HirLiteral::Integer(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(*a as f64 - b))
        }
        (HirBinaryOp::Sub, HirLiteral::Float(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Float(a - *b as f64))
        }
        (HirBinaryOp::Mul, HirLiteral::Integer(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(*a as f64 * b))
        }
        (HirBinaryOp::Mul, HirLiteral::Float(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Float(a * *b as f64))
        }
        (HirBinaryOp::Div, HirLiteral::Integer(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Float(*a as f64 / b))
        }
        (HirBinaryOp::Div, HirLiteral::Float(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Float(a / *b as f64))
        }

        // 比较运算
        (HirBinaryOp::Equal, a, b) => {
            Some(HirLiteral::Boolean(a == b))
        }
        (HirBinaryOp::NotEqual, a, b) => {
            Some(HirLiteral::Boolean(a != b))
        }
        (HirBinaryOp::Less, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Boolean(a < b))
        }
        (HirBinaryOp::LessEqual, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Boolean(a <= b))
        }
        (HirBinaryOp::Greater, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Boolean(a > b))
        }
        (HirBinaryOp::GreaterEqual, HirLiteral::Integer(a), HirLiteral::Integer(b)) => {
            Some(HirLiteral::Boolean(a >= b))
        }
        (HirBinaryOp::Less, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Boolean(a < b))
        }
        (HirBinaryOp::LessEqual, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Boolean(a <= b))
        }
        (HirBinaryOp::Greater, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Boolean(a > b))
        }
        (HirBinaryOp::GreaterEqual, HirLiteral::Float(a), HirLiteral::Float(b)) => {
            Some(HirLiteral::Boolean(a >= b))
        }

        // 逻辑运算
        (HirBinaryOp::And, HirLiteral::Boolean(a), HirLiteral::Boolean(b)) => {
            Some(HirLiteral::Boolean(*a && *b))
        }
        (HirBinaryOp::Or, HirLiteral::Boolean(a), HirLiteral::Boolean(b)) => {
            Some(HirLiteral::Boolean(*a || *b))
        }

        // 字符串拼接
        (HirBinaryOp::Concat, HirLiteral::String(a), HirLiteral::String(b)) => {
            Some(HirLiteral::String(format!("{}{}", a, b)))
        }
        (HirBinaryOp::Add, HirLiteral::String(a), HirLiteral::String(b)) => {
            Some(HirLiteral::String(format!("{}{}", a, b)))
        }

        _ => None,
    }
}

/// 计算一元运算的结果
fn eval_unary_op(op: &HirUnaryOp, operand: &HirLiteral) -> Option<HirLiteral> {
    match (op, operand) {
        (HirUnaryOp::Negate, HirLiteral::Integer(n)) => {
            Some(HirLiteral::Integer(n.checked_neg()?))
        }
        (HirUnaryOp::Negate, HirLiteral::Float(f)) => {
            Some(HirLiteral::Float(-f))
        }
        (HirUnaryOp::Not, HirLiteral::Boolean(b)) => {
            Some(HirLiteral::Boolean(!b))
        }
        _ => None,
    }
}

// ============================================================================
// 死代码消除
// ============================================================================

/// 对整个 HIR 应用死代码消除
pub fn dead_code_eliminate_hir(mut hir: Hir) -> Hir {
    hir.declarations = hir.declarations.into_iter().filter_map(dead_code_eliminate_declaration).collect();
    hir.statements = dead_code_eliminate_statements(hir.statements);
    hir
}

/// 对声明应用死代码消除
fn dead_code_eliminate_declaration(decl: HirDeclaration) -> Option<HirDeclaration> {
    match decl {
        HirDeclaration::Variable(var) => {
            // 保留变量声明（即使没有初始化器，可能用于后续赋值）
            Some(HirDeclaration::Variable(var))
        }
        HirDeclaration::Function(mut func) => {
            func.body = dead_code_eliminate_block(func.body);
            Some(HirDeclaration::Function(func))
        }
        HirDeclaration::Class(mut class) => {
            class.methods = class.methods.into_iter().filter_map(|mut m| {
                m.body = dead_code_eliminate_block(m.body);
                Some(m)
            }).collect();
            Some(HirDeclaration::Class(class))
        }
        HirDeclaration::Trait(mut trait_decl) => {
            trait_decl.methods = trait_decl.methods.into_iter().filter_map(|mut m| {
                m.body = dead_code_eliminate_block(m.body);
                Some(m)
            }).collect();
            Some(HirDeclaration::Trait(trait_decl))
        }
        other => Some(other),
    }
}

/// 对语句列表应用死代码消除
fn dead_code_eliminate_statements(statements: Vec<HirStatement>) -> Vec<HirStatement> {
    let mut result = Vec::new();
    let mut unreachable = false;

    for stmt in statements {
        if unreachable {
            // 已经遇到 return/break/continue，跳过后续语句
            continue;
        }

        let optimized = dead_code_eliminate_statement(stmt);

        // 检查是否是终止语句
        if matches!(optimized, HirStatement::Return(_) | HirStatement::Break | HirStatement::Continue) {
            unreachable = true;
        }

        result.push(optimized);
    }

    result
}

/// 对单个语句应用死代码消除
fn dead_code_eliminate_statement(stmt: HirStatement) -> HirStatement {
    match stmt {
        HirStatement::Expression(expr) => {
            let expr = dead_code_eliminate_expression(expr);
            // 如果表达式是纯字面量且没有副作用，可以删除
            // 但为了简单起见，保留所有表达式语句
            HirStatement::Expression(expr)
        }
        HirStatement::Variable(mut var) => {
            if let Some(init) = var.initializer {
                var.initializer = Some(dead_code_eliminate_expression(init));
            }
            HirStatement::Variable(var)
        }
        HirStatement::If(mut if_stmt) => {
            if_stmt.then_block = dead_code_eliminate_block(if_stmt.then_block);
            if_stmt.else_block = if_stmt.else_block.map(dead_code_eliminate_block);
            HirStatement::If(if_stmt)
        }
        HirStatement::For(mut for_stmt) => {
            for_stmt.iterator = dead_code_eliminate_expression(for_stmt.iterator);
            for_stmt.body = dead_code_eliminate_block(for_stmt.body);
            HirStatement::For(for_stmt)
        }
        HirStatement::While(mut while_stmt) => {
            while_stmt.condition = dead_code_eliminate_expression(while_stmt.condition);
            while_stmt.body = dead_code_eliminate_block(while_stmt.body);
            HirStatement::While(while_stmt)
        }
        HirStatement::Match(mut match_stmt) => {
            match_stmt.expression = dead_code_eliminate_expression(match_stmt.expression);
            match_stmt.cases = match_stmt.cases.into_iter().map(|mut c| {
                c.body = dead_code_eliminate_block(c.body);
                c.guard = c.guard.map(dead_code_eliminate_expression);
                c
            }).collect();
            HirStatement::Match(match_stmt)
        }
        HirStatement::Try(mut try_stmt) => {
            try_stmt.body = dead_code_eliminate_block(try_stmt.body);
            try_stmt.catch_clauses = try_stmt.catch_clauses.into_iter().map(|mut cc| {
                cc.body = dead_code_eliminate_block(cc.body);
                cc
            }).collect();
            try_stmt.finally_block = try_stmt.finally_block.map(dead_code_eliminate_block);
            HirStatement::Try(try_stmt)
        }
        HirStatement::Return(expr) => {
            HirStatement::Return(expr.map(dead_code_eliminate_expression))
        }
        other => other,
    }
}

/// 对表达式应用死代码消除
fn dead_code_eliminate_expression(expr: HirExpression) -> HirExpression {
    match expr {
        HirExpression::Binary(op, left, right) => {
            HirExpression::Binary(op,
                Box::new(dead_code_eliminate_expression(*left)),
                Box::new(dead_code_eliminate_expression(*right)),
            )
        }
        HirExpression::Unary(op, operand) => {
            HirExpression::Unary(op, Box::new(dead_code_eliminate_expression(*operand)))
        }
        HirExpression::Call(callee, args) => {
            HirExpression::Call(
                Box::new(dead_code_eliminate_expression(*callee)),
                args.into_iter().map(dead_code_eliminate_expression).collect(),
            )
        }
        HirExpression::Member(obj, name) => {
            HirExpression::Member(Box::new(dead_code_eliminate_expression(*obj)), name)
        }
        HirExpression::Assign(target, value) => {
            HirExpression::Assign(
                Box::new(dead_code_eliminate_expression(*target)),
                Box::new(dead_code_eliminate_expression(*value)),
            )
        }
        HirExpression::If(cond, then_expr, else_expr) => {
            HirExpression::If(
                Box::new(dead_code_eliminate_expression(*cond)),
                Box::new(dead_code_eliminate_expression(*then_expr)),
                Box::new(dead_code_eliminate_expression(*else_expr)),
            )
        }
        HirExpression::Array(elements) => {
            HirExpression::Array(elements.into_iter().map(dead_code_eliminate_expression).collect())
        }
        HirExpression::Dictionary(entries) => {
            HirExpression::Dictionary(
                entries.into_iter().map(|(k, v)| {
                    (dead_code_eliminate_expression(k), dead_code_eliminate_expression(v))
                }).collect(),
            )
        }
        HirExpression::Record(name, fields) => {
            HirExpression::Record(
                name,
                fields.into_iter().map(|(n, v)| (n, dead_code_eliminate_expression(v))).collect(),
            )
        }
        HirExpression::Range(start, end, inclusive) => {
            HirExpression::Range(
                Box::new(dead_code_eliminate_expression(*start)),
                Box::new(dead_code_eliminate_expression(*end)),
                inclusive,
            )
        }
        HirExpression::Pipe(input, funcs) => {
            HirExpression::Pipe(
                Box::new(dead_code_eliminate_expression(*input)),
                funcs.into_iter().map(dead_code_eliminate_expression).collect(),
            )
        }
        HirExpression::Lambda(params, body) => {
            HirExpression::Lambda(params, dead_code_eliminate_block(body))
        }
        HirExpression::Typed(expr, ty) => {
            HirExpression::Typed(Box::new(dead_code_eliminate_expression(*expr)), ty)
        }
        HirExpression::Given(effect, expr) => {
            HirExpression::Given(effect, Box::new(dead_code_eliminate_expression(*expr)))
        }
        HirExpression::Handle(inner, handlers) => {
            let eliminated_inner = Box::new(dead_code_eliminate_expression(*inner));
            let eliminated_handlers: Vec<(String, HirExpression)> = handlers
                .into_iter()
                .map(|(name, handler)| (name, dead_code_eliminate_expression(handler)))
                .collect();
            HirExpression::Handle(eliminated_inner, eliminated_handlers)
        }
        HirExpression::Wait(wait_type, exprs) => {
            HirExpression::Wait(wait_type, exprs.into_iter().map(dead_code_eliminate_expression).collect())
        }
        other => other,
    }
}

/// 对块应用死代码消除
fn dead_code_eliminate_block(block: HirBlock) -> HirBlock {
    HirBlock {
        statements: dead_code_eliminate_statements(block.statements),
    }
}

// ============================================================================
// 所有权注解
// ============================================================================
// TODO: annotate_ownership 函数已移至 x-perceus 模块，避免循环依赖
// 该函数用于将 Perceus 分析结果集成到 HIR 中

/// 获取变量的所有权信息
pub fn get_var_ownership<'a>(hir: &'a Hir, var_name: &str) -> Option<&'a HirOwnershipInfo> {
    hir.perceus_info.var_ownership.get(var_name)
}

/// 获取函数的所有权签名
pub fn get_function_ownership<'a>(hir: &'a Hir, func_name: &str) -> Option<&'a HirFunctionOwnership> {
    hir.perceus_info.function_signatures.get(func_name)
}

/// 检查类型是否需要 drop
pub fn type_needs_drop(ty: &HirType) -> bool {
    HirOwnershipInfo::type_needs_drop(ty)
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
        let e = HirError::ConversionError {
            message: "test message".to_string(),
        };
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

    // ========================================================================
    // 常量折叠测试
    // ========================================================================

    #[test]
    fn constant_fold_adds_integers() {
        // 1 + 2 => 3
        let expr = HirExpression::Binary(
            HirBinaryOp::Add,
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(2))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(3)));
    }

    #[test]
    fn constant_fold_multiplies_integers() {
        // 3 * 4 => 12
        let expr = HirExpression::Binary(
            HirBinaryOp::Mul,
            Box::new(HirExpression::Literal(HirLiteral::Integer(3))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(4))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(12)));
    }

    #[test]
    fn constant_fold_divides_integers() {
        // 10 / 2 => 5
        let expr = HirExpression::Binary(
            HirBinaryOp::Div,
            Box::new(HirExpression::Literal(HirLiteral::Integer(10))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(2))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(5)));
    }

    #[test]
    fn constant_fold_does_not_fold_division_by_zero() {
        // 1 / 0 => 不折叠
        let expr = HirExpression::Binary(
            HirBinaryOp::Div,
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(0))),
        );

        let result = constant_fold_expression(expr);
        // 应该保持原样
        match result {
            HirExpression::Binary(HirBinaryOp::Div, _, _) => {}
            other => panic!("Expected Binary Div, got {:?}", other),
        }
    }

    #[test]
    fn constant_fold_negates_integer() {
        // -5 => -5
        let expr = HirExpression::Unary(
            HirUnaryOp::Negate,
            Box::new(HirExpression::Literal(HirLiteral::Integer(5))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(-5)));
    }

    #[test]
    fn constant_fold_not_boolean() {
        // !true => false
        let expr = HirExpression::Unary(
            HirUnaryOp::Not,
            Box::new(HirExpression::Literal(HirLiteral::Boolean(true))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Boolean(false)));
    }

    #[test]
    fn constant_fold_compares_integers() {
        // 1 < 2 => true
        let expr = HirExpression::Binary(
            HirBinaryOp::Less,
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(2))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Boolean(true)));
    }

    #[test]
    fn constant_fold_logical_and() {
        // true && false => false
        let expr = HirExpression::Binary(
            HirBinaryOp::And,
            Box::new(HirExpression::Literal(HirLiteral::Boolean(true))),
            Box::new(HirExpression::Literal(HirLiteral::Boolean(false))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Boolean(false)));
    }

    #[test]
    fn constant_fold_concatenates_strings() {
        // "a" ++ "b" => "ab"
        let expr = HirExpression::Binary(
            HirBinaryOp::Concat,
            Box::new(HirExpression::Literal(HirLiteral::String("a".to_string()))),
            Box::new(HirExpression::Literal(HirLiteral::String("b".to_string()))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::String("ab".to_string())));
    }

    #[test]
    fn constant_fold_nests_properly() {
        // (1 + 2) * (3 + 4) => 21
        let inner_left = HirExpression::Binary(
            HirBinaryOp::Add,
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(2))),
        );
        let inner_right = HirExpression::Binary(
            HirBinaryOp::Add,
            Box::new(HirExpression::Literal(HirLiteral::Integer(3))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(4))),
        );
        let expr = HirExpression::Binary(
            HirBinaryOp::Mul,
            Box::new(inner_left),
            Box::new(inner_right),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(21)));
    }

    #[test]
    fn constant_fold_if_true() {
        // if true then 1 else 2 => 1
        let expr = HirExpression::If(
            Box::new(HirExpression::Literal(HirLiteral::Boolean(true))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(2))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(1)));
    }

    #[test]
    fn constant_fold_if_false() {
        // if false then 1 else 2 => 2
        let expr = HirExpression::If(
            Box::new(HirExpression::Literal(HirLiteral::Boolean(false))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(1))),
            Box::new(HirExpression::Literal(HirLiteral::Integer(2))),
        );

        let result = constant_fold_expression(expr);
        assert_eq!(result, HirExpression::Literal(HirLiteral::Integer(2)));
    }

    // ========================================================================
    // 死代码消除测试
    // ========================================================================

    #[test]
    fn dead_code_eliminate_removes_unreachable_code_after_return() {
        let block = HirBlock {
            statements: vec![
                HirStatement::Return(Some(HirExpression::Literal(HirLiteral::Integer(1)))),
                HirStatement::Expression(HirExpression::Literal(HirLiteral::Integer(2))),
            ],
        };

        let result = dead_code_eliminate_block(block);
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(&result.statements[0], HirStatement::Return(_)));
    }

    #[test]
    fn dead_code_eliminate_removes_unreachable_code_after_break() {
        let block = HirBlock {
            statements: vec![
                HirStatement::Break,
                HirStatement::Expression(HirExpression::Literal(HirLiteral::Integer(42))),
            ],
        };

        let result = dead_code_eliminate_block(block);
        assert_eq!(result.statements.len(), 1);
        assert!(matches!(&result.statements[0], HirStatement::Break));
    }

    #[test]
    fn dead_code_eliminate_keeps_reachable_code() {
        let block = HirBlock {
            statements: vec![
                HirStatement::Expression(HirExpression::Literal(HirLiteral::Integer(1))),
                HirStatement::Expression(HirExpression::Literal(HirLiteral::Integer(2))),
            ],
        };

        let result = dead_code_eliminate_block(block);
        assert_eq!(result.statements.len(), 2);
    }

    // ========================================================================
    // 优化配置测试
    // ========================================================================

    #[test]
    fn optimize_hir_with_default_config() {
        let source = "let x = 1 + 2;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        let optimized = optimize_hir(hir, &OptimizationConfig::default());

        // HIR 应该有效
        assert!(!optimized.declarations.is_empty() || !optimized.statements.is_empty());
    }

    #[test]
    fn optimize_hir_disables_constant_folding() {
        let source = "let x = 1 + 2;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        let config = OptimizationConfig {
            constant_folding: false,
            dead_code_elimination: false,
            inline_functions: false,
            inline_max_size: 10,
        };

        let optimized = optimize_hir(hir, &config);

        // HIR 应该有效（但没有优化）
        assert!(!optimized.declarations.is_empty() || !optimized.statements.is_empty());
    }

    // ========================================================================
    // 语义分析测试
    // ========================================================================

    #[test]
    fn semantic_analysis_detects_defined_variables() {
        let source = "let x: Int = 1; let y = x + 2;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        let result = analyze_semantics(&hir);

        // 变量 x 应该被记录
        assert!(result.variables.contains_key("x"));
        // 不应该有未解析的变量
        assert!(result.unresolved_variables.is_empty());
    }

    #[test]
    fn semantic_analysis_detects_defined_functions() {
        let source = "function add(a: Int, b: Int) -> Int { return a + b; }";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        let result = analyze_semantics(&hir);

        // 函数 add 应该被记录
        assert!(result.functions.contains_key("add"));
    }

    #[test]
    fn semantic_analysis_detects_unresolved_variables() {
        let source = "let y = x;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");

        let result = analyze_semantics(&hir);

        // x 应该被检测为未解析
        assert!(result.unresolved_variables.contains(&"x".to_string()));
    }
}
