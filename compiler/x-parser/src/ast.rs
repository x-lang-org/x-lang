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
            Type::Reference(inner) => write!(f, "&{inner}"),
            Type::MutableReference(inner) => write!(f, "&mut {inner}"),
            Type::Pointer(inner) => write!(f, "*{inner}"),
            Type::ConstPointer(inner) => write!(f, "*const {inner}"),
            Type::MutPointer(inner) => write!(f, "*mut {inner}"),
            Type::Void => write!(f, "void"),
            // C FFI types
            Type::CInt => write!(f, "CInt"),
            Type::CUInt => write!(f, "CUInt"),
            Type::CLong => write!(f, "CLong"),
            Type::CULong => write!(f, "CULong"),
            Type::CLongLong => write!(f, "CLongLong"),
            Type::CULongLong => write!(f, "CULongLong"),
            Type::CFloat => write!(f, "CFloat"),
            Type::CDouble => write!(f, "CDouble"),
            Type::CChar => write!(f, "CChar"),
            Type::CSize => write!(f, "CSize"),
            Type::CString => write!(f, "CString"),
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
    ExternFunction(ExternFunctionDecl),
    Class(ClassDecl),
    Trait(TraitDecl),
    Enum(EnumDecl),
    Record(RecordDecl),
    Effect(EffectDecl),
    TypeAlias(TypeAlias),
    Newtype(Newtype),
    Module(ModuleDecl),
    Import(ImportDecl),
    Export(ExportDecl),
    Implement(ImplementDecl),
}

/// 变量声明
#[derive(Debug, PartialEq, Clone)]
pub struct VariableDecl {
    pub pattern: Pattern,
    pub is_mutable: bool,
    pub is_constant: bool,
    /// 外部变量 ABI（仅 extern variable 声明使用）
    pub extern_abi: Option<String>,
    pub type_annot: Option<Type>,
    pub initializer: Option<Expression>,
    /// 访问修饰符（用于类字段）
    pub visibility: Visibility,
    /// 源码位置
    pub span: Span,
}

impl VariableDecl {
    pub fn simple_name(&self) -> Option<&str> {
        match &self.pattern {
            Pattern::Variable(name) => Some(name.as_str()),
            _ => None,
        }
    }
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

/// 外部函数声明（extern function）
#[derive(Debug, PartialEq, Clone)]
pub struct ExternFunctionDecl {
    /// ABI 名称，如 "C", "zig" 等
    pub abi: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    /// 函数名
    pub name: String,
    /// 参数列表
    pub parameters: Vec<Parameter>,
    /// 返回类型
    pub return_type: Option<Type>,
    /// 是否为可变参数函数（如 printf）
    pub is_variadic: bool,
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

/// trait 实现声明：implement Trait for Type { ... }
#[derive(Debug, PartialEq, Clone)]
pub struct ImplementDecl {
    /// 被实现的 trait 名称
    pub trait_name: String,
    /// 实现 trait 的目标类型
    pub target_type: Type,
    /// 类型参数（泛型实现）
    pub type_parameters: Vec<TypeParameter>,
    /// 类型约束（where 子句）
    pub where_clause: Vec<TypeConstraint>,
    /// 方法实现列表
    pub methods: Vec<FunctionDecl>,
    /// 源码位置
    pub span: Span,
}

/// 记录声明：`record Name { field: Type, ... }`
#[derive(Debug, PartialEq, Clone)]
pub struct RecordDecl {
    pub name: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    /// 记录字段
    pub fields: Vec<(String, Type)>,
    /// 类型约束（where 子句）
    pub where_clause: Vec<TypeConstraint>,
    /// 源码位置
    pub span: Span,
}

/// 效果声明：`effect Name<E> { ... }`
#[derive(Debug, PartialEq, Clone)]
pub struct EffectDecl {
    pub name: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    /// 操作签名：名称 → 输入输出类型
    pub operations: Vec<(String, Option<Type>, Option<Type>)>,
    /// 类型约束（where 子句）
    pub where_clause: Vec<TypeConstraint>,
    /// 源码位置
    pub span: Span,
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

/// 枚举声明
#[derive(Debug, PartialEq, Clone)]
pub struct EnumDecl {
    pub name: String,
    /// 类型参数（泛型）
    pub type_parameters: Vec<TypeParameter>,
    /// 枚举变体
    pub variants: Vec<EnumVariant>,
    /// 源码位置
    pub span: Span,
}

/// 枚举变体
#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    pub name: String,
    /// 变体数据（可以是元组式或记录式）
    pub data: EnumVariantData,
    /// 文档注释
    pub doc: Option<String>,
    /// 源码位置
    pub span: Span,
}

/// 枚举变体数据
#[derive(Debug, PartialEq, Clone)]
pub enum EnumVariantData {
    /// 无数据的变体（如 None）
    Unit,
    /// 元组式变体（如 Some(T)）
    Tuple(Vec<Type>),
    /// 记录式变体（如 Point { x: Int, y: Int }）
    Record(Vec<(String, Type)>),
}

/// 类型别名
#[derive(Debug, PartialEq, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub type_: Type,
    /// 源码位置
    pub span: Span,
}

/// 新类型（不透明包装）
#[derive(Debug, PartialEq, Clone)]
pub struct Newtype {
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
    Function(FunctionDecl),
    Return(Option<Expression>),
    If(IfStatement),
    For(ForStatement),
    While(WhileStatement),
    Match(MatchStatement),
    Try(TryStatement),
    Break,
    Continue,
    DoWhile(DoWhileStatement),
    /// Unsafe 块 - 用于 FFI 调用
    Unsafe(Block),
    /// defer 语句 - 延迟执行直到函数退出
    Defer(Expression),
    /// yield 语句 - 生成器产出值
    Yield(Option<Expression>),
    /// loop 循环 - 无限循环
    Loop(Block),
    /// when guard - early return guard: when condition { expr }
    WhenGuard(Expression, Expression),
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

    // 类型转换: expression as Type
    Cast(Box<Expression>, Type),

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

    // 元组 - 圆括号，至少两个元素
    Tuple(Vec<Expression>),

    // 记录
    Record(String, Vec<(String, Expression)>),

    // 范围
    Range(Box<Expression>, Box<Expression>, bool), // 最后一个参数表示是否包含末尾

    // 管道操作
    Pipe(Box<Expression>, Vec<Box<Expression>>), // 第一个表达式是输入，后面是一系列处理函数

    // Wait操作（异步）
    Wait(WaitType, Vec<Expression>),

    // Await 操作：await async_expr
    Await(Box<Expression>),

    // Effect相关
    Needs(String),
    Given(String, Box<Expression>),
    /// Effect handler: handle expr with { EffectName -> handler_fn, ... }
    Handle(Box<Expression>, Vec<(String, Expression)>),

    /// 错误传播：expr? 用于 Result/Option 的提前返回
    TryPropagate(Box<Expression>),

    /// 可选链：expr?.member 访问
    OptionalChain(Box<Expression>, String),

    /// 空合并：expr ?? default 如果为空则使用默认值
    NullCoalescing(Box<Expression>, Box<Expression>),

    /// 模式匹配表达式（given 表达式）: given value { is pattern => expr ... }
    Match(Box<Expression>, Vec<MatchCase>),

    /// when guard: when condition { expr } - early return guard
    WhenGuard(Box<Expression>, Box<Expression>),

    /// Block expression
    Block(Block),

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
    Dynamic, // 动态类型，用于异构字典等场景

    // 复合类型
    Array(Box<Type>),
    Dictionary(Box<Type>, Box<Type>),
    Record(String, Vec<(String, Box<Type>)>),
    Union(String, Vec<Type>),
    Tuple(Vec<Type>),

    // 高级类型
    Function(Vec<Box<Type>>, Box<Type>),
    Async(Box<Type>),

    // 引用类型
    /// 不可变引用 &T
    Reference(Box<Type>),
    /// 可变引用 &mut T
    MutableReference(Box<Type>),

    // FFI 类型
    /// 原始指针类型 (*T)
    Pointer(Box<Type>),
    /// 常量原始指针类型 (*const T)
    ConstPointer(Box<Type>),
    /// 可变原始指针类型 (*mut T)
    MutPointer(Box<Type>),
    /// void 类型（用于 FFI）
    Void,

    // C FFI 类型 - 平台特定大小
    /// C int 类型
    CInt,
    /// C unsigned int 类型
    CUInt,
    /// C long 类型
    CLong,
    /// C unsigned long 类型
    CULong,
    /// C long long 类型
    CLongLong,
    /// C unsigned long long 类型
    CULongLong,
    /// C float 类型
    CFloat,
    /// C double 类型
    CDouble,
    /// C char 类型
    CChar,
    /// C size_t 类型
    CSize,
    /// C 字符串类型 (char*)
    CString,

    // 泛型类型
    /// 泛型类型名（如 List, Map）
    Generic(String),
    /// 类型参数（如 T, U）
    TypeParam(String),
    /// 类型构造器应用：`List<Int>`、`Map<String, Int>`
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
    /// 枚举构造器模式：TypeName.VariantName(patterns)
    /// 例如：Option.Some(value), Option.None
    EnumConstructor(String, String, Vec<Pattern>),
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
    Reference,
    MutableReference,
}

/// Wait类型（异步操作）
#[derive(Debug, PartialEq, Clone)]
pub enum WaitType {
    Single,
    Concurrently,
    Race,
    Atomic,
    Retry,
    Timeout(Box<Expression>),
}
