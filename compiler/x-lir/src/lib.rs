// LIR (Low-level Intermediate Representation)
//
// LIR 即 XIR，是所有后端的统一输入。
// 这是一个简化的、类 C 的中间表示，用于降低到各种后端。
//
// 架构位置：HIR → MIR → LIR → 后端

pub mod lower;
pub mod peephole;

use std::fmt::{self, Display};

pub use lower::{lower_mir_to_lir, LirLowerError, LirLowerResult};
pub use peephole::{peephole_optimize_function, peephole_optimize_program, PeepholeOptimizer};

// ============================================================================
// LIR 程序结构
// ============================================================================

/// LIR 程序
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

/// 声明
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    /// 导入声明
    Import(Import),
    /// 函数声明
    Function(Function),
    /// 全局变量
    Global(GlobalVar),
    /// 结构体定义
    Struct(Struct),
    /// 类定义（支持继承和虚方法）
    Class(Class),
    /// 虚表定义
    VTable(VTable),
    /// 枚举定义
    Enum(Enum),
    /// 类型别名
    TypeAlias(TypeAlias),
    /// 新类型（不透明包装）
    Newtype(Newtype),
    /// trait 定义（接口）
    Trait(Trait),
    /// effect 定义（效果接口）
    Effect(Effect),
    /// trait/effect 实现
    Impl(Impl),
    /// 外部函数声明
    ExternFunction(ExternFunction),
}

/// 函数
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    /// 类型参数（泛型）
    pub type_params: Vec<String>,
    pub return_type: Type,
    pub parameters: Vec<Parameter>,
    pub body: Block,
    pub is_static: bool,
    pub is_inline: bool,
}

/// 参数
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_: Type,
}

/// 全局变量
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVar {
    pub name: String,
    pub type_: Type,
    pub initializer: Option<Expression>,
    pub is_static: bool,
}

/// 结构体定义
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

/// 结构体字段
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: Type,
}

/// 类定义（支持继承和虚方法）
#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub fields: Vec<Field>,
    pub vtable_indices: Vec<(String, usize)>,
    pub has_vtable: bool,
}

/// 虚表定义
#[derive(Debug, Clone, PartialEq)]
pub struct VTable {
    pub name: String,
    pub class_name: String,
    pub entries: Vec<VTableEntry>,
}

/// 虚表条目
#[derive(Debug, Clone, PartialEq)]
pub struct VTableEntry {
    pub method_name: String,
    pub function_type: VTableMethodType,
}

/// 虚方法类型
#[derive(Debug, Clone, PartialEq)]
pub struct VTableMethodType {
    pub return_type: Type,
    pub param_types: Vec<Type>,
}

/// 枚举定义
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

/// 枚举变体
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

/// effect 操作
#[derive(Debug, Clone, PartialEq)]
pub struct EffectOperation {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
}

/// effect 定义
#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub name: String,
    pub type_params: Vec<String>,
    pub operations: Vec<EffectOperation>,
}

/// trait 方法
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub type_params: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub default_body: Option<Block>,
}

/// trait 定义
#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: String,
    pub type_params: Vec<String>,
    pub extends: Vec<String>,
    pub methods: Vec<TraitMethod>,
}

/// 实现（trait 或 effect）
#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub type_params: Vec<String>,
    pub target_type: Type,
    pub trait_name: String,
    pub methods: Vec<Function>,
    pub is_effect: bool,
}

/// 类型别名
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub type_: Type,
}

/// 新类型（不透明包装）
#[derive(Debug, Clone, PartialEq)]
pub struct Newtype {
    pub name: String,
    pub type_: Type,
}

/// 外部函数声明
#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    pub name: String,
    /// 类型参数（泛型）
    pub type_params: Vec<String>,
    pub return_type: Type,
    pub parameters: Vec<Type>,
    pub abi: Option<String>,
}

/// 导入声明
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// 模块路径
    pub module_path: String,
    /// 导入的符号列表：(name, alias)
    pub symbols: Vec<(String, Option<String>)>,
    /// 是否导入全部
    pub import_all: bool,
}

// ============================================================================
// 类型系统
// ============================================================================

/// 类型
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Bool,
    Char,
    Schar,
    Uchar,
    Short,
    Ushort,
    Int,
    Uint,
    Long,
    Ulong,
    LongLong,
    UlongLong,
    Float,
    Double,
    LongDouble,
    Size,
    Ptrdiff,
    Intptr,
    Uintptr,
    Pointer(Box<Type>),
    Array(Box<Type>, Option<u64>),
    FunctionPointer(Box<Type>, Vec<Type>),
    Named(String),
    Qualified(Qualifiers, Box<Type>),
}

/// 类型限定符
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Qualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

impl Qualifiers {
    pub fn none() -> Self {
        Self {
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }

    pub fn const_() -> Self {
        Self {
            is_const: true,
            is_volatile: false,
            is_restrict: false,
        }
    }
}

// ============================================================================
// 语句
// ============================================================================

/// 语句块
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// 语句
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Declaration(Declaration),
    Variable(Variable),
    If(IfStatement),
    While(WhileStatement),
    DoWhile(DoWhileStatement),
    For(ForStatement),
    Switch(SwitchStatement),
    Match(MatchStatement),
    Try(TryStatement),
    Break,
    Continue,
    Return(Option<Expression>),
    Goto(String),
    Label(String),
    Empty,
    Compound(Block),
}

/// 变量声明
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub type_: Type,
    pub initializer: Option<Expression>,
    pub is_static: bool,
    pub is_extern: bool,
}

/// if 语句
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

/// while 语句
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
}

/// do-while 语句
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStatement {
    pub body: Box<Statement>,
    pub condition: Expression,
}

/// for 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub initializer: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub increment: Option<Expression>,
    pub body: Box<Statement>,
}

/// switch 语句
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStatement {
    pub expression: Expression,
    pub cases: Vec<SwitchCase>,
    pub default: Option<Box<Statement>>,
}

/// switch case
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub value: Expression,
    pub body: Box<Statement>,
}

/// match 语句（模式匹配）
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement {
    pub scrutinee: Expression,
    pub cases: Vec<MatchCase>,
}

/// match case
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Block,
    pub guard: Option<Expression>,
}

/// 模式
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Variable(String),
    Literal(Literal),
    Constructor(String, Vec<Pattern>),
    Tuple(Vec<Pattern>),
    Record(String, Vec<(String, Pattern)>),
    Or(Box<Pattern>, Box<Pattern>),
}

/// try 语句
#[derive(Debug, Clone, PartialEq)]
pub struct TryStatement {
    pub body: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub finally_block: Option<Block>,
}

/// catch 子句
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub exception_type: Option<String>,
    pub variable_name: Option<String>,
    pub body: Block,
}

// ============================================================================
// 表达式
// ============================================================================

/// 表达式
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Unary(UnaryOp, Box<Expression>),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    AssignOp(BinaryOp, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Member(Box<Expression>, String),
    PointerMember(Box<Expression>, String),
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
    Cast(Type, Box<Expression>),
    SizeOf(Type),
    SizeOfExpr(Box<Expression>),
    AlignOf(Type),
    Comma(Vec<Expression>),
    Parenthesized(Box<Expression>),
    InitializerList(Vec<Initializer>),
    CompoundLiteral(Type, Vec<Initializer>),
}

/// 字面量
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    UnsignedInteger(u64),
    Long(i64),
    UnsignedLong(u64),
    LongLong(i64),
    UnsignedLongLong(u64),
    Float(f64),
    Double(f64),
    Char(char),
    String(String),
    Bool(bool),
    NullPointer,
}

/// 初始化器
#[derive(Debug, Clone, PartialEq)]
pub enum Initializer {
    Expression(Expression),
    List(Vec<Initializer>),
    Named(String, Box<Initializer>),
    Indexed(Expression, Box<Initializer>),
}

/// 一元运算符
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    BitNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
    Reference,
    MutableReference,
}

/// 二元运算符
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    LeftShift,
    RightShift,
    RightShiftArithmetic,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Equal,
    NotEqual,
    BitAnd,
    BitXor,
    BitOr,
    LogicalAnd,
    LogicalOr,
}

// ============================================================================
// Display implementations (C-like syntax)
// ============================================================================

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for decl in &self.declarations {
            writeln!(f, "{decl}")?;
        }
        Ok(())
    }
}

impl Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Declaration::Import(import) => {
                write!(f, "import {}", import.module_path)?;
                if import.import_all {
                    write!(f, ".*")?;
                } else if !import.symbols.is_empty() {
                    write!(f, ":: {{ ")?;
                    for (name, alias) in &import.symbols {
                        if let Some(alias) = alias {
                            write!(f, "{} as {}, ", name, alias)?;
                        } else {
                            write!(f, "{}, ", name)?;
                        }
                    }
                    write!(f, "}}")?;
                }
                write!(f, ";")
            }
            Declaration::Function(func) => write!(f, "{func}"),
            Declaration::Global(global) => write!(f, "{global};"),
            Declaration::Struct(strct) => write!(f, "{strct};"),
            Declaration::Class(cls) => write!(f, "{cls};"),
            Declaration::VTable(vtable) => write!(f, "{vtable};"),
            Declaration::Enum(enm) => write!(f, "{enm};"),
            Declaration::TypeAlias(alias) => write!(f, "{alias};"),
            Declaration::Newtype(nt) => write!(f, "{nt};"),
            Declaration::Trait(trait_) => write!(f, "trait {};", trait_.name),
            Declaration::Effect(effect) => write!(f, "effect {};", effect.name),
            Declaration::Impl(impl_) => write!(
                f,
                "implement {} for {};",
                impl_.trait_name, impl_.target_type
            ),
            Declaration::ExternFunction(ext) => write!(f, "{ext};"),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_static {
            write!(f, "static ")?;
        }
        if self.is_inline {
            write!(f, "inline ")?;
        }
        write!(f, "{} {}(", self.return_type, self.name)?;
        if self.parameters.is_empty() {
            write!(f, "void")?;
        } else {
            for (i, param) in self.parameters.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{param}")?;
            }
        }
        write!(f, ") {}", self.body)
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.type_, self.name)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::Schar => write!(f, "signed char"),
            Type::Uchar => write!(f, "unsigned char"),
            Type::Short => write!(f, "short"),
            Type::Ushort => write!(f, "unsigned short"),
            Type::Int => write!(f, "int"),
            Type::Uint => write!(f, "unsigned int"),
            Type::Long => write!(f, "long"),
            Type::Ulong => write!(f, "unsigned long"),
            Type::LongLong => write!(f, "long long"),
            Type::UlongLong => write!(f, "unsigned long long"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::LongDouble => write!(f, "long double"),
            Type::Size => write!(f, "size_t"),
            Type::Ptrdiff => write!(f, "ptrdiff_t"),
            Type::Intptr => write!(f, "intptr_t"),
            Type::Uintptr => write!(f, "uintptr_t"),
            Type::Pointer(inner) => write!(f, "{}*", inner),
            Type::Array(inner, Some(size)) => write!(f, "{}[{size}]", inner),
            Type::Array(inner, None) => write!(f, "{}[]", inner),
            Type::FunctionPointer(ret, params) => {
                write!(f, "{} (*)(,", ret)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ")")
            }
            Type::Named(name) => write!(f, "{name}"),
            Type::Qualified(q, inner) => {
                if q.is_const {
                    write!(f, "const ")?;
                }
                if q.is_volatile {
                    write!(f, "volatile ")?;
                }
                if q.is_restrict {
                    write!(f, "restrict ")?;
                }
                write!(f, "{inner}")
            }
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for stmt in &self.statements {
            writeln!(f, "    {stmt}")?;
        }
        write!(f, "}}")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expression(expr) => write!(f, "{expr};"),
            Statement::Declaration(decl) => write!(f, "{decl}"),
            Statement::Variable(var) => write!(f, "{var};"),
            Statement::If(if_stmt) => write!(f, "{if_stmt}"),
            Statement::While(while_stmt) => write!(f, "{while_stmt}"),
            Statement::DoWhile(do_while) => write!(f, "{do_while}"),
            Statement::For(for_stmt) => write!(f, "{for_stmt}"),
            Statement::Switch(switch) => write!(f, "{switch}"),
            Statement::Match(match_stmt) => write!(f, "{match_stmt}"),
            Statement::Try(try_stmt) => write!(f, "{try_stmt}"),
            Statement::Break => write!(f, "break;"),
            Statement::Continue => write!(f, "continue;"),
            Statement::Return(Some(expr)) => write!(f, "return {expr};"),
            Statement::Return(None) => write!(f, "return;"),
            Statement::Goto(label) => write!(f, "goto {label};"),
            Statement::Label(label) => write!(f, "{label}:"),
            Statement::Empty => write!(f, ";"),
            Statement::Compound(block) => write!(f, "{block}"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(lit) => write!(f, "{lit}"),
            Expression::Variable(name) => write!(f, "{name}"),
            Expression::Unary(op, expr) => write!(f, "{op}({expr})"),
            Expression::Binary(op, left, right) => write!(f, "({left} {op} {right})"),
            Expression::Ternary(cond, then_expr, else_expr) => {
                write!(f, "({cond} ? {then_expr} : {else_expr})")
            }
            Expression::Assign(target, value) => write!(f, "({target} = {value})"),
            Expression::AssignOp(op, target, value) => write!(f, "({target} {op}= {value})"),
            Expression::Call(func, args) => {
                write!(f, "{func}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Expression::Index(arr, idx) => write!(f, "{arr}[{idx}]"),
            Expression::Member(obj, name) => write!(f, "{obj}.{name}"),
            Expression::PointerMember(obj, name) => write!(f, "{obj}->{name}"),
            Expression::AddressOf(expr) => write!(f, "&{expr}"),
            Expression::Dereference(expr) => write!(f, "*{expr}"),
            Expression::Cast(ty, expr) => write!(f, "(({ty}){expr})"),
            Expression::SizeOf(ty) => write!(f, "sizeof({ty})"),
            Expression::SizeOfExpr(expr) => write!(f, "sizeof({expr})"),
            Expression::AlignOf(ty) => write!(f, "_Alignof({ty})"),
            Expression::Comma(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{expr}")?;
                }
                Ok(())
            }
            Expression::Parenthesized(expr) => write!(f, "({expr})"),
            Expression::InitializerList(init) => {
                write!(f, "{{")?;
                for (i, item) in init.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "}}")
            }
            Expression::CompoundLiteral(ty, init) => {
                write!(f, "({ty}){{")?;
                for (i, item) in init.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(n) => write!(f, "{n}"),
            Literal::UnsignedInteger(n) => write!(f, "{n}u"),
            Literal::Long(n) => write!(f, "{n}L"),
            Literal::UnsignedLong(n) => write!(f, "{n}uL"),
            Literal::LongLong(n) => write!(f, "{n}LL"),
            Literal::UnsignedLongLong(n) => write!(f, "{n}uLL"),
            Literal::Float(n) => write!(f, "{n}f"),
            Literal::Double(n) => write!(f, "{n}"),
            Literal::Char(c) => write!(f, "'{c}'"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::NullPointer => write!(f, "NULL"),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::BitNot => write!(f, "~"),
            UnaryOp::PreIncrement => write!(f, "++"),
            UnaryOp::PreDecrement => write!(f, "--"),
            UnaryOp::PostIncrement => write!(f, "++"),
            UnaryOp::PostDecrement => write!(f, "--"),
            UnaryOp::Reference => write!(f, "&"),
            UnaryOp::MutableReference => write!(f, "&mut"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::LeftShift => write!(f, "<<"),
            BinaryOp::RightShift => write!(f, ">>>"),
            BinaryOp::RightShiftArithmetic => write!(f, ">>"),
            BinaryOp::LessThan => write!(f, "<"),
            BinaryOp::LessThanEqual => write!(f, "<="),
            BinaryOp::GreaterThan => write!(f, ">"),
            BinaryOp::GreaterThanEqual => write!(f, ">="),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::BitAnd => write!(f, "&"),
            BinaryOp::BitXor => write!(f, "^"),
            BinaryOp::BitOr => write!(f, "|"),
            BinaryOp::LogicalAnd => write!(f, "&&"),
            BinaryOp::LogicalOr => write!(f, "||"),
        }
    }
}

impl Display for Initializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Initializer::Expression(expr) => write!(f, "{expr}"),
            Initializer::List(items) => {
                write!(f, "{{")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "}}")
            }
            Initializer::Named(name, init) => write!(f, ".{name} = {init}"),
            Initializer::Indexed(idx, init) => write!(f, "[{idx}] = {init}"),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_extern {
            write!(f, "extern ")?;
        }
        if self.is_static {
            write!(f, "static ")?;
        }
        write!(f, "{} {}", self.type_, self.name)?;
        if let Some(init) = &self.initializer {
            write!(f, " = {init}")?;
        }
        Ok(())
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct {}", self.name)?;
        if !self.fields.is_empty() {
            writeln!(f, " {{")?;
            for field in &self.fields {
                writeln!(f, "    {field};")?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.type_, self.name)
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class {}", self.name)?;
        if let Some(parent) = &self.extends {
            write!(f, " extends {parent}")?;
        }
        if !self.implements.is_empty() {
            write!(f, " implements {}", self.implements.join(", "))?;
        }
        if !self.fields.is_empty() {
            writeln!(f, " {{")?;
            for field in &self.fields {
                writeln!(f, "    {field};")?;
            }
            if self.has_vtable {
                writeln!(f, "    // vtable: {} methods", self.vtable_indices.len())?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl Display for VTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vtable {} for {}", self.name, self.class_name)?;
        if !self.entries.is_empty() {
            writeln!(f, " {{")?;
            for entry in &self.entries {
                writeln!(f, "    {} -> fn(...);", entry.method_name)?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enum {}", self.name)?;
        if !self.variants.is_empty() {
            writeln!(f, " {{")?;
            for (i, variant) in self.variants.iter().enumerate() {
                if i > 0 {
                    writeln!(f, ",")?;
                }
                write!(f, "    {variant}")?;
            }
            write!(f, "\n}}")?;
        }
        Ok(())
    }
}

impl Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(value) = self.value {
            write!(f, " = {value}")?;
        }
        Ok(())
    }
}

impl Display for TypeAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "typedef {} {}", self.type_, self.name)
    }
}

impl Display for Newtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "newtype {} {}", self.type_, self.name)
    }
}

impl Display for EffectOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation {}(", self.name)?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
        }
        write!(f, ")")?;
        if let Some(ret) = &self.return_type {
            write!(f, " -> {ret}")?;
        }
        Ok(())
    }
}

impl Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "effect {}", self.name)?;
        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{param}")?;
            }
            write!(f, ">")?;
        }
        write!(f, " {{")?;
        for op in &self.operations {
            writeln!(f, "  {op};")?;
        }
        write!(f, "}}")
    }
}

impl Display for TraitMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{param}")?;
            }
            write!(f, ">")?;
        }
        write!(f, "(")?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
        }
        write!(f, ")")?;
        if let Some(ret) = &self.return_type {
            write!(f, " -> {ret}")?;
        }
        if self.default_body.is_some() {
            write!(f, " = ...")?;
        }
        Ok(())
    }
}

impl Display for Trait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trait {}", self.name)?;
        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{param}")?;
            }
            write!(f, ">")?;
        }
        if !self.extends.is_empty() {
            write!(f, " extends ")?;
            for (i, ext) in self.extends.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{ext}")?;
            }
        }
        write!(f, " {{")?;
        for method in &self.methods {
            writeln!(f, "  {method};")?;
        }
        write!(f, "}}")
    }
}

impl Display for Impl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_effect {
            write!(f, "implement effect ")?;
        } else {
            write!(f, "implement ")?;
        }
        write!(f, "{} for {}", self.trait_name, self.target_type)?;
        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{param}")?;
            }
            write!(f, ">")?;
        }
        write!(f, " {{ ... }}")?;
        Ok(())
    }
}

impl Display for ExternFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "extern {} {}(", self.return_type, self.name)?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
        }
        write!(f, ")")
    }
}

impl Display for GlobalVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_static {
            write!(f, "static ")?;
        }
        write!(f, "{} {}", self.type_, self.name)?;
        if let Some(init) = &self.initializer {
            write!(f, " = {init}")?;
        }
        Ok(())
    }
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "if ({}) {}", self.condition, self.then_branch)?;
        if let Some(else_branch) = &self.else_branch {
            write!(f, " else {else_branch}")?;
        }
        Ok(())
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while ({}) {}", self.condition, self.body)
    }
}

impl Display for DoWhileStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "do {} while ({});", self.body, self.condition)
    }
}

impl Display for ForStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "for (")?;
        if let Some(init) = &self.initializer {
            write!(f, "{init}")?;
        } else {
            write!(f, ";")?;
        }
        if let Some(cond) = &self.condition {
            write!(f, " {cond}")?;
        }
        write!(f, ";")?;
        if let Some(inc) = &self.increment {
            write!(f, " {inc}")?;
        }
        write!(f, ") {}", self.body)
    }
}

impl Display for SwitchStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "switch ({}) {{", self.expression)?;
        for case in &self.cases {
            writeln!(f, "    case {}: {}", case.value, case.body)?;
        }
        if let Some(default) = &self.default {
            writeln!(f, "    default: {default}")?;
        }
        write!(f, "}}")
    }
}

impl Display for SwitchCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.value, self.body)
    }
}

impl Display for MatchStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "match ({}) {{", self.scrutinee)?;
        for case in &self.cases {
            writeln!(f, "    {} => {}", case.pattern, case.body)?;
        }
        write!(f, "}}")
    }
}

impl Display for MatchCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}", self.pattern, self.body)
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Variable(name) => write!(f, "{name}"),
            Pattern::Literal(lit) => write!(f, "{lit}"),
            Pattern::Constructor(name, args) => {
                write!(f, "{name}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Pattern::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            }
            Pattern::Record(name, fields) => {
                write!(f, "{name}{{")?;
                for (i, (fname, fpat)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{fname}: {fpat}")?;
                }
                write!(f, "}}")
            }
            Pattern::Or(left, right) => write!(f, "{left} | {right}"),
        }
    }
}

impl Display for TryStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "try {}", self.body)?;
        for catch in &self.catch_clauses {
            write!(f, "catch")?;
            if let Some(ty) = &catch.exception_type {
                write!(f, " ({ty}")?;
                if let Some(var) = &catch.variable_name {
                    write!(f, " {var}")?;
                }
                write!(f, ")")?;
            }
            writeln!(f, " {}", catch.body)?;
        }
        if let Some(finally) = &self.finally_block {
            write!(f, "finally {finally}")?;
        }
        Ok(())
    }
}

// ============================================================================
// Builder
// ============================================================================

/// LIR 程序构建器
pub struct LirBuilder {
    program: Program,
}

impl LirBuilder {
    pub fn new() -> Self {
        Self {
            program: Program {
                declarations: Vec::new(),
            },
        }
    }

    /// 添加函数
    pub fn add_function(&mut self, func: Function) {
        self.program.declarations.push(Declaration::Function(func));
    }

    /// 添加全局变量
    pub fn add_global(&mut self, global: GlobalVar) {
        self.program.declarations.push(Declaration::Global(global));
    }

    /// 添加结构体
    pub fn add_struct(&mut self, strct: Struct) {
        self.program.declarations.push(Declaration::Struct(strct));
    }

    /// 添加枚举
    pub fn add_enum(&mut self, enm: Enum) {
        self.program.declarations.push(Declaration::Enum(enm));
    }

    /// 构建程序
    pub fn build(self) -> Program {
        self.program
    }
}

impl Default for LirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    /// 创建空的 LIR 程序
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
        }
    }

    /// 添加声明
    pub fn add(&mut self, decl: Declaration) {
        self.declarations.push(decl);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Type {
    pub fn pointer(self) -> Self {
        Type::Pointer(Box::new(self))
    }

    pub fn const_(self) -> Self {
        Type::Qualified(Qualifiers::const_(), Box::new(self))
    }

    pub fn array(self, size: u64) -> Self {
        Type::Array(Box::new(self), Some(size))
    }

    pub fn named(name: &str) -> Self {
        Type::Named(name.to_string())
    }

    /// 获取类型在 x86_64 64位系统下的大小（字节）
    pub fn size_of(&self) -> usize {
        match self {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Long | Type::Ulong => 8,
            Type::LongLong | Type::UlongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::LongDouble => 16,
            Type::Size | Type::Ptrdiff | Type::Intptr | Type::Uintptr => 8,
            Type::Pointer(_) => 8,
            Type::FunctionPointer(_, _) => 8,
            Type::Array(elem, Some(len)) => elem.size_of() * (*len as usize),
            Type::Array(elem, None) => elem.size_of(), // unsized array, size is element size
            Type::Named(_) => 0,                       // named types need lookup, handled by caller
            Type::Qualified(_, ty) => ty.size_of(),
        }
    }

    /// 获取类型在 x86_64 64位系统下的对齐要求（字节）
    pub fn align_of(&self) -> usize {
        match self {
            Type::Void => 1,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Long | Type::Ulong => 8,
            Type::LongLong | Type::UlongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::LongDouble => 16,
            Type::Size | Type::Ptrdiff | Type::Intptr | Type::Uintptr => 8,
            Type::Pointer(_) => 8,
            Type::FunctionPointer(_, _) => 8,
            Type::Array(elem, _) => elem.align_of(),
            Type::Named(_) => 8, // default alignment for named types
            Type::Qualified(_, ty) => ty.align_of(),
        }
    }
}

impl Function {
    /// 创建函数
    pub fn new(name: &str, return_type: Type) -> Self {
        Self {
            name: name.to_string(),
            type_params: Vec::new(),
            return_type,
            parameters: Vec::new(),
            body: Block {
                statements: Vec::new(),
            },
            is_static: false,
            is_inline: false,
        }
    }

    /// 添加参数
    pub fn param(mut self, name: &str, type_: Type) -> Self {
        self.parameters.push(Parameter {
            name: name.to_string(),
            type_,
        });
        self
    }

    pub fn static_(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn inline(mut self) -> Self {
        self.is_inline = true;
        self
    }

    pub fn add_stmt(&mut self, stmt: Statement) {
        self.body.statements.push(stmt);
    }
}

impl Block {
    /// 创建空语句块
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    /// 添加语句
    pub fn add(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::should_implement_trait)]
impl Expression {
    pub fn var(name: &str) -> Self {
        Expression::Variable(name.to_string())
    }

    pub fn int(n: i64) -> Self {
        Expression::Literal(Literal::Integer(n))
    }

    pub fn double(n: f64) -> Self {
        Expression::Literal(Literal::Double(n))
    }

    pub fn string(s: &str) -> Self {
        Expression::Literal(Literal::String(s.to_string()))
    }

    pub fn bool(b: bool) -> Self {
        Expression::Literal(Literal::Bool(b))
    }

    pub fn null() -> Self {
        Expression::Literal(Literal::NullPointer)
    }

    pub fn call(self, args: Vec<Expression>) -> Self {
        Expression::Call(Box::new(self), args)
    }

    pub fn add(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::Add, Box::new(self), Box::new(rhs))
    }

    pub fn sub(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::Subtract, Box::new(self), Box::new(rhs))
    }

    pub fn mul(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::Multiply, Box::new(self), Box::new(rhs))
    }

    pub fn div(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::Divide, Box::new(self), Box::new(rhs))
    }

    pub fn eq(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::Equal, Box::new(self), Box::new(rhs))
    }

    pub fn lt(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::LessThan, Box::new(self), Box::new(rhs))
    }

    pub fn gt(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::GreaterThan, Box::new(self), Box::new(rhs))
    }

    pub fn and(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::LogicalAnd, Box::new(self), Box::new(rhs))
    }

    pub fn or(self, rhs: Self) -> Self {
        Expression::Binary(BinaryOp::LogicalOr, Box::new(self), Box::new(rhs))
    }

    pub fn assign(self, rhs: Self) -> Self {
        Expression::Assign(Box::new(self), Box::new(rhs))
    }

    pub fn member(self, name: &str) -> Self {
        Expression::Member(Box::new(self), name.to_string())
    }

    pub fn index(self, idx: Self) -> Self {
        Expression::Index(Box::new(self), Box::new(idx))
    }

    pub fn cast(self, type_: Type) -> Self {
        Expression::Cast(type_, Box::new(self))
    }

    pub fn address_of(self) -> Self {
        Expression::AddressOf(Box::new(self))
    }

    pub fn deref(self) -> Self {
        Expression::Dereference(Box::new(self))
    }

    pub fn not(self) -> Self {
        Expression::Unary(UnaryOp::Not, Box::new(self))
    }

    pub fn neg(self) -> Self {
        Expression::Unary(UnaryOp::Minus, Box::new(self))
    }

    pub fn pre_incr(self) -> Self {
        Expression::Unary(UnaryOp::PreIncrement, Box::new(self))
    }
}

impl Statement {
    pub fn expr(expr: Expression) -> Self {
        Statement::Expression(expr)
    }

    pub fn return_(expr: Option<Expression>) -> Self {
        Statement::Return(expr)
    }

    pub fn if_(cond: Expression, then_: Statement, else_: Option<Statement>) -> Self {
        Statement::If(IfStatement {
            condition: cond,
            then_branch: Box::new(then_),
            else_branch: else_.map(Box::new),
        })
    }

    pub fn while_(cond: Expression, body: Statement) -> Self {
        Statement::While(WhileStatement {
            condition: cond,
            body: Box::new(body),
        })
    }

    pub fn for_(
        init: Option<Statement>,
        cond: Option<Expression>,
        incr: Option<Expression>,
        body: Statement,
    ) -> Self {
        Statement::For(ForStatement {
            initializer: init.map(Box::new),
            condition: cond,
            increment: incr,
            body: Box::new(body),
        })
    }

    pub fn block(block: Block) -> Self {
        Statement::Compound(block)
    }

    pub fn break_() -> Self {
        Statement::Break
    }

    pub fn continue_() -> Self {
        Statement::Continue
    }
}

impl Variable {
    pub fn new(name: &str, type_: Type) -> Self {
        Self {
            name: name.to_string(),
            type_,
            initializer: None,
            is_static: false,
            is_extern: false,
        }
    }

    pub fn init(mut self, expr: Expression) -> Self {
        self.initializer = Some(expr);
        self
    }

    pub fn static_(mut self) -> Self {
        self.is_static = true;
        self
    }
}
