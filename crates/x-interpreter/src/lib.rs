// 解释器库

use std::collections::HashMap;
use x_parser::ast::{Program, Declaration, FunctionDecl, VariableDecl, Expression, Literal, Block, Statement, BinaryOp};

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {
    // 解释器状态
    variables: HashMap<String, Value>,
    functions: HashMap<String, FunctionDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Null,
    None,
    Unit,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut variables = HashMap::new();
        let mut functions = HashMap::new();

        Self { variables, functions }
    }

    pub fn run(&mut self, program: &x_parser::ast::Program) -> Result<(), InterpreterError> {
        // 加载程序中的声明
        self.load_declarations(program)?;

        // 查找并执行main函数
        if let Some(main_func) = self.functions.get("main") {
            let main_func = main_func.clone();
            let _ = self.execute_block(&main_func.body)?;
        } else {
            return Err(InterpreterError::RuntimeError("找不到main函数".to_string()));
        }

        Ok(())
    }

    fn load_declarations(&mut self, program: &Program) -> Result<(), InterpreterError> {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Declaration::Variable(var) => {
                    if let Some(initializer) = &var.initializer {
                        let value = self.evaluate_expression(initializer)?;
                        self.variables.insert(var.name.clone(), value);
                    }
                }
                _ => continue, // 暂时忽略其他类型的声明
            }
        }
        Ok(())
    }

    fn execute_block(&mut self, block: &Block) -> Result<Option<Value>, InterpreterError> {
        for statement in &block.statements {
            if let Some(ret_value) = self.execute_statement(statement)? {
                return Ok(Some(ret_value));
            }
        }
        Ok(None)
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<Option<Value>, InterpreterError> {
        match statement {
            Statement::Variable(var) => {
                if let Some(initializer) = &var.initializer {
                    let value = self.evaluate_expression(initializer)?;
                    self.variables.insert(var.name.clone(), value);
                }
                Ok(None)
            }
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(None)
            }
            Statement::Return(Some(expr)) => {
                let value = self.evaluate_expression(expr)?;
                Ok(Some(value))
            }
            Statement::Return(None) => Ok(Some(Value::Unit)),
            Statement::If(if_stmt) => {
                let cond_value = self.evaluate_expression(&if_stmt.condition)?;
                let run_then = self.is_truthy(&cond_value);
                if run_then {
                    self.execute_block(&if_stmt.then_block)
                } else if let Some(else_blk) = &if_stmt.else_block {
                    self.execute_block(else_blk)
                } else {
                    Ok(None)
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!("未实现的语句类型: {:?}", statement))),
        }
    }

    fn is_truthy(&self, v: &Value) -> bool {
        match v {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Null | Value::None | Value::Unit => false,
            Value::String(_) | Value::Char(_) => true,
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, InterpreterError> {
        match expr {
            Expression::Literal(literal) => Ok(self.evaluate_literal(literal)),
            Expression::Variable(name) => {
                if let Some(value) = self.variables.get(name) {
                    Ok(value.clone())
                } else {
                    Err(InterpreterError::RuntimeError(format!("未定义的变量: {}", name)))
                }
            }
            Expression::Binary(op, left, right) => {
                let l = self.evaluate_expression(left)?;
                let r = self.evaluate_expression(right)?;
                self.eval_binary(op.clone(), &l, &r)
            }
            Expression::Call(callee, args) => {
                if let Expression::Variable(name) = callee.as_ref() {
                    match name.as_str() {
                        "print" => {
                            for arg in args {
                                let arg_value = self.evaluate_expression(arg)?;
                                println!("{}", self.format_value(&arg_value));
                            }
                            Ok(Value::Unit)
                        }
                        _ => {
                            let func = self.functions.get(name).cloned();
                            if let Some(func) = func {
                                let arg_values: Vec<Value> = args
                                    .iter()
                                    .map(|a| self.evaluate_expression(a))
                                    .collect::<Result<Vec<_>, _>>()?;
                                if arg_values.len() != func.parameters.len() {
                                    return Err(InterpreterError::RuntimeError(format!(
                                        "函数 {} 期望 {} 个参数，得到 {}",
                                        name,
                                        func.parameters.len(),
                                        arg_values.len()
                                    )));
                                }
                                let saved = std::mem::take(&mut self.variables);
                                for (param, val) in func.parameters.iter().zip(arg_values.iter()) {
                                    self.variables.insert(param.name.clone(), val.clone());
                                }
                                let result = self.execute_block(&func.body)?;
                                self.variables = saved;
                                Ok(result.unwrap_or(Value::Unit))
                            } else {
                                Err(InterpreterError::RuntimeError(format!("未定义的函数: {}", name)))
                            }
                        }
                    }
                } else {
                    Err(InterpreterError::RuntimeError("只支持调用命名函数".to_string()))
                }
            }
            Expression::Parenthesized(inner) => self.evaluate_expression(inner),
            _ => Err(InterpreterError::RuntimeError(format!("未实现的表达式类型: {:?}", expr))),
        }
    }

    fn eval_binary(&self, op: BinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        use BinaryOp::*;
        match op {
            Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                _ => Err(InterpreterError::RuntimeError("+ 需要数字".to_string())),
            },
            Sub => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(InterpreterError::RuntimeError("- 需要数字".to_string())),
            },
            Mul => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(InterpreterError::RuntimeError("* 需要数字".to_string())),
            },
            Div => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                _ => Err(InterpreterError::RuntimeError("/ 需要数字".to_string())),
            },
            Mod => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
                _ => Err(InterpreterError::RuntimeError("% 需要整数".to_string())),
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
                let ok = matches!(op, Equal) && eq || matches!(op, NotEqual) && !eq;
                Ok(Value::Boolean(ok))
            }
            _ => Err(InterpreterError::RuntimeError(format!("未实现的二元运算: {:?}", op))),
        }
    }

    fn as_f64(&self, v: &Value) -> Result<f64, InterpreterError> {
        match v {
            Value::Integer(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(InterpreterError::RuntimeError("比较运算需要数字".to_string())),
        }
    }

    fn evaluate_literal(&self, literal: &Literal) -> Value {
        match literal {
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
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::Null => "null".to_string(),
            Value::None => "None".to_string(),
            Value::Unit => "()".to_string(),
        }
    }
}

/// 解释器错误
#[derive(thiserror::Error, Debug)]
pub enum InterpreterError {
    #[error("解释器错误: {0}")]
    RuntimeError(String),
}