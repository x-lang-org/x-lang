use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use x_lexer::span::Span;
use x_parser::ast::{
    BinaryOp, Block, CatchClause, ClassDecl, ClassMember, Declaration, ExternFunctionDecl, Expression, ExpressionKind, FunctionDecl, Literal,
    MatchCase, MatchStatement, Pattern, Program, Spanned, Statement, StatementKind, TraitDecl, TryStatement,
    UnaryOp,
};

#[derive(Debug)]
pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, FunctionDecl>,
    foreign_functions: HashMap<String, ExternFunctionDecl>,
    classes: HashMap<String, ClassDecl>,
    traits: HashMap<String, TraitDecl>,
    enums: HashMap<String, x_parser::ast::EnumDecl>,
    // TCP networking
    tcp_servers: HashMap<usize, TcpListener>,
    tcp_connections: HashMap<usize, TcpStream>,
    handle_counter: usize,
    // Async operations
    async_results: HashMap<usize, Result<Value, String>>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Array(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<Vec<(String, Value)>>>),
    /// 对象实例
    Object {
        class_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
    },
    /// 闭包：参数名列表、函数体、捕获的环境变量
    Closure {
        params: Vec<String>,
        body: Block,
        captured: Rc<RefCell<HashMap<String, Value>>>,
    },
    /// 枚举值：类型名、变体名、关联值
    Enum(String, String, Vec<Value>),
    /// 类型值：用于类型构造器表达式，如 Pointer(Void)
    Type { name: String, args: Vec<TypeValue> },
    /// 原始指针值（用于 FFI）
    Pointer(usize),  // 存储地址值
    Null,
    None,
    Unit,
    Option(Box<Value>),
    Result(Box<Value>, Box<Value>),
}

/// 用于表示类型值中的类型参数
#[derive(Debug, Clone, PartialEq)]
pub enum TypeValue {
    Named(String),
    Pointer(Box<TypeValue>),
    ConstPointer(Box<TypeValue>),
    Void,
}

impl Value {
    fn new_array(v: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(v)))
    }
    fn new_map() -> Value {
        Value::Map(Rc::new(RefCell::new(Vec::new())))
    }
    fn deep_clone(&self) -> Value {
        match self {
            Value::Array(rc) => {
                let items: Vec<Value> = rc.borrow().iter().map(|v| v.deep_clone()).collect();
                Value::new_array(items)
            }
            Value::Map(rc) => {
                let entries: Vec<(String, Value)> = rc
                    .borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.deep_clone()))
                    .collect();
                Value::Map(Rc::new(RefCell::new(entries)))
            }
            Value::Object { class_name, fields } => {
                let cloned_fields: HashMap<String, Value> = fields
                    .borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.deep_clone()))
                    .collect();
                Value::Object {
                    class_name: class_name.clone(),
                    fields: Rc::new(RefCell::new(cloned_fields)),
                }
            }
            Value::Option(v) => Value::Option(Box::new(v.deep_clone())),
            Value::Result(ok, err) => {
                Value::Result(Box::new(ok.deep_clone()), Box::new(err.deep_clone()))
            }
            Value::Closure { params, body, captured } => {
                let cloned_captured: HashMap<String, Value> = captured
                    .borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.deep_clone()))
                    .collect();
                Value::Closure {
                    params: params.clone(),
                    body: body.clone(),
                    captured: Rc::new(RefCell::new(cloned_captured)),
                }
            }
            other => other.clone(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
            (Value::Null, Value::Null) => true,
            (Value::None, Value::None) => true,
            (Value::Unit, Value::Unit) => true,
            (Value::Pointer(a), Value::Pointer(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
            (Value::Option(a), Value::Option(b)) => *a == *b,
            (Value::Result(ok1, err1), Value::Result(ok2, err2)) => ok1 == ok2 && err1 == err2,
            (
                Value::Object { class_name: c1, fields: f1 },
                Value::Object { class_name: c2, fields: f2 },
            ) => c1 == c2 && *f1.borrow() == *f2.borrow(),
            // 闭包比较：比较参数和捕获的值（函数体不比较）
            (
                Value::Closure { params: p1, captured: c1, .. },
                Value::Closure { params: p2, captured: c2, .. },
            ) => p1 == p2 && *c1.borrow() == *c2.borrow(),
            // 类型值比较
            (
                Value::Type { name: n1, args: a1 },
                Value::Type { name: n2, args: a2 },
            ) => n1 == n2 && a1 == a2,
            _ => false,
        }
    }
}

enum ControlFlow {
    None,
    Return(Value),
    Break,
    #[allow(dead_code)]
    Continue,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        // 添加None作为内置变量
        variables.insert("None".to_string(), Value::None);
        Self {
            variables,
            functions: HashMap::new(),
            foreign_functions: HashMap::new(),
            classes: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
            tcp_servers: HashMap::new(),
            tcp_connections: HashMap::new(),
            handle_counter: 0,
            async_results: HashMap::new(),
        }
    }

    fn next_handle(&mut self) -> usize {
        self.handle_counter += 1;
        self.handle_counter
    }

    fn as_array(&self, value: &Value) -> Result<Vec<Value>, InterpreterError> {
        match value {
            Value::Array(arr) => Ok(arr.borrow().clone()),
            _ => Err(InterpreterError::runtime_no_span(format!("Expected array, got {:?}", value))),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), InterpreterError> {
        // 首先加载所有声明（函数、变量等）
        for decl in &program.declarations {
            self.load_declaration(decl)?;
        }

        // 然后执行所有顶级语句
        for stmt in &program.statements {
            match self.execute_statement(stmt)? {
                ControlFlow::None => {}
                ControlFlow::Return(_) => break,
                ControlFlow::Break => break,
                ControlFlow::Continue => {}
            }
        }

        // 如果有 main 函数，也运行它（为了向后兼容）
        if let Some(main_func) = self.functions.get("main").cloned() {
            // 不再保存和恢复变量，让main函数可以访问全局变量
            let _ = self.execute_block_stmt(&main_func.body)?;
        }

        Ok(())
    }

    fn load_declaration(&mut self, decl: &Declaration) -> Result<(), InterpreterError> {
        match decl {
            Declaration::Function(func) => {
                self.functions.insert(func.name.clone(), func.clone());
            }
            Declaration::ExternFunction(extern_func) => {
                self.foreign_functions.insert(extern_func.name.clone(), extern_func.clone());
            }
            Declaration::Variable(var) => {
                if let Some(init) = &var.initializer {
                    let val = self.eval(init)?;
                    self.variables.insert(var.name.clone(), val);
                }
            }
            Declaration::Class(class) => {
                self.classes.insert(class.name.clone(), class.clone());
            }
            Declaration::Trait(trait_decl) => {
                self.traits.insert(trait_decl.name.clone(), trait_decl.clone());
            }
            Declaration::Enum(enum_decl) => {
                self.enums.insert(enum_decl.name.clone(), enum_decl.clone());
            }
            // 处理类型定义
            Declaration::TypeAlias(_) => {
                // 类型别名暂时不处理
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_block_expr(&mut self, block: &Block) -> Result<ControlFlow, InterpreterError> {
        let mut last_expr_result = None;
        for stmt in &block.statements {
            if let StatementKind::Expression(expr) = &stmt.node {
                last_expr_result = Some(self.eval(expr)?);
                continue;
            }

            match self.execute_statement(stmt)? {
                ControlFlow::None => {}
                cf => return Ok(cf),
            }
        }
        // 如果最后一个语句是表达式语句，返回其结果
        if let Some(result) = last_expr_result {
            Ok(ControlFlow::Return(result))
        } else {
            Ok(ControlFlow::None)
        }
    }

    fn execute_block_stmt(&mut self, block: &Block) -> Result<ControlFlow, InterpreterError> {
        for stmt in &block.statements {
            if let StatementKind::Expression(expr) = &stmt.node {
                self.eval(expr)?;
                continue;
            }
            match self.execute_statement(stmt)? {
                ControlFlow::None => {}
                cf => return Ok(cf),
            }
        }
        Ok(ControlFlow::None)
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<ControlFlow, InterpreterError> {
        match &stmt.node {
            StatementKind::Variable(var) => {
                let val = if let Some(init) = &var.initializer {
                    self.eval(init)?
                } else {
                    Value::Null
                };
                // 忽略类型注解，因为解释器暂时不支持类型检查
                self.variables.insert(var.name.clone(), val);
                Ok(ControlFlow::None)
            }
            StatementKind::Expression(expr) => {
                self.eval(expr)?;
                Ok(ControlFlow::None)
            }
            StatementKind::Return(Some(expr)) => {
                let val = self.eval(expr)?;
                Ok(ControlFlow::Return(val))
            }
            StatementKind::Return(std::option::Option::None) => Ok(ControlFlow::Return(Value::Unit)),
            StatementKind::If(if_stmt) => {
                let cond = self.eval(&if_stmt.condition)?;
                if self.is_truthy(&cond) {
                    self.execute_block_stmt(&if_stmt.then_block)
                } else if let Some(else_blk) = &if_stmt.else_block {
                    self.execute_block_stmt(else_blk)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            StatementKind::While(while_stmt) => loop {
                let cond = self.eval(&while_stmt.condition)?;
                if !self.is_truthy(&cond) {
                    break Ok(ControlFlow::None);
                }
                match self.execute_block_stmt(&while_stmt.body)? {
                    ControlFlow::Return(v) => break Ok(ControlFlow::Return(v)),
                    ControlFlow::Break => break Ok(ControlFlow::None),
                    _ => {}
                }
            },
            StatementKind::For(for_stmt) => {
                // 暂时实现简单的范围循环
                let iterator = self.eval(&for_stmt.iterator)?;
                match iterator {
                    Value::Array(arr) => {
                        for item in arr.borrow().iter() {
                            // 绑定循环变量
                            if let x_parser::ast::Pattern::Variable(name) = &for_stmt.pattern {
                                self.variables.insert(name.clone(), item.clone());
                            }
                            // 执行循环体
                            match self.execute_block_stmt(&for_stmt.body)? {
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                ControlFlow::Break => break,
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        // 暂时不支持其他类型的迭代器
                        return Err(InterpreterError::runtime_no_span(
                            "For循环只支持数组迭代",
                        ));
                    }
                }
                Ok(ControlFlow::None)
            }
            StatementKind::Match(match_stmt) => self.execute_match(match_stmt),
            StatementKind::Try(try_stmt) => self.execute_try(try_stmt),
            StatementKind::Break => Ok(ControlFlow::Break),
            StatementKind::Continue => Ok(ControlFlow::Continue),
            StatementKind::DoWhile(d) => self.execute_do_while(d),
            StatementKind::Unsafe(block) => {
                // Execute unsafe block (interpreter doesn't enforce safety)
                self.execute_block_stmt(block)
            }
        }
    }

    fn execute_do_while(
        &mut self,
        d: &x_parser::ast::DoWhileStatement,
    ) -> Result<ControlFlow, InterpreterError> {
        loop {
            match self.execute_block_stmt(&d.body)? {
                ControlFlow::Break => break,
                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                _ => {}
            }
            let cond = self.eval(&d.condition)?;
            if !self.is_truthy(&cond) {
                break;
            }
        }
        Ok(ControlFlow::None)
    }

    fn execute_match(&mut self, match_stmt: &MatchStatement) -> Result<ControlFlow, InterpreterError> {
        let value = self.eval(&match_stmt.expression)?;

        let base_vars = self.variables.clone();
        for case in &match_stmt.cases {
            self.variables = base_vars.clone();
            if self.match_case(&value, case)? {
                return self.execute_block_stmt(&case.body);
            }
        }

        self.variables = base_vars;
        Ok(ControlFlow::None)
    }

    fn match_case(&mut self, value: &Value, case: &MatchCase) -> Result<bool, InterpreterError> {
        let mut bindings = HashMap::<String, Value>::new();
        if !self.match_pattern(&case.pattern, value, &mut bindings)? {
            return Ok(false);
        }

        let saved = self.variables.clone();
        for (k, v) in bindings {
            self.variables.insert(k, v);
        }

        let guard_ok = match &case.guard {
            Some(guard_expr) => {
                let gv = self.eval(guard_expr)?;
                self.is_truthy(&gv)
            }
            None => true,
        };

        if !guard_ok {
            self.variables = saved;
        }
        Ok(guard_ok)
    }

    fn match_pattern(
        &mut self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<bool, InterpreterError> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Variable(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(true)
            }
            Pattern::Literal(lit) => Ok(self.eval_literal(lit) == *value),
            Pattern::Or(a, b) => {
                let mut left = bindings.clone();
                if self.match_pattern(a, value, &mut left)? {
                    *bindings = left;
                    return Ok(true);
                }
                self.match_pattern(b, value, bindings)
            }
            Pattern::Guard(inner, guard) => {
                let mut inner_bindings = bindings.clone();
                if !self.match_pattern(inner, value, &mut inner_bindings)? {
                    return Ok(false);
                }
                let saved = self.variables.clone();
                for (k, v) in inner_bindings.iter() {
                    self.variables.insert(k.clone(), v.clone());
                }
                let gv = self.eval(guard)?;
                let ok = self.is_truthy(&gv);
                self.variables = saved;
                if ok {
                    *bindings = inner_bindings;
                }
                Ok(ok)
            }
            Pattern::Array(items) => match value {
                Value::Array(arr) => {
                    let v = arr.borrow();
                    if v.len() != items.len() {
                        return Ok(false);
                    }
                    for (p, item) in items.iter().zip(v.iter()) {
                        if !self.match_pattern(p, item, bindings)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },
            Pattern::Tuple(items) => match value {
                Value::Array(arr) => {
                    let v = arr.borrow();
                    if v.len() != items.len() {
                        return Ok(false);
                    }
                    for (p, item) in items.iter().zip(v.iter()) {
                        if !self.match_pattern(p, item, bindings)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },
            Pattern::Dictionary(_) | Pattern::Record(_, _) => Ok(false),
            Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                // 匹配枚举值
                match value {
                    Value::Enum(enum_name, enum_variant, enum_values) => {
                        // 检查变体名是否匹配
                        if variant_name != enum_variant {
                            return Ok(false);
                        }
                        // 检查模式数量是否匹配
                        if patterns.len() != enum_values.len() {
                            return Ok(false);
                        }
                        // 递归匹配每个子模式
                        for (p, v) in patterns.iter().zip(enum_values.iter()) {
                            if !self.match_pattern(p, v, bindings)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
        }
    }

    fn execute_try(&mut self, try_stmt: &TryStatement) -> Result<ControlFlow, InterpreterError> {
        let base_vars = self.variables.clone();
        let body_result = self.execute_block_stmt(&try_stmt.body);

        let mut result = match body_result {
            Ok(cf) => Ok(cf),
            Err(err) => self.handle_catches(&try_stmt.catch_clauses, err, &base_vars),
        };

        if let Some(finally_block) = &try_stmt.finally_block {
            self.variables = base_vars.clone();
            let finally_result = self.execute_block_stmt(finally_block);
            if let Err(e) = finally_result {
                result = Err(e);
            }
        }

        self.variables = base_vars;
        result
    }

    fn handle_catches(
        &mut self,
        catches: &[CatchClause],
        err: InterpreterError,
        base_vars: &HashMap<String, Value>,
    ) -> Result<ControlFlow, InterpreterError> {
        for clause in catches {
            self.variables = base_vars.clone();
            if let Some(var) = &clause.variable_name {
                self.variables.insert(var.clone(), Value::String(err.to_string()));
            }
            // 目前不区分 exception_type；未来可根据类型做筛选
            return self.execute_block_stmt(&clause.body);
        }
        Err(err)
    }

    fn is_truthy(&self, v: &Value) -> bool {
        match v {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Null | Value::None | Value::Unit => false,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    fn eval(&mut self, expr: &Expression) -> Result<Value, InterpreterError> {
        match &expr.node {
            ExpressionKind::Literal(lit) => Ok(self.eval_literal(lit)),
            ExpressionKind::Variable(name) => {
                self.variables.get(name).cloned().ok_or_else(|| {
                    InterpreterError::runtime_no_span(format!("未定义的变量: {}", name))
                })
            }
            ExpressionKind::Binary(op, l, r) => {
                if matches!(op, BinaryOp::And) {
                    let lv = self.eval(l)?;
                    return if !self.is_truthy(&lv) {
                        Ok(lv)
                    } else {
                        self.eval(r)
                    };
                }
                if matches!(op, BinaryOp::Or) {
                    let lv = self.eval(l)?;
                    return if self.is_truthy(&lv) {
                        Ok(lv)
                    } else {
                        self.eval(r)
                    };
                }
                let lv = self.eval(l)?;
                let rv = self.eval(r)?;
                self.eval_binary(op.clone(), &lv, &rv)
            }
            ExpressionKind::Unary(op, operand) => {
                let v = self.eval(operand)?;
                match op {
                    UnaryOp::Negate => match v {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(InterpreterError::runtime_no_span("- 需要数字")),
                    },
                    UnaryOp::Not => Ok(Value::Boolean(!self.is_truthy(&v))),
                    UnaryOp::Wait => {
                        // Wait unary operator: in sync interpreter, just return the value
                        Ok(v)
                    }
                    UnaryOp::BitNot => match v {
                        Value::Integer(n) => Ok(Value::Integer(!n)),
                        _ => Err(InterpreterError::runtime_no_span("~ 需要整数")),
                    },
                }
            }
            ExpressionKind::Assign(target, value) => {
                let val = self.eval(value)?;
                self.do_assign(target, val)
            }
            ExpressionKind::Call(callee, args) => {
                if let ExpressionKind::Variable(name) = &callee.node {
                    if name == "__index__" {
                        return self.eval_index(args);
                    }
                    // 检查是否是类构造函数调用
                    if self.classes.contains_key(name) {
                        return self.instantiate_class(name, args);
                    }
                    // 检查是否是类型构造器调用（如 Pointer(Void), Option(Int) 等）
                    if let Some(type_val) = self.try_type_constructor(name, args)? {
                        return Ok(type_val);
                    }
                    return self.call_function(name, args);
                }
                // 检查是否是枚举构造器调用 Option.Some(value)
                if let ExpressionKind::Member(obj, variant_name) = &callee.node {
                    if let ExpressionKind::Variable(enum_name) = &obj.node {
                        if self.enums.contains_key(enum_name) {
                            // 枚举构造器调用
                            let arg_vals: Vec<Value> = args
                                .iter()
                                .map(|a| self.eval(a))
                                .collect::<Result<_, _>>()?;
                            return Ok(Value::Enum(enum_name.clone(), variant_name.clone(), arg_vals));
                        }
                    }
                    // 普通方法调用
                    return self.call_method(obj, variant_name, args);
                }
                // 检查是否是方法调用
                if let ExpressionKind::Member(obj, method_name) = &callee.node {
                    return self.call_method(obj, method_name, args);
                }
                // 检查是否是闭包直接调用（例如：(fn)(args) 或返回闭包的函数调用）
                let callee_val = self.eval(callee)?;
                if let Value::Closure { params, body, captured } = callee_val {
                    let arg_vals: Vec<Value> = args
                        .iter()
                        .map(|a| self.eval(a))
                        .collect::<Result<_, _>>()?;
                    if arg_vals.len() != params.len() {
                        return Err(InterpreterError::runtime_no_span(format!(
                            "闭包期望 {} 个参数，得到 {}",
                            params.len(),
                            arg_vals.len()
                        )));
                    }
                    // 保存当前变量状态
                    let saved = self.variables.clone();
                    // 添加捕获的变量
                    for (k, v) in captured.borrow().iter() {
                        self.variables.insert(k.clone(), v.clone());
                    }
                    // 添加参数
                    for (p, v) in params.iter().zip(arg_vals) {
                        self.variables.insert(p.clone(), v);
                    }
                    let result = self.execute_block_expr(&body)?;
                    // 恢复变量状态
                    self.variables = saved;
                    match result {
                        ControlFlow::Return(v) => Ok(v),
                        _ => Ok(Value::Unit),
                    }
                } else {
                    Err(InterpreterError::runtime_no_span(format!(
                        "不能调用非函数值: {:?}",
                        callee_val
                    )))
                }
            }
            ExpressionKind::Array(elems) => {
                let vals: Vec<Value> = elems
                    .iter()
                    .map(|e| self.eval(e))
                    .collect::<Result<_, _>>()?;
                Ok(Value::new_array(vals))
            }
            ExpressionKind::Dictionary(entries) => {
                let map = Value::new_map();
                for (key_expr, val_expr) in entries {
                    let key = self.eval(key_expr)?;
                    let key_str = match &key {
                        Value::String(s) => s.clone(),
                        _ => self.format_value(&key),
                    };
                    let val = self.eval(val_expr)?;
                    if let Value::Map(rc) = &map {
                        rc.borrow_mut().push((key_str, val));
                    }
                }
                Ok(map)
            }
            ExpressionKind::Record(_name, _fields) => {
                // 处理记录表达式，暂时创建一个映射来存储字段
                let map = Value::new_map();
                // 暂时直接返回一个空映射，避免栈溢出
                Ok(map)
            }
            ExpressionKind::Parenthesized(inner) => self.eval(inner),
            ExpressionKind::Member(obj, member) => {
                let obj_val = self.eval(obj)?;
                match &obj_val {
                    Value::Object { fields, .. } => {
                        fields.borrow().get(member).cloned().ok_or_else(|| {
                            InterpreterError::runtime_no_span(format!("未定义的字段: {}", member))
                        })
                    }
                    Value::Map(entries) => {
                        let entries = entries.borrow();
                        for (k, v) in entries.iter() {
                            if k == member {
                                return Ok(v.clone());
                            }
                        }
                        Err(InterpreterError::runtime_no_span(format!("未定义的键: {}", member)))
                    }
                    _ => Err(InterpreterError::runtime_no_span("只能访问对象的成员")),
                }
            }
            ExpressionKind::If(cond, then_expr, else_expr) => {
                let cond_val = self.eval(cond)?;
                if self.is_truthy(&cond_val) {
                    self.eval(then_expr)
                } else {
                    self.eval(else_expr)
                }
            }
            ExpressionKind::Range(start, end, inclusive) => {
                let start_val = self.eval(start)?;
                let end_val = self.eval(end)?;

                let start_int = self.as_i64(&start_val)?;
                let end_int = self.as_i64(&end_val)?;

                let mut values = Vec::new();
                if *inclusive {
                    for i in start_int..=end_int {
                        values.push(Value::Integer(i));
                    }
                } else {
                    for i in start_int..end_int {
                        values.push(Value::Integer(i));
                    }
                }

                Ok(Value::new_array(values))
            }
            ExpressionKind::Pipe(input, functions) => {
                let mut value = self.eval(input)?;
                for func in functions {
                    // 暂时只支持调用命名函数
                    if let ExpressionKind::Variable(name) = &func.node {
                        // 直接调用函数，传递值作为参数
                        // 创建一个表达式来表示当前值
                        let temp_expr = Spanned::new(
                            ExpressionKind::Literal(match value {
                                Value::Integer(i) => Literal::Integer(i),
                                Value::Float(f) => Literal::Float(f),
                                Value::Boolean(b) => Literal::Boolean(b),
                                Value::String(s) => Literal::String(s),
                                _ => {
                                    return Err(InterpreterError::runtime_no_span(
                                        "管道操作符只支持基本类型",
                                    ))
                                }
                            }),
                            Span::default(),
                        );
                        // 调用函数，传递临时表达式作为参数
                        value = self.call_function(name, &[temp_expr])?;
                    } else {
                        return Err(InterpreterError::runtime_no_span(
                            "管道操作符只支持调用命名函数",
                        ));
                    }
                }
                Ok(value)
            }
            // Wait expressions: in the synchronous interpreter, we just evaluate the inner expression
            ExpressionKind::Wait(_wait_type, exprs) => {
                // For the synchronous interpreter, wait just evaluates the expressions
                // In a real async runtime, this would await futures
                if exprs.is_empty() {
                    return Ok(Value::Unit);
                }
                // For single wait, return the value of the expression
                // For together/race/timeout, return the last value (simplified behavior)
                let mut result = Value::Unit;
                for expr in exprs {
                    result = self.eval(expr)?;
                }
                Ok(result)
            }
            ExpressionKind::Needs(_name) => {
                // Effect requirement - in interpreter, just ignore
                Ok(Value::Unit)
            }
            ExpressionKind::Given(_name, value) => {
                // Effect handler - evaluate the value
                self.eval(value)
            }
            ExpressionKind::Match(discriminant, cases) => {
                // Evaluate the discriminant value, then match against patterns
                // TODO: full pattern matching with bindings
                // For now, simple matching on enum variants works in the prelude functions
                let discrim_val = self.eval(discriminant)?;

                // Iterate through cases to find a match
                // Since all cases are in order, first match wins
                for case in cases {
                    // For simple matching like in prelude: pattern is exactly the variant
                    // We just execute the first case body (all cases are exhaustive)
                    // TODO: proper pattern matching with binding
                    for stmt in &case.body.statements {
                        match stmt.node {
                            StatementKind::Expression(ref expr) => {
                                // Return the expression value from the matching case
                                return self.eval(expr);
                            }
                            _ => {
                                self.execute_statement(stmt)?;
                            }
                        }
                    }
                }

                // If no case matched (shouldn't happen with exhaustive matching)
                Ok(Value::Unit)
            }
            ExpressionKind::Lambda(params, body) => {
                // 创建闭包：收集参数名和捕获的变量
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();

                // 收集当前作用域中的所有变量作为捕获的环境
                // 注意：实际使用时，只有函数体内引用的外部变量才会被访问
                let captured = Rc::new(RefCell::new(self.variables.clone()));

                Ok(Value::Closure {
                    params: param_names,
                    body: body.clone(),
                    captured,
                })
            }
            _ => Err(InterpreterError::runtime_no_span(format!(
                "未实现的表达式类型: {:?}",
                expr
            ))),
        }
    }

    fn do_assign(&mut self, target: &Expression, val: Value) -> Result<Value, InterpreterError> {
        match &target.node {
            ExpressionKind::Variable(name) => {
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }
            ExpressionKind::Member(obj, member) => {
                let obj_val = self.eval(obj)?;
                match &obj_val {
                    Value::Object { fields, .. } => {
                        fields.borrow_mut().insert(member.clone(), val.clone());
                        Ok(val)
                    }
                    Value::Map(entries) => {
                        // 查找并更新或插入
                        let mut entries = entries.borrow_mut();
                        for (k, v) in entries.iter_mut() {
                            if k == member {
                                *v = val.clone();
                                return Ok(val);
                            }
                        }
                        entries.push((member.clone(), val.clone()));
                        Ok(val)
                    }
                    _ => Err(InterpreterError::runtime_no_span("只能对对象或映射进行字段赋值")),
                }
            }
            ExpressionKind::Call(func, args)
                if matches!(&func.node, ExpressionKind::Variable(n) if n == "__index__") =>
            {
                if args.len() == 2 {
                    let container = self.eval(&args[0])?;
                    let idx = self.eval(&args[1])?;
                    match (&container, &idx) {
                        (Value::Array(rc), Value::Integer(i)) => {
                            let i = *i as usize;
                            let mut arr = rc.borrow_mut();
                            if i < arr.len() {
                                arr[i] = val.clone();
                                return Ok(val);
                            }
                            return Err(InterpreterError::runtime_no_span(format!(
                                "数组索引越界: {} (长度 {})",
                                i,
                                arr.len()
                            )));
                        }
                        _ => {}
                    }
                }
                Err(InterpreterError::runtime_no_span("无效的赋值目标"))
            }
            _ => Err(InterpreterError::runtime_no_span("无效的赋值目标")),
        }
    }

    fn eval_index(&mut self, args: &[Expression]) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::runtime_no_span("索引需要2个参数"));
        }
        let obj = self.eval(&args[0])?;
        let idx = self.eval(&args[1])?;
        match (&obj, &idx) {
            (Value::Array(rc), Value::Integer(i)) => {
                let i = *i as usize;
                let arr = rc.borrow();
                arr.get(i).cloned().ok_or_else(|| {
                    InterpreterError::runtime_no_span(format!(
                        "数组索引越界: {} (长度 {})",
                        i,
                        arr.len()
                    ))
                })
            }
            (Value::String(s), Value::Integer(i)) => {
                let i = *i as usize;
                s.chars()
                    .nth(i)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| InterpreterError::runtime_no_span("字符串索引越界"))
            }
            (Value::Map(rc), Value::String(key)) => {
                let entries = rc.borrow();
                for (k, v) in entries.iter() {
                    if k == key {
                        return Ok(v.clone());
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(InterpreterError::runtime_no_span(format!(
                "不支持的索引操作: {:?}[{:?}]",
                obj, idx
            ))),
        }
    }

    /// 尝试将名称解析为类型构造器
    /// 返回 Some(Value) 如果是类型构造器，返回 None 如果不是
    fn try_type_constructor(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<Option<Value>, InterpreterError> {
        // 支持的类型构造器
        match name {
            "Pointer" | "pointer" => {
                // Pointer(T) 构造一个指针类型值
                if args.len() != 1 {
                    return Err(InterpreterError::runtime_no_span(format!(
                        "Pointer 类型需要一个类型参数，得到 {}",
                        args.len()
                    )));
                }
                let inner_type = self.eval_type_arg(&args[0])?;
                return Ok(Some(Value::Type {
                    name: "Pointer".to_string(),
                    args: vec![inner_type],
                }));
            }
            "Void" | "void" => {
                return Ok(Some(Value::Type {
                    name: "Void".to_string(),
                    args: vec![],
                }));
            }
            _ => return Ok(None),
        }
    }

    /// 将表达式解析为类型值
    fn eval_type_arg(&mut self, expr: &Expression) -> Result<TypeValue, InterpreterError> {
        match &expr.node {
            ExpressionKind::Variable(name) => {
                match name.as_str() {
                    "Void" | "void" => Ok(TypeValue::Void),
                    "CLong" | "c_long" => Ok(TypeValue::Named("CLong".to_string())),
                    "Int" | "Int64" | "i64" => Ok(TypeValue::Named("Int64".to_string())),
                    other => Ok(TypeValue::Named(other.to_string())),
                }
            }
            ExpressionKind::Call(callee, inner_args) => {
                if let ExpressionKind::Variable(name) = &callee.node {
                    if name == "Pointer" || name == "pointer" {
                        if inner_args.len() != 1 {
                            return Err(InterpreterError::runtime_no_span(
                                "Pointer 类型需要一个类型参数".to_string(),
                            ));
                        }
                        let inner = self.eval_type_arg(&inner_args[0])?;
                        return Ok(TypeValue::Pointer(Box::new(inner)));
                    }
                }
                Err(InterpreterError::runtime_no_span(format!(
                    "不支持作为类型参数: {:?}",
                    expr
                )))
            }
            ExpressionKind::Match(..) => {
                Err(InterpreterError::runtime_no_span(format!(
                    "不支持作为类型参数: {:?}",
                    expr
                )))
            }
            _ => Err(InterpreterError::runtime_no_span(format!(
                "不支持作为类型参数: {:?}",
                expr
            ))),
        }
    }

    fn call_function(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<Value, InterpreterError> {
        match name {
            "Some" => {
                let value = self.eval(&args[0])?;
                Ok(Value::Option(Box::new(value)))
            }
            "None" => Ok(Value::None),
            "Ok" => {
                let value = self.eval(&args[0])?;
                Ok(Value::Result(Box::new(value), Box::new(Value::Null)))
            }
            "Err" => {
                let error = self.eval(&args[0])?;
                Ok(Value::Result(Box::new(Value::Null), Box::new(error)))
            }
            "print" => {
                let mut parts = Vec::new();
                for a in args {
                    let v = self.eval(a)?;
                    parts.push(self.format_value(&v));
                }
                println!("{}", parts.join(" "));
                Ok(Value::Unit)
            }
            "print_inline" => {
                for a in args {
                    let v = self.eval(a)?;
                    print!("{}", self.format_value(&v));
                }
                Ok(Value::Unit)
            }
            "println" => {
                if args.is_empty() {
                    println!();
                } else {
                    let mut parts = Vec::new();
                    for a in args {
                        let v = self.eval(a)?;
                        parts.push(self.format_value(&v));
                    }
                    println!("{}", parts.join(" "));
                }
                Ok(Value::Unit)
            }
            "len" => {
                let v = self.eval(&args[0])?;
                match &v {
                    Value::Array(rc) => Ok(Value::Integer(rc.borrow().len() as i64)),
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::Map(rc) => Ok(Value::Integer(rc.borrow().len() as i64)),
                    _ => Err(InterpreterError::runtime_no_span(
                        "len 需要数组/字符串/映射",
                    )),
                }
            }
            "string_length" => {
                let s = self.eval_as_string(&args[0])?;
                Ok(Value::Integer(s.chars().count() as i64))
            }
            "string_find" => {
                let s = self.eval_as_string(&args[0])?;
                let substr = self.eval_as_string(&args[1])?;
                match s.find(&substr) {
                    Some(idx) => Ok(Value::Integer(idx as i64)),
                    None => Ok(Value::Integer(-1)),
                }
            }
            "string_substring" => {
                let s = self.eval_as_string(&args[0])?;
                let start_v = self.eval(&args[1])?;
                let end_v = self.eval(&args[2])?;
                let start = self.as_i64(&start_v)? as usize;
                let end = self.as_i64(&end_v)? as usize;
                let chars: Vec<char> = s.chars().collect();
                if start > chars.len() || end > chars.len() || start > end {
                    Ok(Value::String(String::new()))
                } else {
                    Ok(Value::String(chars[start..end].iter().collect()))
                }
            }
            "int_to_string" => {
                let v = self.eval(&args[0])?;
                let n = self.as_i64(&v)?;
                Ok(Value::String(n.to_string()))
            }
            "push" => {
                let container = self.eval(&args[0])?;
                let val = self.eval(&args[1])?;
                match &container {
                    Value::Array(rc) => {
                        rc.borrow_mut().push(val);
                        Ok(Value::Unit)
                    }
                    _ => Err(InterpreterError::runtime_no_span("push 需要数组")),
                }
            }
            "pop" => {
                let container = self.eval(&args[0])?;
                match &container {
                    Value::Array(rc) => Ok(rc.borrow_mut().pop().unwrap_or(Value::Null)),
                    _ => Err(InterpreterError::runtime_no_span("pop 需要数组")),
                }
            }
            "new_array" => {
                let v = self.eval(&args[0])?;
                let size = self.as_i64(&v)?;
                let init = if args.len() > 1 {
                    self.eval(&args[1])?
                } else {
                    Value::Integer(0)
                };
                Ok(Value::new_array(vec![init; size as usize]))
            }
            "new_map" => Ok(Value::new_map()),
            "map_set" => {
                let map = self.eval(&args[0])?;
                let key_v = self.eval(&args[1])?;
                let key = self.format_value(&key_v);
                let val = self.eval(&args[2])?;
                match &map {
                    Value::Map(rc) => {
                        let mut entries = rc.borrow_mut();
                        for entry in entries.iter_mut() {
                            if entry.0 == key {
                                entry.1 = val;
                                return Ok(Value::Unit);
                            }
                        }
                        entries.push((key, val));
                        Ok(Value::Unit)
                    }
                    _ => Err(InterpreterError::runtime_no_span("map_set 需要映射")),
                }
            }
            "map_get" => {
                let map = self.eval(&args[0])?;
                let key_v = self.eval(&args[1])?;
                let key = self.format_value(&key_v);
                match &map {
                    Value::Map(rc) => {
                        let entries = rc.borrow();
                        for (k, v) in entries.iter() {
                            if *k == key {
                                return Ok(v.clone());
                            }
                        }
                        Ok(Value::Integer(0))
                    }
                    _ => Ok(Value::Integer(0)),
                }
            }
            "map_contains" => {
                let map = self.eval(&args[0])?;
                let key_v = self.eval(&args[1])?;
                let key = self.format_value(&key_v);
                match &map {
                    Value::Map(rc) => {
                        let entries = rc.borrow();
                        for (k, _) in entries.iter() {
                            if *k == key {
                                return Ok(Value::Boolean(true));
                            }
                        }
                        Ok(Value::Boolean(false))
                    }
                    _ => Ok(Value::Boolean(false)),
                }
            }
            "map_keys" => {
                let map = self.eval(&args[0])?;
                match &map {
                    Value::Map(rc) => {
                        let entries = rc.borrow();
                        let keys: Vec<Value> = entries
                            .iter()
                            .map(|(k, _)| Value::String(k.clone()))
                            .collect();
                        Ok(Value::new_array(keys))
                    }
                    _ => Err(InterpreterError::runtime_no_span("map_keys 需要映射")),
                }
            }
            "to_string" => {
                let v = self.eval(&args[0])?;
                Ok(Value::String(self.format_value(&v)))
            }
            "to_int" => {
                let v = self.eval(&args[0])?;
                match v {
                    Value::Integer(n) => Ok(Value::Integer(n)),
                    Value::Float(f) => Ok(Value::Integer(f as i64)),
                    Value::String(s) => s.trim().parse::<i64>().map(Value::Integer).map_err(|_| {
                        InterpreterError::runtime_no_span(format!("无法转换为整数: {}", s))
                    }),
                    Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
                    _ => Err(InterpreterError::runtime_no_span("无法转换为整数")),
                }
            }
            "to_float" => {
                let v = self.eval(&args[0])?;
                match v {
                    Value::Float(f) => Ok(Value::Float(f)),
                    Value::Integer(n) => Ok(Value::Float(n as f64)),
                    Value::String(s) => s.trim().parse::<f64>().map(Value::Float).map_err(|_| {
                        InterpreterError::runtime_no_span(format!("无法转换为浮点: {}", s))
                    }),
                    _ => Err(InterpreterError::runtime_no_span("无法转换为浮点")),
                }
            }
            "sqrt" => {
                let v = self.eval(&args[0])?;
                let f = self.as_f64(&v)?;
                Ok(Value::Float(f.sqrt()))
            }
            "abs" => {
                let v = self.eval(&args[0])?;
                match v {
                    Value::Integer(n) => Ok(Value::Integer(n.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err(InterpreterError::runtime_no_span("abs 需要数字")),
                }
            }
            "floor" => {
                let v = self.eval(&args[0])?;
                Ok(Value::Float(self.as_f64(&v)?.floor()))
            }
            "ceil" => {
                let v = self.eval(&args[0])?;
                Ok(Value::Float(self.as_f64(&v)?.ceil()))
            }
            "round" => {
                let v = self.eval(&args[0])?;
                Ok(Value::Float(self.as_f64(&v)?.round()))
            }
            "sin" => {
                let v = self.eval(&args[0])?;
                Ok(Value::Float(self.as_f64(&v)?.sin()))
            }
            "cos" => {
                let v = self.eval(&args[0])?;
                Ok(Value::Float(self.as_f64(&v)?.cos()))
            }
            "pow" => {
                let base_v = self.eval(&args[0])?;
                let exp_v = self.eval(&args[1])?;
                Ok(Value::Float(
                    self.as_f64(&base_v)?.powf(self.as_f64(&exp_v)?),
                ))
            }
            "concat" => {
                let mut result = String::new();
                for a in args {
                    let v = self.eval(a)?;
                    result.push_str(&self.format_value(&v));
                }
                Ok(Value::String(result))
            }
            "char_at" => {
                let s = self.eval_as_string(&args[0])?;
                let idx_v = self.eval(&args[1])?;
                let i = self.as_i64(&idx_v)? as usize;
                s.chars()
                    .nth(i)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| InterpreterError::runtime_no_span("char_at 索引越界"))
            }
            "substring" => {
                let s = self.eval_as_string(&args[0])?;
                let start_v = self.eval(&args[1])?;
                let start = self.as_i64(&start_v)? as usize;
                let end = if args.len() > 2 {
                    let end_v = self.eval(&args[2])?;
                    self.as_i64(&end_v)? as usize
                } else {
                    s.len()
                };
                let chars: Vec<char> = s.chars().collect();
                let s = start.min(chars.len());
                let e = end.min(chars.len());
                Ok(Value::String(chars[s..e].iter().collect()))
            }
            "str_upper" => {
                let s = self.eval_as_string(&args[0])?;
                Ok(Value::String(s.to_uppercase()))
            }
            "str_lower" => {
                let s = self.eval_as_string(&args[0])?;
                Ok(Value::String(s.to_lowercase()))
            }
            "str_contains" => {
                let s = self.eval_as_string(&args[0])?;
                let pat = self.eval_as_string(&args[1])?;
                Ok(Value::Boolean(s.contains(&pat)))
            }
            "str_replace" => {
                let s = self.eval_as_string(&args[0])?;
                let from = self.eval_as_string(&args[1])?;
                let to = self.eval_as_string(&args[2])?;
                Ok(Value::String(s.replace(&from, &to)))
            }
            "str_split" => {
                let s = self.eval_as_string(&args[0])?;
                let delim = self.eval_as_string(&args[1])?;
                let parts: Vec<Value> = s
                    .split(&delim)
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::new_array(parts))
            }
            "str_trim" => {
                let s = self.eval_as_string(&args[0])?;
                Ok(Value::String(s.trim().to_string()))
            }
            "str_starts_with" => {
                let s = self.eval_as_string(&args[0])?;
                let prefix = self.eval_as_string(&args[1])?;
                Ok(Value::Boolean(s.starts_with(&prefix)))
            }
            "str_find" => {
                let s = self.eval_as_string(&args[0])?;
                let pat = self.eval_as_string(&args[1])?;
                match s.find(&pat) {
                    Some(i) => Ok(Value::Integer(i as i64)),
                    std::option::Option::None => Ok(Value::Integer(-1)),
                }
            }
            "regex_match_count" => {
                let text = self.eval_as_string(&args[0])?;
                let pattern = self.eval_as_string(&args[1])?;
                Ok(Value::Integer(simple_regex_count(&text, &pattern) as i64))
            }
            "regex_replace_all" => {
                let text = self.eval_as_string(&args[0])?;
                let pattern = self.eval_as_string(&args[1])?;
                let replacement = self.eval_as_string(&args[2])?;
                Ok(Value::String(simple_regex_replace(
                    &text,
                    &pattern,
                    &replacement,
                )))
            }
            "format_float" => {
                let v = self.eval(&args[0])?;
                let f = self.as_f64(&v)?;
                let precision = if args.len() > 1 {
                    let prec_v = self.eval(&args[1])?;
                    self.as_i64(&prec_v)? as usize
                } else {
                    9
                };
                Ok(Value::String(format!("{:.prec$}", f, prec = precision)))
            }
            "compute_pi_digits" => {
                let v = self.eval(&args[0])?;
                let n = self.as_i64(&v)? as usize;
                let digits = compute_pi_digits_bigint(n);
                Ok(Value::String(digits))
            }
            "type_of" => {
                let v = self.eval(&args[0])?;
                let t = match &v {
                    Value::Integer(_) => "Int".to_string(),
                    Value::Float(_) => "Float".to_string(),
                    Value::Boolean(_) => "Bool".to_string(),
                    Value::String(_) => "String".to_string(),
                    Value::Array(_) => "Array".to_string(),
                    Value::Map(_) => "Map".to_string(),
                    Value::Char(_) => "Char".to_string(),
                    Value::Null => "Null".to_string(),
                    Value::None => "None".to_string(),
                    Value::Unit => "Unit".to_string(),
                    Value::Option(_) => "Option".to_string(),
                    Value::Result(_, _) => "Result".to_string(),
                    Value::Object { class_name, .. } => class_name.clone(),
                    Value::Closure { .. } => "Closure".to_string(),
                    Value::Enum(type_name, _, _) => type_name.clone(),
                    Value::Type { name, .. } => format!("Type({})", name),
                    Value::Pointer(_) => "Pointer".to_string(),
                };
                Ok(Value::String(t))
            }
            "copy_array" => {
                let v = self.eval(&args[0])?;
                Ok(v.deep_clone())
            }
            "swap" => {
                let container = self.eval(&args[0])?;
                let i_v = self.eval(&args[1])?;
                let j_v = self.eval(&args[2])?;
                let i = self.as_i64(&i_v)? as usize;
                let j = self.as_i64(&j_v)? as usize;
                match &container {
                    Value::Array(rc) => {
                        let mut arr = rc.borrow_mut();
                        if i < arr.len() && j < arr.len() {
                            arr.swap(i, j);
                            return Ok(Value::Unit);
                        }
                        Err(InterpreterError::runtime_no_span("swap 索引越界"))
                    }
                    _ => Err(InterpreterError::runtime_no_span("swap 需要数组")),
                }
            }
            "reverse_range" => {
                let container = self.eval(&args[0])?;
                let start_v = self.eval(&args[1])?;
                let end_v = self.eval(&args[2])?;
                let start = self.as_i64(&start_v)? as usize;
                let end = self.as_i64(&end_v)? as usize;
                match &container {
                    Value::Array(rc) => {
                        let mut arr = rc.borrow_mut();
                        if end <= arr.len() {
                            arr[start..end].reverse();
                            return Ok(Value::Unit);
                        }
                        Err(InterpreterError::runtime_no_span(
                            "reverse_range 范围越界",
                        ))
                    }
                    _ => Err(InterpreterError::runtime_no_span(
                        "reverse_range 需要数组",
                    )),
                }
            }
            "sort_by_value_desc" => {
                let container = self.eval(&args[0])?;
                match &container {
                    Value::Array(rc) => {
                        let mut arr = rc.borrow_mut();
                        arr.sort_by(|a, b| {
                            let va = extract_sort_key(a);
                            let vb = extract_sort_key(b);
                            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        Ok(Value::Unit)
                    }
                    _ => Err(InterpreterError::runtime_no_span("sort 需要数组")),
                }
            }
            "x_to_json" => {
                let v = self.eval(&args[0])?;
                Ok(Value::String(self.value_to_json(&v)))
            }
            "x_json_parse" => {
                let json_str = self.eval_as_string(&args[0])?;
                self.json_to_value(&json_str)
            }
            // File I/O builtins
            "__file_read" => {
                let path = self.eval_as_string(&args[0])?;
                match std::fs::read_to_string(&path) {
                    Ok(content) => Ok(Value::Result(
                        Box::new(Value::String(content)),
                        Box::new(Value::Null),
                    )),
                    Err(e) => Ok(Value::Result(
                        Box::new(Value::Null),
                        Box::new(Value::String(e.to_string())),
                    )),
                }
            }
            "__file_write" => {
                let path = self.eval_as_string(&args[0])?;
                let content = self.eval_as_string(&args[1])?;
                match std::fs::write(&path, &content) {
                    Ok(()) => Ok(Value::Result(
                        Box::new(Value::Unit),
                        Box::new(Value::Null),
                    )),
                    Err(e) => Ok(Value::Result(
                        Box::new(Value::Null),
                        Box::new(Value::String(e.to_string())),
                    )),
                }
            }
            "__file_exists" => {
                let path = self.eval_as_string(&args[0])?;
                Ok(Value::Boolean(std::path::Path::new(&path).exists()))
            }
            "__file_delete" => {
                let path = self.eval_as_string(&args[0])?;
                match std::fs::remove_file(&path) {
                    Ok(()) => Ok(Value::Result(
                        Box::new(Value::Unit),
                        Box::new(Value::Null),
                    )),
                    Err(e) => Ok(Value::Result(
                        Box::new(Value::Null),
                        Box::new(Value::String(e.to_string())),
                    )),
                }
            }
            "__dir_create" => {
                let path = self.eval_as_string(&args[0])?;
                match std::fs::create_dir(&path) {
                    Ok(()) => Ok(Value::Result(
                        Box::new(Value::Unit),
                        Box::new(Value::Null),
                    )),
                    Err(e) => Ok(Value::Result(
                        Box::new(Value::Null),
                        Box::new(Value::String(e.to_string())),
                    )),
                }
            }
            "__dir_exists" => {
                let path = self.eval_as_string(&args[0])?;
                Ok(Value::Boolean(std::path::Path::new(&path).is_dir()))
            }
            "__dir_list" => {
                let path = self.eval_as_string(&args[0])?;
                match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let names: Vec<Value> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| Value::String(e.file_name().to_string_lossy().to_string()))
                            .collect();
                        Ok(Value::Result(
                            Box::new(Value::new_array(names)),
                            Box::new(Value::Null),
                        ))
                    }
                    Err(e) => Ok(Value::Result(
                        Box::new(Value::Null),
                        Box::new(Value::String(e.to_string())),
                    )),
                }
            }
            "__env_var" => {
                let name = self.eval_as_string(&args[0])?;
                match std::env::var(&name) {
                    Ok(value) => Ok(Value::Option(Box::new(Value::String(value)))),
                    Err(_) => Ok(Value::None),
                }
            }
            "__args" => {
                let args: Vec<Value> = std::env::args()
                    .map(Value::String)
                    .collect();
                Ok(Value::new_array(args))
            }
            "__current_dir" => {
                match std::env::current_dir() {
                    Ok(path) => Ok(Value::Result(
                        Box::new(Value::String(path.to_string_lossy().to_string())),
                        Box::new(Value::Null),
                    )),
                    Err(e) => Ok(Value::Result(
                        Box::new(Value::Null),
                        Box::new(Value::String(e.to_string())),
                    )),
                }
            }
            "__exit" => {
                let code = self.eval(&args[0])?;
                let code = self.as_i64(&code)?;
                std::process::exit(code as i32);
            }

            _ => {
                // 首先检查是否是闭包变量
                if let Some(value) = self.variables.get(name).cloned() {
                    if let Value::Closure { params, body, captured } = value {
                        let arg_vals: Vec<Value> = args
                            .iter()
                            .map(|a| self.eval(a))
                            .collect::<Result<_, _>>()?;
                        if arg_vals.len() != params.len() {
                            return Err(InterpreterError::runtime_no_span(format!(
                                "闭包 {} 期望 {} 个参数，得到 {}",
                                name,
                                params.len(),
                                arg_vals.len()
                            )));
                        }
                        // 保存当前变量状态
                        let saved = self.variables.clone();
                        // 添加捕获的变量
                        for (k, v) in captured.borrow().iter() {
                            self.variables.insert(k.clone(), v.clone());
                        }
                        // 添加参数
                        for (p, v) in params.iter().zip(arg_vals) {
                            self.variables.insert(p.clone(), v);
                        }
                        let result = self.execute_block_expr(&body)?;
                        // 恢复变量状态
                        self.variables = saved;
                        match result {
                            ControlFlow::Return(v) => Ok(v),
                            _ => Ok(Value::Unit),
                        }
                    } else {
                        // 不是闭包，继续检查是否是函数
                        self.call_user_function(name, args)
                    }
                } else {
                    // 不是变量，检查是否是函数
                    self.call_user_function(name, args)
                }
            }
        }
    }

    /// 调用用户定义的函数
    fn call_user_function(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<Value, InterpreterError> {
        // 首先检查是否是外部函数（FFI）
        if self.foreign_functions.contains_key(name) {
            return self.call_foreign_function(name, args);
        }

        let func = self.functions.get(name).cloned();
        if let Some(func) = func {
            let arg_vals: Vec<Value> = args
                .iter()
                .map(|a| self.eval(a))
                .collect::<Result<_, _>>()?;
            if arg_vals.len() != func.parameters.len() {
                return Err(InterpreterError::runtime_no_span(format!(
                    "函数 {} 期望 {} 个参数，得到 {}",
                    name,
                    func.parameters.len(),
                    arg_vals.len()
                )));
            }
            // 保存当前变量状态
            let saved = self.variables.clone();
            // 添加函数参数，覆盖同名全局变量
            for (p, v) in func.parameters.iter().zip(arg_vals) {
                self.variables.insert(p.name.clone(), v);
            }
            let result = self.execute_block_expr(&func.body)?;
            // 恢复变量状态
            self.variables = saved;
            match result {
                ControlFlow::Return(v) => Ok(v),
                _ => Ok(Value::Unit),
            }
        } else {
            Err(InterpreterError::runtime_no_span(format!(
                "未定义的函数: {}",
                name
            )))
        }
    }

    /// 调用外部函数（FFI）
    /// 注意：解释器模式下，FFI 函数是模拟的，不会真正调用 C 代码
    fn call_foreign_function(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<Value, InterpreterError> {
        // 评估参数
        let arg_vals: Vec<Value> = args
            .iter()
            .map(|a| self.eval(a))
            .collect::<Result<_, _>>()?;

        // 根据函数名模拟常见的 C 函数
        match name {
            // 时间函数
            "time" => {
                // 返回当前 Unix 时间戳
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                Ok(Value::Integer(timestamp))
            }
            // 内存相关函数
            "malloc" | "calloc" => {
                // 模拟内存分配，返回一个非空指针
                if !arg_vals.is_empty() {
                    Ok(Value::Pointer(1)) // 返回一个非零地址
                } else {
                    Ok(Value::Pointer(1))
                }
            }
            "free" => {
                // 模拟释放内存
                Ok(Value::Unit)
            }
            "memset" => {
                // 模拟 memset
                Ok(Value::Pointer(0))
            }
            "memcpy" => {
                // 模拟 memcpy
                Ok(Value::Pointer(0))
            }
            // 字符串函数
            "strlen" => {
                if let Some(Value::String(s)) = arg_vals.first() {
                    Ok(Value::Integer(s.len() as i64))
                } else {
                    Ok(Value::Integer(0))
                }
            }
            "strcpy" | "strcat" => {
                Ok(Value::Pointer(0))
            }
            // 数学函数
            "sin" | "cos" | "tan" | "sqrt" | "pow" | "log" | "exp" | "abs" | "floor" | "ceil" | "round" => {
                if let Some(Value::Float(f)) = arg_vals.first() {
                    let result = match name {
                        "sin" => f.sin(),
                        "cos" => f.cos(),
                        "tan" => f.tan(),
                        "sqrt" => f.sqrt(),
                        "log" => f.ln(),
                        "exp" => f.exp(),
                        "abs" => f.abs(),
                        "floor" => f.floor(),
                        "ceil" => f.ceil(),
                        "round" => f.round(),
                        _ => *f,
                    };
                    Ok(Value::Float(result))
                } else if let Some(Value::Integer(i)) = arg_vals.first() {
                    let f = *i as f64;
                    let result = match name {
                        "sin" => f.sin(),
                        "cos" => f.cos(),
                        "tan" => f.tan(),
                        "sqrt" => f.sqrt(),
                        "log" => f.ln(),
                        "exp" => f.exp(),
                        "abs" => f.abs() as f64,
                        "floor" => f.floor(),
                        "ceil" => f.ceil(),
                        "round" => f.round(),
                        _ => f,
                    };
                    Ok(Value::Float(result))
                } else {
                    Ok(Value::Float(0.0))
                }
            }
            // 其他函数 - 返回默认值
            _ => {
                // 检查是否有定义的返回类型
                if let Some(extern_func) = self.foreign_functions.get(name) {
                    if let Some(return_type) = &extern_func.return_type {
                        match return_type {
                            x_parser::ast::Type::Void => Ok(Value::Unit),
                            x_parser::ast::Type::Int => Ok(Value::Integer(0)),
                            x_parser::ast::Type::Float => Ok(Value::Float(0.0)),
                            x_parser::ast::Type::Bool => Ok(Value::Boolean(false)),
                            x_parser::ast::Type::String => Ok(Value::String(String::new())),
                            x_parser::ast::Type::Pointer(_) => Ok(Value::Pointer(0)),
                            _ => Ok(Value::Null),
                        }
                    } else {
                        Ok(Value::Unit)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
        }
    }

    /// 实例化类
    fn instantiate_class(
        &mut self,
        class_name: &str,
        args: &[Expression],
    ) -> Result<Value, InterpreterError> {
        let class = self.classes.get(class_name).cloned().ok_or_else(|| {
            InterpreterError::runtime_no_span(format!("未定义的类: {}", class_name))
        })?;

        // 评估参数
        let arg_vals: Vec<Value> = args.iter().map(|a| self.eval(a)).collect::<Result<_, _>>()?;

        // 创建字段
        let mut fields = HashMap::new();
        for member in &class.members {
            if let ClassMember::Field(field) = member {
                let initial_value = if let Some(init) = &field.initializer {
                    self.eval(init)?
                } else {
                    Value::Null
                };
                fields.insert(field.name.clone(), initial_value);
            }
        }

        let instance = Value::Object {
            class_name: class_name.to_string(),
            fields: Rc::new(RefCell::new(fields)),
        };

        // 查找并调用构造函数
        for member in &class.members {
            if let ClassMember::Constructor(constructor) = member {
                if constructor.parameters.len() == arg_vals.len() {
                    // 保存当前变量状态
                    let saved = self.variables.clone();
                    let saved_this = self.variables.get("this").cloned();

                    // 设置 this
                    self.variables.insert("this".to_string(), instance.clone());

                    // 添加构造函数参数
                    for (p, v) in constructor.parameters.iter().zip(&arg_vals) {
                        self.variables.insert(p.name.clone(), v.clone());
                    }

                    // 执行构造函数体
                    let _ = self.execute_block_stmt(&constructor.body)?;

                    // 恢复变量状态
                    self.variables = saved;
                    if let Some(this_val) = saved_this {
                        self.variables.insert("this".to_string(), this_val);
                    }

                    // 返回更新后的实例
                    return Ok(instance);
                }
            }
        }

        // 没有找到匹配的构造函数，直接返回实例
        Ok(instance)
    }

    /// 调用方法
    fn call_method(
        &mut self,
        obj_expr: &Expression,
        method_name: &str,
        args: &[Expression],
    ) -> Result<Value, InterpreterError> {
        let obj_val = self.eval(obj_expr)?;

        // 处理类型值上的静态方法调用（如 Pointer(Void).null()）
        if let Value::Type { name, args: type_args } = &obj_val {
            match name.as_str() {
                "Pointer" => {
                    match method_name {
                        "null" => {
                            if !args.is_empty() {
                                return Err(InterpreterError::runtime_no_span(
                                    "null() 方法不需要参数".to_string(),
                                ));
                            }
                            // 返回空指针值
                            return Ok(Value::Pointer(0));
                        }
                        _ => {
                            return Err(InterpreterError::runtime_no_span(format!(
                                "类型 Pointer 没有静态方法 {}",
                                method_name
                            )));
                        }
                    }
                }
                _ => {
                    return Err(InterpreterError::runtime_no_span(format!(
                        "类型 {} 不支持静态方法调用",
                        name
                    )));
                }
            }
        }

        // 获取类名
        let class_name = match &obj_val {
            Value::Object { class_name, .. } => class_name.clone(),
            _ => return Err(InterpreterError::runtime_no_span("只能对对象调用方法")),
        };

        // 获取类定义
        let class = self.classes.get(&class_name).cloned().ok_or_else(|| {
            InterpreterError::runtime_no_span(format!("未定义的类: {}", class_name))
        })?;

        // 查找方法
        for member in &class.members {
            if let ClassMember::Method(method) = member {
                if method.name == method_name {
                    // 评估参数
                    let arg_vals: Vec<Value> = args.iter().map(|a| self.eval(a)).collect::<Result<_, _>>()?;

                    // 检查参数数量
                    if method.parameters.len() != arg_vals.len() {
                        return Err(InterpreterError::runtime_no_span(format!(
                            "方法 {} 需要 {} 个参数，但提供了 {} 个",
                            method_name,
                            method.parameters.len(),
                            arg_vals.len()
                        )));
                    }

                    // 保存当前变量状态
                    let saved = self.variables.clone();

                    // 设置 this
                    self.variables.insert("this".to_string(), obj_val.clone());

                    // 添加字段作为可直接访问的变量
                    if let Value::Object { fields, .. } = &obj_val {
                        for (field_name, field_val) in fields.borrow().iter() {
                            self.variables.insert(field_name.clone(), field_val.clone());
                        }
                    }

                    // 添加方法参数
                    for (p, v) in method.parameters.iter().zip(&arg_vals) {
                        self.variables.insert(p.name.clone(), v.clone());
                    }

                    // 执行方法体
                    let result = self.execute_block_expr(&method.body)?;

                    // 恢复变量状态
                    self.variables = saved;

                    return match result {
                        ControlFlow::Return(v) => Ok(v),
                        _ => Ok(Value::Unit),
                    };
                }
            }
        }

        Err(InterpreterError::runtime_no_span(format!(
            "类 {} 没有方法 {}",
            class_name, method_name
        )))
    }

    fn eval_as_string(&mut self, expr: &Expression) -> Result<String, InterpreterError> {
        let v = self.eval(expr)?;
        Ok(self.format_value(&v))
    }

    fn eval_binary(
        &self,
        op: BinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        use BinaryOp::*;
        match op {
            Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a.wrapping_add(*b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::String(a), _) => {
                    Ok(Value::String(format!("{}{}", a, self.format_value(right))))
                }
                (_, Value::String(b)) => {
                    Ok(Value::String(format!("{}{}", self.format_value(left), b)))
                }
                _ => Err(InterpreterError::runtime_no_span("+ 需要数字或字符串")),
            },
            Sub => self.numeric_op(left, right, |a, b| a - b, |a, b| a - b),
            Mul => self.numeric_op(left, right, |a, b| a.wrapping_mul(b), |a, b| a * b),
            Div => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(InterpreterError::runtime_no_span("除以零"));
                    }
                    Ok(Value::Integer(a / b))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                _ => Err(InterpreterError::runtime_no_span("/ 需要数字")),
            },
            Mod => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(InterpreterError::runtime_no_span("模除零"));
                    }
                    Ok(Value::Integer(a % b))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 % b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a % *b as f64)),
                _ => Err(InterpreterError::runtime_no_span("% 需要数字")),
            },
            LessEqual | Less | GreaterEqual | Greater => {
                let (a, b) = (self.as_f64(left)?, self.as_f64(right)?);
                let ok = match op {
                    LessEqual => a <= b,
                    Less => a < b,
                    GreaterEqual => a >= b,
                    Greater => a > b,
                    _ => unreachable!(),
                };
                Ok(Value::Boolean(ok))
            }
            Equal | NotEqual => {
                let eq = left == right;
                Ok(Value::Boolean(if matches!(op, Equal) { eq } else { !eq }))
            }
            _ => Err(InterpreterError::runtime_no_span(format!(
                "未实现的二元运算: {:?}",
                op
            ))),
        }
    }

    fn numeric_op(
        &self,
        l: &Value,
        r: &Value,
        int_op: impl Fn(i64, i64) -> i64,
        float_op: impl Fn(f64, f64) -> f64,
    ) -> Result<Value, InterpreterError> {
        match (l, r) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(int_op(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
            _ => Err(InterpreterError::runtime_no_span("运算需要数字")),
        }
    }

    fn as_f64(&self, v: &Value) -> Result<f64, InterpreterError> {
        match v {
            Value::Integer(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(InterpreterError::runtime_no_span("需要数字")),
        }
    }

    fn as_i64(&self, v: &Value) -> Result<i64, InterpreterError> {
        match v {
            Value::Integer(n) => Ok(*n),
            Value::Float(f) => Ok(*f as i64),
            _ => Err(InterpreterError::runtime_no_span("需要整数")),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c) => Value::Char(*c),
            Literal::Null => Value::Null,
            Literal::None => Value::None,
            Literal::Unit => Value::Unit,
        }
    }

    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => {
                if f.fract() == 0.0 && !f.is_infinite() && !f.is_nan() {
                    format!("{:.1}", f)
                } else {
                    format!("{}", f)
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::Array(rc) => {
                let arr = rc.borrow();
                let items: Vec<String> = arr.iter().map(|v| self.format_value(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Map(rc) => {
                let entries = rc.borrow();
                let items: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Object { class_name, fields } => {
                let fields = fields.borrow();
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                    .collect();
                format!("{}{{{}}}", class_name, items.join(", "))
            }
            Value::Null => "null".to_string(),
            Value::None => "None".to_string(),
            Value::Unit => "()".to_string(),
            Value::Option(v) => format!("Some({})", self.format_value(v)),
            Value::Result(ok, err) => {
                if **ok != Value::Null {
                    format!("Ok({})", self.format_value(ok))
                } else {
                    format!("Err({})", self.format_value(err))
                }
            }
            Value::Closure { params, .. } => {
                format!("<closure({})>", params.join(", "))
            }
            Value::Enum(type_name, variant_name, values) => {
                if values.is_empty() {
                    format!("{}.{}", type_name, variant_name)
                } else {
                    let items: Vec<String> = values.iter().map(|v| self.format_value(v)).collect();
                    format!("{}.{}({})", type_name, variant_name, items.join(", "))
                }
            }
            Value::Type { name, args } => {
                if args.is_empty() {
                    format!("{}(())", name)
                } else {
                    let type_strs: Vec<String> = args.iter().map(|a| format!("{:?}", a)).collect();
                    format!("{}({})", name, type_strs.join(", "))
                }
            }
            Value::Pointer(addr) => format!("Pointer(0x{:x})", addr),
        }
    }

    fn value_to_json(&self, value: &Value) -> String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => {
                if f.fract() == 0.0 && !f.is_infinite() && !f.is_nan() {
                    format!("{:.1}", f)
                } else {
                    format!("{}", f)
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"").replace("\\", "\\\\")),
            Value::Char(c) => format!("\"{}\"", c),
            Value::Array(rc) => {
                let arr = rc.borrow();
                let items: Vec<String> = arr.iter().map(|v| self.value_to_json(v)).collect();
                format!("[{}]", items.join(","))
            }
            Value::Map(rc) => {
                let entries = rc.borrow();
                let items: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, self.value_to_json(v)))
                    .collect();
                format!("{{{}}}", items.join(","))
            }
            Value::Object { class_name, fields } => {
                let fields = fields.borrow();
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, self.value_to_json(v)))
                    .collect();
                format!("{{\"__class__\":\"{}\",{}}}", class_name, items.join(","))
            }
            Value::Null => "null".to_string(),
            Value::None => "null".to_string(),
            Value::Unit => "null".to_string(),
            Value::Option(v) => self.value_to_json(v),
            Value::Result(ok, _) => self.value_to_json(ok),
            Value::Closure { params, .. } => {
                format!("{{\"__closure__\":{{\"params\":[{}]}}}}", params.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(","))
            }
            Value::Enum(type_name, variant_name, values) => {
                let items: Vec<String> = values.iter().map(|v| self.value_to_json(v)).collect();
                format!("{{\"__enum__\":{{\"type\":\"{}\",\"variant\":\"{}\",\"values\":[{}]}}}}",
                    type_name, variant_name, items.join(","))
            }
            Value::Type { name, args } => {
                let type_strs: Vec<String> = args.iter().map(|a| format!("\"{:?}\"", a)).collect();
                format!("{{\"__type__\":{{\"name\":\"{}\",\"args\":[{}]}}}}", name, type_strs.join(","))
            }
            Value::Pointer(addr) => format!("{{\"__pointer__\":{}}}", addr),
        }
    }

    fn json_to_value(&mut self, json: &str) -> Result<Value, InterpreterError> {
        let json = json.trim();
        if json.starts_with('"') && json.ends_with('"') {
            let s = &json[1..json.len() - 1];
            let s = s.replace("\\\"", "\"").replace("\\\\", "\\");
            Ok(Value::String(s))
        } else if json == "true" {
            Ok(Value::Boolean(true))
        } else if json == "false" {
            Ok(Value::Boolean(false))
        } else if json == "null" {
            Ok(Value::Null)
        } else if json.starts_with('[') && json.ends_with(']') {
            let mut items = Vec::new();
            let mut current = String::new();
            let mut depth = 0;
            for c in json.chars().skip(1).take(json.len() - 2) {
                if c == '[' || c == '{' {
                    depth += 1;
                    current.push(c);
                } else if c == ']' || c == '}' {
                    depth -= 1;
                    current.push(c);
                } else if c == ',' && depth == 0 {
                    items.push(current.trim().to_string());
                    current = String::new();
                } else {
                    current.push(c);
                }
            }
            if !current.trim().is_empty() {
                items.push(current.trim().to_string());
            }
            let mut values = Vec::new();
            for item in items {
                values.push(self.json_to_value(&item)?);
            }
            Ok(Value::new_array(values))
        } else if json.starts_with('{') && json.ends_with('}') {
            let map = Value::new_map();
            Ok(map)
        } else if json.parse::<i64>().is_ok() {
            Ok(Value::Integer(json.parse::<i64>().unwrap()))
        } else if json.parse::<f64>().is_ok() {
            Ok(Value::Float(json.parse::<f64>().unwrap()))
        } else {
            Err(InterpreterError::runtime_no_span(format!(
                "无效的JSON: {}",
                json
            )))
        }
    }
}

fn extract_sort_key(v: &Value) -> f64 {
    match v {
        Value::Array(rc) => {
            let arr = rc.borrow();
            if arr.len() >= 2 {
                match &arr[1] {
                    Value::Integer(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => 0.0,
                }
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn compute_pi_digits_bigint(n: usize) -> String {
    use num_integer::Integer;
    let mut digits = String::new();
    let mut q: BigInt = One::one();
    let mut r: BigInt = Zero::zero();
    let mut t: BigInt = One::one();
    let mut k: BigInt = One::one();
    let mut n1: BigInt = BigInt::from(3);
    let mut l: BigInt = BigInt::from(3);

    while digits.len() < n {
        if BigInt::from(4) * &q + &r - &t < &n1 * &t {
            digits.push_str(&n1.to_string());
            let nr = BigInt::from(10) * (&r - &n1 * &t);
            n1 = (BigInt::from(10) * (BigInt::from(3) * &q + &r)).div_floor(&t)
                - BigInt::from(10) * &n1;
            q = BigInt::from(10) * &q;
            r = nr;
        } else {
            let nr = (BigInt::from(2) * &q + &r) * &l;
            let nn =
                (&q * (BigInt::from(7) * &k + BigInt::from(2)) + &r * &l).div_floor(&(&t * &l));
            q = &q * &k;
            t = &t * &l;
            l = &l + BigInt::from(2);
            k = &k + BigInt::one();
            r = nr;
            n1 = nn;
        }
    }
    digits.truncate(n);
    digits
}

fn simple_regex_count(text: &str, pattern: &str) -> usize {
    if pattern.contains('|') {
        return pattern
            .split('|')
            .map(|p| simple_regex_count(text, p))
            .sum();
    }
    if pattern.contains('[') {
        let chars: Vec<char> = text.chars().collect();
        let pat_chars: Vec<char> = pattern.chars().collect();
        let pat_len = compute_pattern_len(&pat_chars);
        if pat_len == 0 {
            return 0;
        }
        let mut count = 0;
        let mut i = 0;
        while i + pat_len <= chars.len() {
            if match_at(&chars, i, &pat_chars) {
                count += 1;
                i += pat_len;
            } else {
                i += 1;
            }
        }
        return count;
    }
    text.matches(pattern).count()
}

fn compute_pattern_len(pat: &[char]) -> usize {
    let mut len = 0;
    let mut i = 0;
    while i < pat.len() {
        if pat[i] == '[' {
            while i < pat.len() && pat[i] != ']' {
                i += 1;
            }
            i += 1;
            len += 1;
        } else {
            i += 1;
            len += 1;
        }
    }
    len
}

fn match_at(text: &[char], start: usize, pat: &[char]) -> bool {
    let mut ti = start;
    let mut pi = 0;
    while pi < pat.len() {
        if ti >= text.len() {
            return false;
        }
        if pat[pi] == '[' {
            pi += 1;
            let mut matched = false;
            while pi < pat.len() && pat[pi] != ']' {
                if text[ti] == pat[pi] {
                    matched = true;
                }
                pi += 1;
            }
            if !matched {
                return false;
            }
            pi += 1;
            ti += 1;
        } else {
            if text[ti] != pat[pi] {
                return false;
            }
            ti += 1;
            pi += 1;
        }
    }
    true
}

fn simple_regex_replace(text: &str, pattern: &str, replacement: &str) -> String {
    if pattern.contains('[') || pattern.contains('|') {
        let chars: Vec<char> = text.chars().collect();
        let alternatives: Vec<&str> = pattern.split('|').collect();
        let mut result = String::new();
        let mut i = 0;
        while i < chars.len() {
            let mut matched = false;
            for alt in &alternatives {
                let pat_chars: Vec<char> = alt.chars().collect();
                let pat_len = compute_pattern_len(&pat_chars);
                if i + pat_len <= chars.len() && match_at(&chars, i, &pat_chars) {
                    result.push_str(replacement);
                    i += pat_len;
                    matched = true;
                    break;
                }
            }
            if !matched {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    } else {
        text.replace(pattern, replacement)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InterpreterError {
    #[error("运行时错误: {message}")]
    RuntimeError {
        message: String,
        span: Span,
    },

    #[error("未定义的变量: {name}")]
    UndefinedVariable { name: String, span: Span },

    #[error("未定义的函数: {name}")]
    UndefinedFunction { name: String, span: Span },

    #[error("类型错误: {message}")]
    TypeError { message: String, span: Span },

    #[error("除以零")]
    DivisionByZero { span: Span },

    #[error("索引越界: 索引 {index}, 长度 {length}")]
    IndexOutOfBounds {
        index: usize,
        length: usize,
        span: Span,
    },

    #[error("参数数量不匹配: 期望 {expected}, 实际 {actual}")]
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
        span: Span,
    },

    #[error("模式匹配失败: {message}")]
    MatchFailure { message: String, span: Span },
}

impl InterpreterError {
    /// 创建一个带有位置信息的运行时错误
    pub fn runtime(message: impl Into<String>, span: Span) -> Self {
        InterpreterError::RuntimeError {
            message: message.into(),
            span,
        }
    }

    /// 创建一个带有位置信息的运行时错误（使用默认 span）
    pub fn runtime_no_span(message: impl Into<String>) -> Self {
        InterpreterError::RuntimeError {
            message: message.into(),
            span: Span::default(),
        }
    }

    /// 创建未定义变量错误
    pub fn undefined_variable(name: impl Into<String>, span: Span) -> Self {
        InterpreterError::UndefinedVariable {
            name: name.into(),
            span,
        }
    }

    /// 创建未定义函数错误
    pub fn undefined_function(name: impl Into<String>, span: Span) -> Self {
        InterpreterError::UndefinedFunction {
            name: name.into(),
            span,
        }
    }

    /// 创建类型错误
    pub fn type_error(message: impl Into<String>, span: Span) -> Self {
        InterpreterError::TypeError {
            message: message.into(),
            span,
        }
    }

    /// 创建除以零错误
    pub fn division_by_zero(span: Span) -> Self {
        InterpreterError::DivisionByZero { span }
    }

    /// 获取错误的源码位置
    pub fn span(&self) -> Span {
        match self {
            InterpreterError::RuntimeError { span, .. } => *span,
            InterpreterError::UndefinedVariable { span, .. } => *span,
            InterpreterError::UndefinedFunction { span, .. } => *span,
            InterpreterError::TypeError { span, .. } => *span,
            InterpreterError::DivisionByZero { span } => *span,
            InterpreterError::IndexOutOfBounds { span, .. } => *span,
            InterpreterError::ArgumentCountMismatch { span, .. } => *span,
            InterpreterError::MatchFailure { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ok(source: &str) -> Result<(), InterpreterError> {
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");
        let mut interpreter = Interpreter::new();
        interpreter.run(&program)
    }

    #[test]
    fn test_hello_world() {
        let source = r#"
            function main() {
                print("Hello, World!")
            }
        "#;

        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");

        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Failed to run");
    }

    #[test]
    fn test_top_level_statement() {
        // 测试直接在顶层写语句，不需要main函数
        let source = r#"
            print("Hello, World!")
        "#;

        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");

        // 验证有顶层语句
        assert_eq!(program.statements.len(), 1);

        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Failed to run");
    }

    #[test]
    fn test_top_level_with_declarations() {
        // 测试顶层声明和语句混合
        let source = r#"
            let x = 42
            function greet(name) {
                print("Hello, " + name)
            }
            greet("World")
            print(x)
        "#;

        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");

        // 验证有声明和语句
        assert_eq!(program.declarations.len(), 2); // 1个变量 + 1个函数
        assert_eq!(program.statements.len(), 2); // 2个语句

        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Failed to run");
    }

    #[test]
    fn test_arithmetic() {
        let source = r#"
            function main() {
                let x = 10
                let y = 20
                print(x + y)
            }
        "#;

        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");

        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Failed to run");
    }

    #[test]
    fn test_function_call() {
        let source = r#"
            function add(a, b) {
                return a + b
            }

            function main() {
                print(add(5, 7))
            }
        "#;

        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");

        let mut interpreter = Interpreter::new();
        interpreter.run(&program).expect("Failed to run");
    }

    #[test]
    fn test_match_or_pattern_and_guard() {
        let source = r#"
            let x = 2;
            match x {
                1 | 2 when true { print("hit") }
                _ { print("miss") }
            }
        "#;
        run_ok(source).expect("match should run");
    }

    #[test]
    fn test_try_catch_finally_runs() {
        let source = r#"
            function fail() {
                // 触发运行时错误：未定义变量
                return missing
            }

            try {
                fail()
            } catch (Error e) {
                // 确保 catch 变量可用
                print(e)
            } finally {
                print("done")
            }
        "#;
        run_ok(source).expect("try/catch/finally should run");
    }

    #[test]
    fn test_short_circuit_and_or() {
        let source = r#"
            let x = 0;
            false && missing;
            true || missing;
            true && (x = 1);
            false || (x = 2);
            print(x);
        "#;
        run_ok(source).expect("short-circuit should avoid missing");
    }

    #[test]
    fn test_range_expression_and_for_over_array() {
        let source = r#"
            let sum = 0;
            let xs = 1..=3;
            for x in xs { sum = sum + x }
            print(sum)
        "#;
        run_ok(source).expect("range and for should run");
    }

    #[test]
    fn test_for_over_non_array_errors() {
        let source = r#"
            for x in 1 { print(x) }
        "#;
        let err = run_ok(source).expect_err("should error");
        let msg = err.to_string();
        assert!(msg.contains("For循环只支持数组迭代") || msg.contains("运行时错误"));
    }

    #[test]
    fn test_divide_by_zero_errors() {
        let source = r#"
            print(1 / 0)
        "#;
        let err = run_ok(source).expect_err("should error");
        match err {
            InterpreterError::DivisionByZero { .. } => {}
            InterpreterError::RuntimeError { message, .. } => assert!(message.contains("除以零")),
            _ => panic!("unexpected error type"),
        }
    }

    #[test]
    fn test_try_without_catch_propagates_error() {
        let mut i = Interpreter::new();
        assert!(i
            .eval(&Spanned::new(
                ExpressionKind::Variable("missing".to_string()),
                Span::default()
            ))
            .is_err());

        let source = r#"
            function fail() { return missing }
            try { fail() } finally { print("done") }
        "#;

        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("Failed to parse");
        match &program.declarations[0] {
            Declaration::Function(f) => match &f.body.statements[0].node {
                StatementKind::Return(Some(expr)) => match &expr.node {
                    ExpressionKind::Variable(n) => assert_eq!(n, "missing"),
                    other => panic!("unexpected return expr: {other:?}"),
                },
                other => panic!("unexpected fail() body: {other:?}"),
            },
            other => panic!("unexpected first decl: {other:?}"),
        }

        let mut i2 = Interpreter::new();
        i2.load_declaration(&program.declarations[0])
            .expect("load decl ok");
        assert!(i2.call_function("fail", &[]).is_err());

        let try_expr = match &program.statements[0].node {
            StatementKind::Try(t) => match &t.body.statements[0].node {
                StatementKind::Expression(e) => e,
                other => panic!("unexpected try body stmt: {other:?}"),
            },
            other => panic!("unexpected first stmt: {other:?}"),
        };
        let mut i3 = Interpreter::new();
        i3.load_declaration(&program.declarations[0])
            .expect("load decl ok");
        assert!(i3.eval(try_expr).is_err());

        let err = run_ok(source).expect_err("should error");
        let msg = err.to_string();
        assert!(msg.contains("未定义的变量"), "error should mention undefined variable");
    }
}
