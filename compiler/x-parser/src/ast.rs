use std::fmt;

// 为Type枚举添加to_string方法
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
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
            Type::Var(name) => write!(f, "{name}"),
        }
    }
}

// 抽象语法树定义

/// X语言程序的根节点
#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
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
}

/// 函数声明
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_async: bool,
}

/// 参数
#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annot: Option<Type>,
    pub default: Option<Expression>,
}

/// 类声明
#[derive(Debug, PartialEq, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub members: Vec<ClassMember>,
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
}

/// 接口声明
#[derive(Debug, PartialEq, Clone)]
pub struct TraitDecl {
    pub name: String,
    pub methods: Vec<FunctionDecl>,
}

/// 类型别名
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub type_: Type,
}

/// 模块声明
#[derive(Debug, PartialEq, Clone)]
pub struct ModuleDecl {
    pub name: String,
}

/// 导入声明
#[derive(Debug, PartialEq, Clone)]
pub struct ImportDecl {
    pub module_path: String,
    pub symbols: Vec<ImportSymbol>,
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
}

/// 块语句
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// 语句类型
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
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

/// 表达式类型
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
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
    Float,
    Bool,
    String,
    Char,
    Unit,
    Never,

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
    Generic(String),
    TypeParam(String),

    // 类型变量
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
