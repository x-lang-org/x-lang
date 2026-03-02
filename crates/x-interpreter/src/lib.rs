use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use x_parser::ast::{
    BinaryOp, Block, Declaration, Expression, FunctionDecl, Literal, Program, Statement, UnaryOp,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, FunctionDecl>,
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
    Null,
    None,
    Unit,
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
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
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
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), InterpreterError> {
        self.load_declarations(program)?;
        if let Some(main_func) = self.functions.get("main").cloned() {
            let saved = std::mem::take(&mut self.variables);
            let _ = self.execute_block(&main_func.body)?;
            self.variables = saved;
        } else {
            return Err(InterpreterError::RuntimeError("找不到main函数".into()));
        }
        Ok(())
    }

    fn load_declarations(&mut self, program: &Program) -> Result<(), InterpreterError> {
        for decl in &program.declarations {
            match decl {
                Declaration::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Declaration::Variable(var) => {
                    if let Some(init) = &var.initializer {
                        let val = self.eval(init)?;
                        self.variables.insert(var.name.clone(), val);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn execute_block(&mut self, block: &Block) -> Result<ControlFlow, InterpreterError> {
        for stmt in &block.statements {
            match self.execute_statement(stmt)? {
                ControlFlow::None => {}
                cf => return Ok(cf),
            }
        }
        Ok(ControlFlow::None)
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<ControlFlow, InterpreterError> {
        match stmt {
            Statement::Variable(var) => {
                let val = if let Some(init) = &var.initializer {
                    self.eval(init)?
                } else {
                    Value::Null
                };
                self.variables.insert(var.name.clone(), val);
                Ok(ControlFlow::None)
            }
            Statement::Expression(expr) => {
                self.eval(expr)?;
                Ok(ControlFlow::None)
            }
            Statement::Return(Some(expr)) => {
                let val = self.eval(expr)?;
                Ok(ControlFlow::Return(val))
            }
            Statement::Return(std::option::Option::None) => Ok(ControlFlow::Return(Value::Unit)),
            Statement::If(if_stmt) => {
                let cond = self.eval(&if_stmt.condition)?;
                if self.is_truthy(&cond) {
                    self.execute_block(&if_stmt.then_block)
                } else if let Some(else_blk) = &if_stmt.else_block {
                    self.execute_block(else_blk)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Statement::While(while_stmt) => loop {
                let cond = self.eval(&while_stmt.condition)?;
                if !self.is_truthy(&cond) {
                    break Ok(ControlFlow::None);
                }
                match self.execute_block(&while_stmt.body)? {
                    ControlFlow::Return(v) => break Ok(ControlFlow::Return(v)),
                    ControlFlow::Break => break Ok(ControlFlow::None),
                    _ => {}
                }
            },
            _ => Err(InterpreterError::RuntimeError(format!(
                "未实现的语句类型: {:?}",
                stmt
            ))),
        }
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
        match expr {
            Expression::Literal(lit) => Ok(self.eval_literal(lit)),
            Expression::Variable(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| InterpreterError::RuntimeError(format!("未定义的变量: {}", name))),
            Expression::Binary(op, l, r) => {
                if matches!(op, BinaryOp::And) {
                    let lv = self.eval(l)?;
                    return if !self.is_truthy(&lv) { Ok(lv) } else { self.eval(r) };
                }
                if matches!(op, BinaryOp::Or) {
                    let lv = self.eval(l)?;
                    return if self.is_truthy(&lv) { Ok(lv) } else { self.eval(r) };
                }
                let lv = self.eval(l)?;
                let rv = self.eval(r)?;
                self.eval_binary(op.clone(), &lv, &rv)
            }
            Expression::Unary(op, operand) => {
                let v = self.eval(operand)?;
                match op {
                    UnaryOp::Negate => match v {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(InterpreterError::RuntimeError("- 需要数字".into())),
                    },
                    UnaryOp::Not => Ok(Value::Boolean(!self.is_truthy(&v))),
                    _ => Err(InterpreterError::RuntimeError(format!(
                        "未实现的一元运算: {:?}",
                        op
                    ))),
                }
            }
            Expression::Assign(target, value) => {
                let val = self.eval(value)?;
                self.do_assign(target, val)
            }
            Expression::Call(callee, args) => {
                if let Expression::Variable(name) = callee.as_ref() {
                    if name == "__index__" {
                        return self.eval_index(args);
                    }
                    return self.call_function(name, args);
                }
                Err(InterpreterError::RuntimeError("只支持调用命名函数".into()))
            }
            Expression::Array(elems) => {
                let vals: Vec<Value> = elems
                    .iter()
                    .map(|e| self.eval(e))
                    .collect::<Result<_, _>>()?;
                Ok(Value::new_array(vals))
            }
            Expression::Parenthesized(inner) => self.eval(inner),
            Expression::Member(obj, _member) => self.eval(obj),
            _ => Err(InterpreterError::RuntimeError(format!(
                "未实现的表达式类型: {:?}",
                expr
            ))),
        }
    }

    fn do_assign(&mut self, target: &Expression, val: Value) -> Result<Value, InterpreterError> {
        match target {
            Expression::Variable(name) => {
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }
            Expression::Call(func, args)
                if matches!(func.as_ref(), Expression::Variable(n) if n == "__index__") =>
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
                            return Err(InterpreterError::RuntimeError(format!(
                                "数组索引越界: {} (长度 {})",
                                i,
                                arr.len()
                            )));
                        }
                        _ => {}
                    }
                }
                Err(InterpreterError::RuntimeError("无效的赋值目标".into()))
            }
            _ => Err(InterpreterError::RuntimeError("无效的赋值目标".into())),
        }
    }

    fn eval_index(&mut self, args: &[Expression]) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError("索引需要2个参数".into()));
        }
        let obj = self.eval(&args[0])?;
        let idx = self.eval(&args[1])?;
        match (&obj, &idx) {
            (Value::Array(rc), Value::Integer(i)) => {
                let i = *i as usize;
                let arr = rc.borrow();
                arr.get(i).cloned().ok_or_else(|| {
                    InterpreterError::RuntimeError(format!(
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
                    .ok_or_else(|| InterpreterError::RuntimeError("字符串索引越界".into()))
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
            _ => Err(InterpreterError::RuntimeError(format!(
                "不支持的索引操作: {:?}[{:?}]",
                obj, idx
            ))),
        }
    }

    fn call_function(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<Value, InterpreterError> {
        match name {
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
                    _ => Err(InterpreterError::RuntimeError("len 需要数组/字符串/映射".into())),
                }
            }
            "push" => {
                let container = self.eval(&args[0])?;
                let val = self.eval(&args[1])?;
                match &container {
                    Value::Array(rc) => {
                        rc.borrow_mut().push(val);
                        Ok(Value::Unit)
                    }
                    _ => Err(InterpreterError::RuntimeError("push 需要数组".into())),
                }
            }
            "pop" => {
                let container = self.eval(&args[0])?;
                match &container {
                    Value::Array(rc) => Ok(rc.borrow_mut().pop().unwrap_or(Value::Null)),
                    _ => Err(InterpreterError::RuntimeError("pop 需要数组".into())),
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
                    _ => Err(InterpreterError::RuntimeError("map_set 需要映射".into())),
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
                        let keys: Vec<Value> =
                            entries.iter().map(|(k, _)| Value::String(k.clone())).collect();
                        Ok(Value::new_array(keys))
                    }
                    _ => Err(InterpreterError::RuntimeError("map_keys 需要映射".into())),
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
                    Value::String(s) => s
                        .trim()
                        .parse::<i64>()
                        .map(Value::Integer)
                        .map_err(|_| {
                            InterpreterError::RuntimeError(format!("无法转换为整数: {}", s))
                        }),
                    Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
                    _ => Err(InterpreterError::RuntimeError("无法转换为整数".into())),
                }
            }
            "to_float" => {
                let v = self.eval(&args[0])?;
                match v {
                    Value::Float(f) => Ok(Value::Float(f)),
                    Value::Integer(n) => Ok(Value::Float(n as f64)),
                    Value::String(s) => s
                        .trim()
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| {
                            InterpreterError::RuntimeError(format!("无法转换为浮点: {}", s))
                        }),
                    _ => Err(InterpreterError::RuntimeError("无法转换为浮点".into())),
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
                    _ => Err(InterpreterError::RuntimeError("abs 需要数字".into())),
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
                Ok(Value::Float(self.as_f64(&base_v)?.powf(self.as_f64(&exp_v)?)))
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
                    .ok_or_else(|| InterpreterError::RuntimeError("char_at 索引越界".into()))
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
                let parts: Vec<Value> =
                    s.split(&delim).map(|p| Value::String(p.to_string())).collect();
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
                Ok(Value::String(simple_regex_replace(&text, &pattern, &replacement)))
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
                let t = match v {
                    Value::Integer(_) => "Int",
                    Value::Float(_) => "Float",
                    Value::Boolean(_) => "Bool",
                    Value::String(_) => "String",
                    Value::Array(_) => "Array",
                    Value::Map(_) => "Map",
                    Value::Char(_) => "Char",
                    Value::Null => "Null",
                    Value::None => "None",
                    Value::Unit => "Unit",
                };
                Ok(Value::String(t.to_string()))
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
                        Err(InterpreterError::RuntimeError("swap 索引越界".into()))
                    }
                    _ => Err(InterpreterError::RuntimeError("swap 需要数组".into())),
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
                        Err(InterpreterError::RuntimeError("reverse_range 范围越界".into()))
                    }
                    _ => Err(InterpreterError::RuntimeError("reverse_range 需要数组".into())),
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
                    _ => Err(InterpreterError::RuntimeError("sort 需要数组".into())),
                }
            }
            _ => {
                let func = self.functions.get(name).cloned();
                if let Some(func) = func {
                    let arg_vals: Vec<Value> = args
                        .iter()
                        .map(|a| self.eval(a))
                        .collect::<Result<_, _>>()?;
                    if arg_vals.len() != func.parameters.len() {
                        return Err(InterpreterError::RuntimeError(format!(
                            "函数 {} 期望 {} 个参数，得到 {}",
                            name,
                            func.parameters.len(),
                            arg_vals.len()
                        )));
                    }
                    let saved = std::mem::take(&mut self.variables);
                    for (p, v) in func.parameters.iter().zip(arg_vals) {
                        self.variables.insert(p.name.clone(), v);
                    }
                    let result = self.execute_block(&func.body)?;
                    self.variables = saved;
                    match result {
                        ControlFlow::Return(v) => Ok(v),
                        _ => Ok(Value::Unit),
                    }
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "未定义的函数: {}",
                        name
                    )))
                }
            }
        }
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
                _ => Err(InterpreterError::RuntimeError("+ 需要数字或字符串".into())),
            },
            Sub => self.numeric_op(left, right, |a, b| a - b, |a, b| a - b),
            Mul => self.numeric_op(left, right, |a, b| a.wrapping_mul(b), |a, b| a * b),
            Div => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(InterpreterError::RuntimeError("除以零".into()));
                    }
                    Ok(Value::Integer(a / b))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                _ => Err(InterpreterError::RuntimeError("/ 需要数字".into())),
            },
            Mod => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        return Err(InterpreterError::RuntimeError("模除零".into()));
                    }
                    Ok(Value::Integer(a % b))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 % b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a % *b as f64)),
                _ => Err(InterpreterError::RuntimeError("% 需要数字".into())),
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
            _ => Err(InterpreterError::RuntimeError(format!(
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
            _ => Err(InterpreterError::RuntimeError("运算需要数字".into())),
        }
    }

    fn as_f64(&self, v: &Value) -> Result<f64, InterpreterError> {
        match v {
            Value::Integer(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(InterpreterError::RuntimeError("需要数字".into())),
        }
    }

    fn as_i64(&self, v: &Value) -> Result<i64, InterpreterError> {
        match v {
            Value::Integer(n) => Ok(*n),
            Value::Float(f) => Ok(*f as i64),
            _ => Err(InterpreterError::RuntimeError("需要整数".into())),
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
            Value::Null => "null".to_string(),
            Value::None => "None".to_string(),
            Value::Unit => "()".to_string(),
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
            let nn = (&q * (BigInt::from(7) * &k + BigInt::from(2)) + &r * &l)
                .div_floor(&(&t * &l));
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
        return pattern.split('|').map(|p| simple_regex_count(text, p)).sum();
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
    #[error("解释器错误: {0}")]
    RuntimeError(String),
}
