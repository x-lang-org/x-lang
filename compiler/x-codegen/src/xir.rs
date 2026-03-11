//! X IR - X语言中间表示
//!
//! 这是一个简化的、类 C 的中间表示，用于降低到 Zig、LLVM 和其他后端。

use std::fmt::{self, Display, Write};

/// X IR 程序
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

/// 声明
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    /// 函数声明
    Function(Function),
    /// 全局变量
    Global(GlobalVar),
    /// 结构体定义
    Struct(Struct),
    /// 枚举定义
    Enum(Enum),
    /// 类型别名
    TypeAlias(TypeAlias),
    /// 外部函数声明
    ExternFunction(ExternFunction),
}

/// 函数
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
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

/// 类型别名
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub type_: Type,
}

/// 外部函数声明
#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<Type>,
}

/// 类型
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// void
    Void,
    /// bool (boolean type)
    Bool,
    /// char
    Char,
    /// signed char
    Schar,
    /// unsigned char
    Uchar,
    /// short
    Short,
    /// unsigned short
    Ushort,
    /// int
    Int,
    /// unsigned int
    Uint,
    /// long
    Long,
    /// unsigned long
    Ulong,
    /// long long
    LongLong,
    /// unsigned long long
    UlongLong,
    /// float
    Float,
    /// double
    Double,
    /// long double
    LongDouble,
    /// size_t
    Size,
    /// ptrdiff_t
    Ptrdiff,
    /// intptr_t
    Intptr,
    /// uintptr_t
    Uintptr,
    /// 指针类型
    Pointer(Box<Type>),
    /// 数组类型
    Array(Box<Type>, Option<u64>),
    /// 函数指针类型
    FunctionPointer(Box<Type>, Vec<Type>),
    /// 命名类型（结构体/枚举/typedef）
    Named(String),
    /// 限定类型（const/volatile/restrict）
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

/// 语句块
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// 语句
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// 表达式语句
    Expression(Expression),
    /// 声明语句
    Declaration(Declaration),
    /// 变量声明
    Variable(Variable),
    /// if 语句
    If(IfStatement),
    /// while 语句
    While(WhileStatement),
    /// do-while 语句
    DoWhile(DoWhileStatement),
    /// for 语句
    For(ForStatement),
    /// switch 语句
    Switch(SwitchStatement),
    /// match 语句（模式匹配）
    Match(MatchStatement),
    /// try 语句
    Try(TryStatement),
    /// break
    Break,
    /// continue
    Continue,
    /// return
    Return(Option<Expression>),
    /// goto
    Goto(String),
    /// 标签
    Label(String),
    /// 空语句
    Empty,
    /// 复合语句（块）
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

/// for-in 语句（迭代器风格）
#[derive(Debug, Clone, PartialEq)]
pub struct ForInStatement {
    pub variable: String,
    pub iterator: Expression,
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
    /// 通配符
    Wildcard,
    /// 变量绑定
    Variable(String),
    /// 字面量
    Literal(Literal),
    /// 构造器模式
    Constructor(String, Vec<Pattern>),
    /// 元组模式
    Tuple(Vec<Pattern>),
    /// 记录模式
    Record(String, Vec<(String, Pattern)>),
    /// 或模式
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

/// 表达式
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// 字面量
    Literal(Literal),
    /// 变量引用
    Variable(String),
    /// 一元运算
    Unary(UnaryOp, Box<Expression>),
    /// 二元运算
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    /// 三元运算
    Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
    /// 赋值
    Assign(Box<Expression>, Box<Expression>),
    /// 复合赋值
    AssignOp(BinaryOp, Box<Expression>, Box<Expression>),
    /// 函数调用
    Call(Box<Expression>, Vec<Expression>),
    /// 数组下标
    Index(Box<Expression>, Box<Expression>),
    /// 成员访问
    Member(Box<Expression>, String),
    /// 指针成员访问
    PointerMember(Box<Expression>, String),
    /// 取地址
    AddressOf(Box<Expression>),
    /// 解引用
    Dereference(Box<Expression>),
    /// 类型转换
    Cast(Type, Box<Expression>),
    /// sizeof
    SizeOf(Type),
    /// sizeof 表达式
    SizeOfExpr(Box<Expression>),
    /// alignof
    AlignOf(Type),
    /// 逗号表达式
    Comma(Vec<Expression>),
    /// 括号表达式
    Parenthesized(Box<Expression>),
    /// 初始化列表
    InitializerList(Vec<Initializer>),
    /// 复合字面量
    CompoundLiteral(Type, Vec<Initializer>),
}

/// 字面量
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// 整数字面量
    Integer(i64),
    /// 无符号整数字面量
    UnsignedInteger(u64),
    /// 长整数字面量
    Long(i64),
    /// 无符号长整数字面量
    UnsignedLong(u64),
    /// 长长整数字面量
    LongLong(i64),
    /// 无符号长长整数字面量
    UnsignedLongLong(u64),
    /// 浮点数字面量
    Float(f64),
    /// 双精度浮点数字面量
    Double(f64),
    /// 字符字面量
    Char(char),
    /// 字符串字面量
    String(String),
    /// 布尔字面量
    Bool(bool),
    /// 空指针
    NullPointer,
}

/// 初始化器
#[derive(Debug, Clone, PartialEq)]
pub enum Initializer {
    /// 表达式初始化器
    Expression(Expression),
    /// 数组/结构体初始化器
    List(Vec<Initializer>),
    /// 命名初始化器
    Named(String, Box<Initializer>),
    /// 下标初始化器
    Indexed(Expression, Box<Initializer>),
}

/// 一元运算符
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum UnaryOp {
    /// +
    Plus,
    /// -
    Minus,
    /// !
    Not,
    /// ~
    BitNot,
    /// ++ (前置)
    PreIncrement,
    /// -- (前置)
    PreDecrement,
    /// ++ (后置)
    PostIncrement,
    /// -- (后置)
    PostDecrement,
}

/// 二元运算符
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BinaryOp {
    /// +
    Add,
    /// -
    Subtract,
    /// *
    Multiply,
    /// /
    Divide,
    /// %
    Modulo,
    /// <<
    LeftShift,
    /// >>
    RightShift,
    /// <
    LessThan,
    /// <=
    LessThanEqual,
    /// >
    GreaterThan,
    /// >=
    GreaterThanEqual,
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// &
    BitAnd,
    /// ^
    BitXor,
    /// |
    BitOr,
    /// &&
    LogicalAnd,
    /// ||
    LogicalOr,
}

// ============================================================================
// Display implementations (C-like syntax, used for code generation)
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
            Declaration::Function(func) => write!(f, "{func}"),
            Declaration::Global(global) => write!(f, "{global};"),
            Declaration::Struct(strct) => write!(f, "{strct};"),
            Declaration::Enum(enm) => write!(f, "{enm};"),
            Declaration::TypeAlias(alias) => write!(f, "{alias};"),
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

impl Display for ExternFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "extern {} {}(", self.return_type, self.name)?;
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
        write!(f, ")")
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
            Type::Pointer(inner) => {
                if let Type::FunctionPointer(ret, params) = inner.as_ref() {
                    write!(f, "{ret}(*")?;
                    if !params.is_empty() {
                        write!(f, "(")?;
                        for (i, param) in params.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{param}")?;
                        }
                        write!(f, ")")?;
                    }
                    write!(f, ")")?;
                } else {
                    write!(f, "{inner}*")?;
                }
                Ok(())
            }
            Type::Array(inner, size) => {
                write!(f, "{inner}")?;
                if let Some(s) = size {
                    write!(f, "[{s}]")?;
                } else {
                    write!(f, "[]")?;
                }
                Ok(())
            }
            Type::FunctionPointer(ret, params) => {
                write!(f, "{ret}(*)(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ")")
            }
            Type::Named(name) => write!(f, "{name}"),
            Type::Qualified(qual, inner) => {
                if qual.is_const {
                    write!(f, "const ")?;
                }
                if qual.is_volatile {
                    write!(f, "volatile ")?;
                }
                if qual.is_restrict {
                    write!(f, "restrict ")?;
                }
                write!(f, "{inner}")
            }
        }
    }
}

impl Display for Qualifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_const {
            write!(f, "const ")?;
        }
        if self.is_volatile {
            write!(f, "volatile ")?;
        }
        if self.is_restrict {
            write!(f, "restrict ")?;
        }
        Ok(())
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for stmt in &self.statements {
            for line in stmt.to_string().lines() {
                writeln!(f, "    {line}")?;
            }
        }
        write!(f, "}}")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expression(expr) => writeln!(f, "{expr};"),
            Statement::Declaration(decl) => writeln!(f, "{decl}"),
            Statement::Variable(var) => writeln!(f, "{var};"),
            Statement::If(if_stmt) => write!(f, "{if_stmt}"),
            Statement::While(while_stmt) => write!(f, "{while_stmt}"),
            Statement::DoWhile(do_while) => write!(f, "{do_while}"),
            Statement::For(for_stmt) => write!(f, "{for_stmt}"),
            Statement::Switch(switch) => write!(f, "{switch}"),
            Statement::Match(match_stmt) => write!(f, "/* match: {:?} */", match_stmt.scrutinee),
            Statement::Try(try_stmt) => write!(f, "try {{ ... }} catch {{ ... }}"),
            Statement::Break => writeln!(f, "break;"),
            Statement::Continue => writeln!(f, "continue;"),
            Statement::Return(None) => writeln!(f, "return;"),
            Statement::Return(Some(expr)) => writeln!(f, "return {expr};"),
            Statement::Goto(label) => writeln!(f, "goto {label};"),
            Statement::Label(label) => writeln!(f, "{label}:;"),
            Statement::Empty => writeln!(f, ";"),
            Statement::Compound(block) => write!(f, "{block}"),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_static {
            write!(f, "static ")?;
        }
        if self.is_extern {
            write!(f, "extern ")?;
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
        write!(f, "if ({}) ", self.condition)?;
        match &*self.then_branch {
            Statement::Compound(block) => write!(f, "{block}")?,
            _ => write!(f, "{{\n    {};\n}}", self.then_branch)?,
        }
        if let Some(else_branch) = &self.else_branch {
            write!(f, " else ")?;
            match &**else_branch {
                Statement::Compound(block) => write!(f, "{block}")?,
                Statement::If(_) => write!(f, "{else_branch}")?,
                _ => write!(f, "{{\n    {};\n}}", else_branch)?,
            }
        }
        Ok(())
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while ({}) ", self.condition)?;
        match &*self.body {
            Statement::Compound(block) => write!(f, "{block}"),
            _ => write!(f, "{{\n    {};\n}}", self.body),
        }
    }
}

impl Display for DoWhileStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "do ")?;
        match &*self.body {
            Statement::Compound(block) => write!(f, "{block}")?,
            _ => write!(f, "{{\n    {};\n}}", self.body)?,
        }
        write!(f, " while ({});", self.condition)
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
        write!(f, " ")?;
        if let Some(cond) = &self.condition {
            write!(f, "{cond}")?;
        }
        write!(f, "; ")?;
        if let Some(incr) = &self.increment {
            write!(f, "{incr}")?;
        }
        write!(f, ") ")?;
        match &*self.body {
            Statement::Compound(block) => write!(f, "{block}"),
            _ => write!(f, "{{\n    {};\n}}", self.body),
        }
    }
}

impl Display for SwitchStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "switch ({}) {{\n", self.expression)?;
        for case in &self.cases {
            write!(f, "    case {}: {}", case.value, case.body)?;
        }
        if let Some(default) = &self.default {
            write!(f, "    default: {default}")?;
        }
        write!(f, "}}")
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(lit) => write!(f, "{lit}"),
            Expression::Variable(name) => write!(f, "{name}"),
            Expression::Unary(op, expr) => write!(f, "{op}{expr}"),
            Expression::Binary(op, left, right) => write!(f, "({left} {op} {right})"),
            Expression::Ternary(cond, then_, else_) => {
                write!(f, "({cond} ? {then_} : {else_})")
            }
            Expression::Assign(left, right) => write!(f, "({left} = {right})"),
            Expression::AssignOp(op, left, right) => write!(f, "({left} {op}= {right})"),
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
            Expression::Index(array, index) => write!(f, "{array}[{index}]"),
            Expression::Member(expr, member) => write!(f, "{expr}.{member}"),
            Expression::PointerMember(expr, member) => write!(f, "{expr}->{member}"),
            Expression::AddressOf(expr) => write!(f, "&{expr}"),
            Expression::Dereference(expr) => write!(f, "*{expr}"),
            Expression::Cast(type_, expr) => write!(f, "(({type_}){expr})"),
            Expression::SizeOf(type_) => write!(f, "sizeof({type_})"),
            Expression::SizeOfExpr(expr) => write!(f, "sizeof {expr}"),
            Expression::AlignOf(type_) => write!(f, "alignof({type_})"),
            Expression::Comma(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{expr}")?;
                }
                write!(f, ")")
            }
            Expression::Parenthesized(expr) => write!(f, "({expr})"),
            Expression::InitializerList(inits) => {
                write!(f, "{{")?;
                for (i, init) in inits.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{init}")?;
                }
                write!(f, "}}")
            }
            Expression::CompoundLiteral(type_, inits) => {
                write!(f, "({type_}){{")?;
                for (i, init) in inits.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{init}")?;
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
            Literal::UnsignedLong(n) => write!(f, "{n}UL"),
            Literal::LongLong(n) => write!(f, "{n}LL"),
            Literal::UnsignedLongLong(n) => write!(f, "{n}ULL"),
            Literal::Float(n) => write!(f, "{n}f"),
            Literal::Double(n) => write!(f, "{n}"),
            Literal::Char(c) => write!(f, "'{c}'"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Bool(true) => write!(f, "true"),
            Literal::Bool(false) => write!(f, "false"),
            Literal::NullPointer => write!(f, "NULL"),
        }
    }
}

impl Display for Initializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Initializer::Expression(expr) => write!(f, "{expr}"),
            Initializer::List(inits) => {
                write!(f, "{{")?;
                for (i, init) in inits.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{init}")?;
                }
                write!(f, "}}")
            }
            Initializer::Named(name, init) => write!(f, ".{name} = {init}"),
            Initializer::Indexed(index, init) => write!(f, "[{index}] = {init}"),
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
            BinaryOp::RightShift => write!(f, ">>"),
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

// ============================================================================
// Builder utilities
// ============================================================================

impl Program {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
        }
    }

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
}

impl Function {
    pub fn new(name: &str, return_type: Type) -> Self {
        Self {
            name: name.to_string(),
            return_type,
            parameters: Vec::new(),
            body: Block {
                statements: Vec::new(),
            },
            is_static: false,
            is_inline: false,
        }
    }

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
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

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
