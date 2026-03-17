use std::fmt;
use x_lexer::span::Span;

/// 访问修饰符
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Visibility {
    #[default]
    Private,
    Public,
    Protected,
    Internal,
}

/// 方法修饰符
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct MethodModifiers {
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_static: bool,
    pub visibility: Visibility,
}

/// 类修饰符
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ClassModifiers {
    pub is_abstract: bool,
    pub is_final: bool,
}

/// 类型参数（泛型参数）
#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter {
    /// 类型参数名称
    pub name: String,
    /// 类型约束（T: Trait）
    pub constraints: Vec<TypeConstraint>,
    /// 源码位置
    pub span: Span,
}

/// 类型约束（T: Trait）
#[derive(Debug, PartialEq, Clone)]
pub struct TypeConstraint {
    /// Trait 名称
    pub trait_name: String,
    /// 源码位置
    pub span: Span,
}

/// 效果类型
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Effect {
    /// IO 效果
    IO,
    /// 异步效果
    Async,
    /// 状态效果（带状态类型名称）
    State(String),
    /// 异常效果（带错误类型名称）
    Throws(String),
    /// 非确定性效果
    NonDet,
    /// 自定义效果
    Custom(String),
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::IO => write!(f, "IO"),
            Effect::Async => write!(f, "Async"),
            Effect::State(ty) => write!(f, "State<{}>", ty),
            Effect::Throws(ty) => write!(f, "Throws<{}>", ty),
            Effect::NonDet => write!(f, "NonDet"),
            Effect::Custom(name) => write!(f, "{}", name),
        }
    }
}

// 为Type枚举添加to_string方法
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::UnsignedInt => write!(f, "UnsignedInt"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Char => write!(f, "Char"),
            Type::Unit => write!(f, "Unit"),
            Type::Never => write!(f, "Never"),
            Type::Array(inner) => write!(f, "Array<{inner}>"),
            Type::Dictionary(key, value) => write!(f, "Dictionary<{key}, {value}>"),
            Type::Record(name, _) => write!(f, "Record<{name}>"),
            Type::Union(name, _) => write!(f, "Union<{name}>"),
            Type::Tuple(types) => write!(
                f,
                "Tuple<{}>",
                types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Option(inner) => write!(f, "Option<{inner}>"),
            Type::Result(ok, err) => write!(f, "Result<{ok}, {err}>"),
            Type::Function(params, ret) => write!(
                f,
                "Function<{}, {ret}>",
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Async(inner) => write!(f, "Async<{inner}>"),
            Type::Generic(name) => write!(f, "{name}"),
            Type::TypeParam(name) => write!(f, "{name}"),
            Type::TypeConstructor(name, args) => write!(
                f,
                "{}<{}>",
                name,
                args.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Var(name) => write!(f, "{name}"),
            Type::Dynamic => write!(f, "Dynamic"),
        }
    }
}

// 抽象语法树定义

/// 带位置信息的包装器
#[derive(Debug, PartialEq, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// 创建带位置信息的节点
pub fn spanned<T>(node: T, span: Span) -> Spanned<T> {
    Spanned::new(node, span)
}

/// X语言程序的根节点
#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
    /// 源码位置（整个程序）
    pub span: Span,
}

/// 声明类型
#[derive(Debug, PartialEq, Clone)]
pub enum Declaration {
    Variable(VariableDecl),
    Function(FunctionDecl),
    Class(ClassDecl),
    Trait(TraitDecl),
    TypeAlias(TypeAlias),
    Module(ModuleDecl),
    Import(ImportDecl),
    Export(ExportDecl),
}

/// 变量声明
#[derive(Debug, PartialEq, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub is_mutable: bool,
    pub type_annot: Option<Type>,
    pub initializer: Option<Expression>,
    /// 访问修饰符（用于类字段）
    pub visibility: Visibility,
    /// 源码位置
    pub span: Span,
}

/// 函数声明
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    /// 效果注解（with IO, Async 等）
    pub effects: Vec<Effect>,
    pub body: Block,
    pub is_async: bool,
    /// 方法修饰符（用于类方法）
    pub modifiers: MethodModifiers,
    /// 源码位置
    pub span: Span,
}

/// 参数
#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annot: Option<Type>,
    pub default: Option<Expression>,
    /// 源码位置
    pub span: Span,
}

/// 类声明
#[derive(Debug, PartialEq, Clone)]
pub struct ClassDecl {
    pub name: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub members: Vec<ClassMember>,
    /// 类修饰符
    pub modifiers: ClassModifiers,
    /// 源码位置
    pub span: Span,
}

/// 类成员
#[derive(Debug, PartialEq, Clone)]
pub enum ClassMember {
    Field(VariableDecl),
    Method(FunctionDecl),
    Constructor(ConstructorDecl),
}

/// 构造函数声明
#[derive(Debug, PartialEq, Clone)]
pub struct ConstructorDecl {
    pub parameters: Vec<Parameter>,
    pub body: Block,
    /// 访问修饰符
    pub visibility: Visibility,
}

/// 接口声明
#[derive(Debug, PartialEq, Clone)]
pub struct TraitDecl {
    pub name: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    /// 父 trait 列表
    pub extends: Vec<String>,
    pub methods: Vec<FunctionDecl>,
    /// 源码位置
    pub span: Span,
}

/// 类型别名
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub type_: Type,
    /// 源码位置
    pub span: Span,
}

/// 模块声明
#[derive(Debug, PartialEq, Clone)]
pub struct ModuleDecl {
    pub name: String,
    /// 源码位置
    pub span: Span,
}

/// 导入声明
#[derive(Debug, PartialEq, Clone)]
pub struct ImportDecl {
    pub module_path: String,
    pub symbols: Vec<ImportSymbol>,
    /// 源码位置
    pub span: Span,
}

/// 导入符号
#[derive(Debug, PartialEq, Clone)]
pub enum ImportSymbol {
    All,
    Named(String, Option<String>), // (原名, 别名)
}

/// 导出声明
#[derive(Debug, PartialEq, Clone)]
pub struct ExportDecl {
    pub symbol: String,
    /// 源码位置
    pub span: Span,
}

/// 块语句
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// 语句类型（带位置信息）
pub type Statement = Spanned<StatementKind>;

/// 语句种类
#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind {
    Expression(Expression),
    Variable(VariableDecl),
    Return(Option<Expression>),
    If(IfStatement),
    For(ForStatement),
    While(WhileStatement),
    Match(MatchStatement),
    Try(TryStatement),
    Break,
    Continue,
    DoWhile(DoWhileStatement),
}

/// do-while 语句
#[derive(Debug, PartialEq, Clone)]
pub struct DoWhileStatement {
    pub body: Block,
    pub condition: Expression,
}

/// if语句
#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

/// for语句
#[derive(Debug, PartialEq, Clone)]
pub struct ForStatement {
    pub pattern: Pattern,
    pub iterator: Expression,
    pub body: Block,
}

/// while语句
#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

/// match语句（模式匹配）
#[derive(Debug, PartialEq, Clone)]
pub struct MatchStatement {
    pub expression: Expression,
    pub cases: Vec<MatchCase>,
}

/// match case
#[derive(Debug, PartialEq, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Block,
    pub guard: Option<Expression>,
}

/// try语句（异常处理）
#[derive(Debug, PartialEq, Clone)]
pub struct TryStatement {
    pub body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub finally_block: Option<Block>,
}

/// catch子句
#[derive(Debug, PartialEq, Clone)]
pub struct CatchClause {
    pub exception_type: Option<String>,
    pub variable_name: Option<String>,
    pub body: Block,
}

/// 表达式类型（带位置信息）
pub type Expression = Spanned<ExpressionKind>;

/// 表达式种类
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionKind {
    // 字面量
    Literal(Literal),

    // 变量引用
    Variable(String),

    // 成员访问
    Member(Box<Expression>, String),

    // 函数调用
    Call(Box<Expression>, Vec<Expression>),

    // 二元运算
    Binary(BinaryOp, Box<Expression>, Box<Expression>),

    // 一元运算
    Unary(UnaryOp, Box<Expression>),

    // 赋值
    Assign(Box<Expression>, Box<Expression>),

    // 三元条件
    If(Box<Expression>, Box<Expression>, Box<Expression>),

    // lambda函数
    Lambda(Vec<Parameter>, Block),

    // 数组
    Array(Vec<Expression>),

    // 字典
    Dictionary(Vec<(Expression, Expression)>),

    // 记录
    Record(String, Vec<(String, Expression)>),

    // 范围
    Range(Box<Expression>, Box<Expression>, bool), // 最后一个参数表示是否包含末尾

    // 管道操作
    Pipe(Box<Expression>, Vec<Box<Expression>>), // 第一个表达式是输入，后面是一系列处理函数

    // Wait操作（异步）
    Wait(WaitType, Vec<Expression>),

    // Effect相关
    Needs(String),
    Given(String, Box<Expression>),
    /// Effect handler: handle expr with { EffectName -> handler_fn, ... }
    Handle(Box<Expression>, Vec<(String, Expression)>),

    /// 错误传播：expr? 用于 Result/Option 的提前返回
    TryPropagate(Box<Expression>),

    // 其他
    Parenthesized(Box<Expression>),
}

/// 字面量类型
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Null,
    None,
    Unit,
}

/// 类型定义
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // 基本类型
    Int,
    UnsignedInt,
    Float,
    Bool,
    String,
    Char,
    Unit,
    Never,
    Dynamic,  // 动态类型，用于异构字典等场景

    // 复合类型
    Array(Box<Type>),
    Dictionary(Box<Type>, Box<Type>),
    Record(String, Vec<(String, Box<Type>)>),
    Union(String, Vec<Type>),
    Tuple(Vec<Type>),

    // 高级类型
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Function(Vec<Box<Type>>, Box<Type>),
    Async(Box<Type>),

    // 泛型类型
    /// 泛型类型名（如 List, Map）
    Generic(String),
    /// 类型参数（如 T, U）
    TypeParam(String),
    /// 类型构造器应用：List<Int>, Map<String, Int>
    TypeConstructor(String, Vec<Type>),

    // 类型变量
    /// 类型变量（用于类型推断）
    Var(String),
}

/// 模式类型
#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Wildcard,
    Variable(String),
    Literal(Literal),
    Array(Vec<Pattern>),
    Dictionary(Vec<(Pattern, Pattern)>),
    Record(String, Vec<(String, Pattern)>),
    Tuple(Vec<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
    Guard(Box<Pattern>, Box<Expression>),
}

/// 二元运算符
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    // 算术运算
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // 逻辑运算
    And,
    Or,

    // 比较运算
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // 位运算
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,

    // 其他
    Concat,
    RangeExclusive,
    RangeInclusive,
}

/// 一元运算符
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
    BitNot,
    Wait,
}

/// Wait类型（异步操作）
#[derive(Debug, PartialEq, Clone)]
pub enum WaitType {
    Single,
    Together,
    Race,
    Timeout(Box<Expression>),
}
