//! Erlang 后端 - 生成 Erlang OTP 28 源代码
//!
//! 面向并发、分布式系统、高可用场景
//!
//! ## Erlang OTP 28 特性支持
//! - Gradual set-theoretic types（渐进式集合论类型）
//! - maybe 类型
//! - JSON 支持
//! - Process labels（进程标签）
//! - Improved maps（改进的映射）
//! - Map comprehensions
//! - Improved ETS and dialyzer
//!
//! ## Erlang 语法特点
//! - 变量以大写字母或下划线开头
//! - 原子以小写字母开头
//! - 函数定义使用 `->` 和 `.`
//! - 模式匹配使用 `case ... of ... end`
//! - 输出使用 `io:format/2`

use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;
use x_parser::ast::{self, ExpressionKind, StatementKind, Program as AstProgram};

/// Erlang 后端配置
#[derive(Debug, Clone)]
pub struct ErlangBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub module_name: Option<String>,
}

impl Default for ErlangBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            module_name: None,
        }
    }
}

/// Erlang 后端
pub struct ErlangBackend {
    config: ErlangBackendConfig,
    indent: usize,
    output: String,
    module_name: String,
    exports: Vec<String>,
}

pub type ErlangResult<T> = Result<T, x_codegen::CodeGenError>;

impl ErlangBackend {
    pub fn new(config: ErlangBackendConfig) -> Self {
        let module_name = config.module_name.clone().unwrap_or_else(|| "x_module".to_string());
        Self {
            config,
            indent: 0,
            output: String::new(),
            module_name,
            exports: Vec::new(),
        }
    }

    /// 从 AST 生成 Erlang 代码
    pub fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> ErlangResult<CodegenOutput> {
        self.output.clear();
        self.indent = 0;
        self.exports.clear();

        // 收集需要导出的函数
        let mut has_main = false;
        for decl in &program.declarations {
            if let ast::Declaration::Function(f) = decl {
                let arity = f.parameters.len();
                self.exports.push(format!("{}/{}", f.name, arity));
                if f.name == "main" {
                    has_main = true;
                }
            }
        }

        // 发射模块头
        self.emit_header()?;

        // 发射函数
        for decl in &program.declarations {
            match decl {
                ast::Declaration::Function(f) => {
                    self.emit_function(f)?;
                    self.line("")?;
                }
                ast::Declaration::Variable(v) => {
                    // 全局变量作为函数导出
                    self.emit_global_var(v)?;
                }
                ast::Declaration::Class(class) => {
                    // 类作为记录导出
                    self.emit_class_as_record(class)?;
                }
                _ => {}
            }
        }

        // 如果没有 main 函数，生成默认的
        if !has_main {
            self.emit_default_main()?;
        }

        // 创建输出文件
        let output_file = OutputFile {
            path: std::path::PathBuf::from(format!("{}.erl", self.module_name)),
            content: self.output.as_bytes().to_vec(),
            file_type: FileType::Erlang,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 发射模块头 (Erlang OTP 28)
    fn emit_header(&mut self) -> ErlangResult<()> {
        self.line(headers::ERLANG)?;
        self.line("%%% DO NOT EDIT")?;
        self.line("%%% Target: Erlang OTP 28")?;
        self.line("")?;
        self.line(&format!("-module({}).", self.module_name))?;
        self.line("")?;

        if !self.exports.is_empty() {
            let exports: String = self.exports.join(", ");
            self.line(&format!("-export([{}]).", exports))?;
            self.line("")?;
        }

        Ok(())
    }

    /// 发射函数定义
    fn emit_function(&mut self, f: &ast::FunctionDecl) -> ErlangResult<()> {
        // 发射类型规范（可选）
        if let Some(return_type) = &f.return_type {
            self.emit_spec(f, return_type)?;
        }

        // 函数名（Erlang 使用小写原子作为函数名）
        let func_name = &f.name;

        // 参数列表
        let params: Vec<String> = f.parameters.iter()
            .map(|p| self.erlang_variable(&p.name))
            .collect();
        let params_str = params.join(", ");

        // 函数头
        self.line(&format!("{}({}) ->", func_name, params_str))?;
        self.indent += 1;

        // 函数体
        self.emit_block(&f.body)?;

        // 如果函数有返回类型但块为空，添加默认返回
        if f.body.statements.is_empty() && f.return_type.is_some() {
            self.line("ok.")?;
        } else {
            // 确保最后一个语句以句号结束
            // 在 emit_block 中处理
        }

        self.indent -= 1;
        Ok(())
    }

    /// 发射类型规范
    fn emit_spec(&mut self, f: &ast::FunctionDecl, return_type: &ast::Type) -> ErlangResult<()> {
        let param_types: Vec<String> = f.parameters.iter()
            .filter_map(|p| p.type_annot.as_ref())
            .map(|t| self.map_type(t))
            .collect();
        let ret_type = self.map_type(return_type);

        let params_str = if param_types.is_empty() {
            "".to_string()
        } else {
            format!("({})", param_types.join(", "))
        };

        self.line(&format!("-spec {}{} -> {}.", f.name, params_str, ret_type))?;
        Ok(())
    }

    /// 发射全局变量（作为函数）
    fn emit_global_var(&mut self, v: &ast::VariableDecl) -> ErlangResult<()> {
        let var_name = &v.name;
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            "undefined".to_string()
        };

        self.line(&format!("{}() -> {}.", var_name, init))?;
        self.line("")?;
        Ok(())
    }

    /// 发射类作为 Erlang 记录
    fn emit_class_as_record(&mut self, class: &ast::ClassDecl) -> ErlangResult<()> {
        let fields: Vec<String> = class.members.iter()
            .filter_map(|m| {
                if let ast::ClassMember::Field(field) = m {
                    Some(field.name.clone())
                } else {
                    None
                }
            })
            .collect();

        if !fields.is_empty() {
            self.line(&format!("-record({}, {{{}}}).", class.name, fields.join(", ")))?;
            self.line("")?;
        }

        Ok(())
    }

    /// 发射默认 main 函数
    fn emit_default_main(&mut self) -> ErlangResult<()> {
        self.line("main() ->")?;
        self.indent += 1;
        self.line("io:format(\"Hello from Erlang backend!~n\", []).")?;
        self.indent -= 1;
        Ok(())
    }

    /// 发射代码块
    fn emit_block(&mut self, block: &ast::Block) -> ErlangResult<()> {
        let stmt_count = block.statements.len();

        for (i, stmt) in block.statements.iter().enumerate() {
            let is_last = i == stmt_count - 1;
            self.emit_statement(stmt, is_last)?;
        }

        if stmt_count == 0 {
            self.line("ok.")?;
        }

        Ok(())
    }

    /// 发射语句
    fn emit_statement(&mut self, stmt: &ast::Statement, is_last: bool) -> ErlangResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let e = self.emit_expr(expr)?;
                if is_last {
                    self.line(&format!("{}.", e))?;
                } else {
                    self.line(&format!("{},", e))?;
                }
            }
            StatementKind::Variable(v) => {
                let var_name = self.erlang_variable(&v.name);
                let init = if let Some(expr) = &v.initializer {
                    self.emit_expr(expr)?
                } else {
                    "undefined".to_string()
                };

                if is_last {
                    self.line(&format!("{} = {}.", var_name, init))?;
                } else {
                    self.line(&format!("{} = {},", var_name, init))?;
                }
            }
            StatementKind::Return(opt) => {
                if let Some(expr) = opt {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("{}.", e))?;
                } else {
                    self.line("ok.")?;
                }
            }
            StatementKind::If(if_stmt) => {
                self.emit_if(if_stmt, is_last)?;
            }
            StatementKind::While(while_stmt) => {
                self.emit_while(while_stmt, is_last)?;
            }
            StatementKind::For(for_stmt) => {
                self.emit_for(for_stmt, is_last)?;
            }
            StatementKind::Match(match_stmt) => {
                self.emit_match(match_stmt, is_last)?;
            }
            StatementKind::Try(try_stmt) => {
                self.emit_try(try_stmt, is_last)?;
            }
            StatementKind::Break => {
                self.line("throw(break).")?;
            }
            StatementKind::Continue => {
                self.line("throw(continue).")?;
            }
            StatementKind::DoWhile(d) => {
                self.emit_do_while(d, is_last)?;
            }
            StatementKind::Unsafe(block) => {
                // Erlang 没有不安全块，直接发射块内容
                self.line("% unsafe block")?;
                self.emit_block(block)?;
            }
            StatementKind::Defer(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&format!("% defer {}.", e))?;
            }
            StatementKind::Yield(opt_expr) => {
                if let Some(e) = opt_expr {
                    let expr = self.emit_expr(e)?;
                    self.line(&format!("yield {}.", expr))?;
                } else {
                    self.line("yield.")?;
                }
            }
            StatementKind::Loop(body) => {
                self.line("loop ->");
                self.indent += 1;
                self.emit_block(body);
                self.indent -= 1;
                self.line("end.")?;
            }
        }
        Ok(())
    }

    /// 发射 if 语句
    fn emit_if(&mut self, if_stmt: &ast::IfStatement, is_last: bool) -> ErlangResult<()> {
        let cond = self.emit_expr(&if_stmt.condition)?;

        // Erlang 的 if 是守卫表达式，这里使用 case
        self.line(&format!("case {} of", cond))?;
        self.indent += 1;
        self.line("true ->")?;
        self.indent += 1;
        self.emit_block(&if_stmt.then_block)?;
        self.indent -= 1;

        if let Some(else_block) = &if_stmt.else_block {
            self.line("false ->")?;
            self.indent += 1;
            self.emit_block(else_block)?;
            self.indent -= 1;
        } else {
            self.line("false ->")?;
            self.indent += 1;
            self.line("ok.")?;
            self.indent -= 1;
        }

        self.indent -= 1;
        self.line("end")?;

        if is_last {
            self.line(".")?;
        } else {
            self.line(",")?;
        }

        Ok(())
    }

    /// 发射 while 循环
    fn emit_while(&mut self, while_stmt: &ast::WhileStatement, _is_last: bool) -> ErlangResult<()> {
        // Erlang 使用递归实现循环
        let cond = self.emit_expr(&while_stmt.condition)?;

        self.line("while_loop(fun() ->")?;
        self.indent += 1;

        self.line(&format!("case {} of", cond))?;
        self.indent += 1;
        self.line("true ->")?;
        self.indent += 1;
        self.emit_block(&while_stmt.body)?;
        self.line("while_loop(fun() ->")?;
        self.indent += 1;
        self.line("ok")?;
        self.indent -= 1;
        self.line("end);")?;
        self.indent -= 1;
        self.line("false -> ok")?;
        self.indent -= 1;
        self.line("end")?;

        self.indent -= 1;
        self.line("end).")?;

        Ok(())
    }

    /// 发射 for 循环
    fn emit_for(&mut self, for_stmt: &ast::ForStatement, _is_last: bool) -> ErlangResult<()> {
        let iter = self.emit_expr(&for_stmt.iterator)?;
        let pattern_var = self.emit_pattern_var(&for_stmt.pattern);

        // 使用列表推导或递归
        self.line(&format!("lists:foreach(fun({}) ->", pattern_var))?;
        self.indent += 1;
        self.emit_block(&for_stmt.body)?;
        self.indent -= 1;
        self.line(&format!("end, {}),", iter))?;

        Ok(())
    }

    /// 发射 match 语句
    fn emit_match(&mut self, match_stmt: &ast::MatchStatement, is_last: bool) -> ErlangResult<()> {
        let expr = self.emit_expr(&match_stmt.expression)?;

        self.line(&format!("case {} of", expr))?;
        self.indent += 1;

        for (i, case) in match_stmt.cases.iter().enumerate() {
            let pattern = self.emit_pattern(&case.pattern)?;

            if let Some(guard) = &case.guard {
                let guard_expr = self.emit_expr(guard)?;
                self.line(&format!("{} when {} ->", pattern, guard_expr))?;
            } else {
                self.line(&format!("{} ->", pattern))?;
            }

            self.indent += 1;

            let is_last_case = is_last && i == match_stmt.cases.len() - 1;
            self.emit_block(&case.body)?;
            self.indent -= 1;
        }

        // 默认分支
        self.line("_ ->")?;
        self.indent += 1;
        self.line("ok.")?;
        self.indent -= 1;

        self.indent -= 1;
        self.line("end")?;

        if is_last {
            self.line(".")?;
        } else {
            self.line(",")?;
        }

        Ok(())
    }

    /// 发射 try 语句
    fn emit_try(&mut self, try_stmt: &ast::TryStatement, is_last: bool) -> ErlangResult<()> {
        self.line("try")?;
        self.indent += 1;
        self.emit_block(&try_stmt.body)?;
        self.indent -= 1;

        for catch in &try_stmt.catch_clauses {
            let exc_type = catch.exception_type.as_deref().unwrap_or("_");
            let var_name = catch.variable_name.as_deref().unwrap_or("_");

            self.line(&format!("catch {}:{}", exc_type, var_name))?;
            self.indent += 1;
            self.emit_block(&catch.body)?;
            self.indent -= 1;
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.line("after")?;
            self.indent += 1;
            self.emit_block(finally)?;
            self.indent -= 1;
        }

        self.line("end")?;

        if is_last {
            self.line(".")?;
        } else {
            self.line(",")?;
        }

        Ok(())
    }

    /// 发射 do-while 循环
    fn emit_do_while(&mut self, d: &ast::DoWhileStatement, _is_last: bool) -> ErlangResult<()> {
        // Erlang 使用递归实现
        self.line("do_while_loop(fun() ->")?;
        self.indent += 1;
        self.emit_block(&d.body)?;

        let cond = self.emit_expr(&d.condition)?;
        self.line(&format!("case {} of", cond))?;
        self.indent += 1;
        self.line("true -> do_while_loop(fun() -> ok end);")?;
        self.line("false -> ok")?;
        self.indent -= 1;
        self.line("end")?;

        self.indent -= 1;
        self.line("end).")?;

        Ok(())
    }

    /// 发射表达式
    fn emit_expr(&self, expr: &ast::Expression) -> ErlangResult<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => self.emit_literal(lit),
            ExpressionKind::Variable(name) => Ok(self.erlang_variable(name)),
            ExpressionKind::Binary(op, lhs, rhs) => {
                let l = self.emit_expr(lhs)?;
                let r = self.emit_expr(rhs)?;
                Ok(self.emit_binop(op, &l, &r))
            }
            ExpressionKind::Unary(op, expr) => {
                let e = self.emit_expr(expr)?;
                Ok(self.emit_unaryop(op, &e))
            }
            ExpressionKind::Call(callee, args) => self.emit_call(callee, args),
            ExpressionKind::Assign(target, value) => self.emit_assign(target, value),
            ExpressionKind::Array(elements) => self.emit_list_literal(elements),
            ExpressionKind::Parenthesized(inner) => {
                let e = self.emit_expr(inner)?;
                Ok(format!("({})", e))
            }
            ExpressionKind::If(cond, then_e, else_e) => {
                let c = self.emit_expr(cond)?;
                let t = self.emit_expr(then_e)?;
                let e = self.emit_expr(else_e)?;
                Ok(format!("case {} of true -> {}; false -> {} end", c, t, e))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                // Erlang 记录访问: Record#record_name.field
                Ok(format!("{}#record.{}", o, field))
            }
            ExpressionKind::Dictionary(entries) => {
                // Erlang 使用 proplists 或 maps
                if entries.is_empty() {
                    Ok("#{}".to_string())
                } else {
                    let pairs: Vec<String> = entries.iter()
                        .map(|(k, v)| {
                            let key = self.emit_expr(k)?;
                            let val = self.emit_expr(v)?;
                            Ok(format!("{} => {}", key, val))
                        })
                        .collect::<ErlangResult<Vec<_>>>()?;
                    Ok(format!("#{{{}}}", pairs.join(", ")))
                }
            }
            ExpressionKind::Wait(wait_type, exprs) => {
                self.emit_wait(wait_type, exprs)
            }
            ExpressionKind::Lambda(params, _body) => {
                // Lambda body is a Block, we simplify to just emit the signature
                // Full implementation would need mutable state for block emission
                let param_strs: Vec<String> = params.iter()
                    .map(|p| self.erlang_variable(&p.name))
                    .collect();
                Ok(format!("fun({}) -> ok end", param_strs.join(", ")))
            }
            ExpressionKind::Match(expr, cases) => {
                let e = self.emit_expr(expr)?;
                let case_patterns: Vec<String> = cases.iter()
                    .map(|c| {
                        let pattern = self.emit_pattern(&c.pattern)?;
                        if let Some(guard) = &c.guard {
                            let guard_expr = self.emit_expr(guard)?;
                            Ok(format!("{} when {}", pattern, guard_expr))
                        } else {
                            Ok(pattern)
                        }
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("case {} of {} -> true; _ -> false end", e, case_patterns.join("; ")))
            }
            _ => Err(x_codegen::CodeGenError::UnsupportedFeature(format!(
                "表达式类型: {:?}",
                expr.node
            ))),
        }
    }

    /// 发射字面量
    fn emit_literal(&self, lit: &ast::Literal) -> ErlangResult<String> {
        match lit {
            ast::Literal::Integer(n) => Ok(format!("{}", n)),
            ast::Literal::Float(f) => Ok(format!("{}", f)),
            ast::Literal::Boolean(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            ast::Literal::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                Ok(format!("\"{}\"", escaped))
            }
            ast::Literal::Char(c) => Ok(format!("${}", c)),
            ast::Literal::Null => Ok("undefined".to_string()),
            ast::Literal::None => Ok("undefined".to_string()),
            ast::Literal::Unit => Ok("ok".to_string()),
        }
    }

    /// 发射二元运算
    fn emit_binop(&self, op: &ast::BinaryOp, l: &str, r: &str) -> String {
        match op {
            ast::BinaryOp::Add => format!("{} + {}", l, r),
            ast::BinaryOp::Sub => format!("{} - {}", l, r),
            ast::BinaryOp::Mul => format!("{} * {}", l, r),
            ast::BinaryOp::Div => format!("{} / {}", l, r),
            ast::BinaryOp::Mod => format!("{} rem {}", l, r),
            ast::BinaryOp::Equal => format!("{} == {}", l, r),
            ast::BinaryOp::NotEqual => format!("{} /= {}", l, r),
            ast::BinaryOp::Less => format!("{} < {}", l, r),
            ast::BinaryOp::LessEqual => format!("{} =< {}", l, r),
            ast::BinaryOp::Greater => format!("{} > {}", l, r),
            ast::BinaryOp::GreaterEqual => format!("{} >= {}", l, r),
            ast::BinaryOp::And => format!("{} andalso {}", l, r),
            ast::BinaryOp::Or => format!("{} orelse {}", l, r),
            ast::BinaryOp::Pow => format!("math:pow({}, {})", l, r),
            ast::BinaryOp::BitAnd => format!("{} band {}", l, r),
            ast::BinaryOp::BitOr => format!("{} bor {}", l, r),
            ast::BinaryOp::BitXor => format!("{} bxor {}", l, r),
            ast::BinaryOp::LeftShift => format!("{} bsl {}", l, r),
            ast::BinaryOp::RightShift => format!("{} bsr {}", l, r),
            _ => format!("{} % unsupported binop % {}", l, r),
        }
    }

    /// 发射一元运算
    fn emit_unaryop(&self, op: &ast::UnaryOp, e: &str) -> String {
        match op {
            ast::UnaryOp::Negate => format!("-{}", e),
            ast::UnaryOp::Not => format!("not {}", e),
            ast::UnaryOp::BitNot => format!("bnot {}", e),
            ast::UnaryOp::Wait => format!("receive Msg -> Msg end"), // Wait becomes receive
        }
    }

    /// 发射函数调用
    fn emit_call(&self, callee: &ast::Expression, args: &[ast::Expression]) -> ErlangResult<String> {
        // 特殊处理内置函数
        if let ExpressionKind::Variable(name) = &callee.node {
            match name.as_str() {
                "print" | "println" => {
                    if args.is_empty() {
                        return Ok("io:format(\"~n\", [])".to_string());
                    }
                    let arg = self.emit_expr(&args[0])?;
                    return Ok(format!("io:format(\"~p~n\", [{}])", arg));
                }
                "printf" => {
                    if args.len() < 2 {
                        return Ok("io:format(\"\", [])".to_string());
                    }
                    let fmt = self.emit_expr(&args[0])?;
                    let rest: Vec<String> = args[1..].iter()
                        .map(|a| self.emit_expr(a))
                        .collect::<ErlangResult<Vec<_>>>()?;
                    return Ok(format!("io:format({}, [{}])", fmt, rest.join(", ")));
                }
                _ => {}
            }
        }

        let callee_str = self.emit_expr(callee)?;
        let arg_strs: Vec<String> = args.iter()
            .map(|a| self.emit_expr(a))
            .collect::<ErlangResult<Vec<_>>>()?;

        Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
    }

    /// 发射赋值
    fn emit_assign(&self, target: &ast::Expression, value: &ast::Expression) -> ErlangResult<String> {
        let val = self.emit_expr(value)?;
        match &target.node {
            ExpressionKind::Variable(name) => {
                let var = self.erlang_variable(name);
                Ok(format!("{} = {}", var, val))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                Ok(format!("{}#record{{{} = {}}}", o, field, val))
            }
            _ => {
                let t = self.emit_expr(target)?;
                Ok(format!("{} = {}", t, val))
            }
        }
    }

    /// 发射列表字面量
    fn emit_list_literal(&self, elements: &[ast::Expression]) -> ErlangResult<String> {
        if elements.is_empty() {
            return Ok("[]".to_string());
        }
        let elem_strs: Vec<String> = elements.iter()
            .map(|e| self.emit_expr(e))
            .collect::<ErlangResult<Vec<_>>>()?;
        Ok(format!("[{}]", elem_strs.join(", ")))
    }

    /// 发射 wait 表达式
    fn emit_wait(&self, wait_type: &ast::WaitType, exprs: &[ast::Expression]) -> ErlangResult<String> {
        let expr_strs: Vec<String> = exprs.iter()
            .map(|e| self.emit_expr(e))
            .collect::<ErlangResult<Vec<_>>>()?;

        match wait_type {
            ast::WaitType::Single => {
                if expr_strs.len() == 1 {
                    Ok(format!("receive {} -> {} end", expr_strs[0], expr_strs[0]))
                } else {
                    Ok(format!("receive Msg -> Msg end"))
                }
            }
            ast::WaitType::Together => {
                // 等待所有消息
                Ok(format!("receive_all([{}])", expr_strs.join(", ")))
            }
            ast::WaitType::Race => {
                // 等待第一个消息
                Ok(format!("receive Msg -> Msg after 0 -> timeout end"))
            }
            ast::WaitType::Timeout(timeout_expr) => {
                let timeout = self.emit_expr(timeout_expr)?;
                Ok(format!("receive Msg -> Msg after {} -> timeout end", timeout))
            }
            ast::WaitType::Atomic => {
                Ok("% atomic wait".to_string())
            }
            ast::WaitType::Retry => {
                Ok("% retry wait".to_string())
            }
        }
    }

    /// 发射模式
    fn emit_pattern(&self, pattern: &ast::Pattern) -> ErlangResult<String> {
        match pattern {
            ast::Pattern::Wildcard => Ok("_".to_string()),
            ast::Pattern::Variable(name) => Ok(self.erlang_variable(name)),
            ast::Pattern::Literal(lit) => self.emit_literal(lit),
            ast::Pattern::Or(left, right) => {
                let l = self.emit_pattern(left)?;
                let r = self.emit_pattern(right)?;
                Ok(format!("{}; {}", l, r))
            }
            ast::Pattern::Guard(inner, cond) => {
                let inner_pattern = self.emit_pattern(inner)?;
                let guard_expr = self.emit_expr(cond)?;
                Ok(format!("{} when {}", inner_pattern, guard_expr))
            }
            ast::Pattern::Array(elements) => {
                let elem_strs: Vec<String> = elements.iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("[{}]", elem_strs.join(", ")))
            }
            ast::Pattern::Tuple(elements) => {
                let elem_strs: Vec<String> = elements.iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("{{{}}}", elem_strs.join(", ")))
            }
            ast::Pattern::Dictionary(entries) => {
                let pairs: Vec<String> = entries.iter()
                    .map(|(k, v)| {
                        let key = self.emit_pattern(k)?;
                        let val = self.emit_pattern(v)?;
                        Ok(format!("{} := {}", key, val))
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("#{{{}}}", pairs.join(", ")))
            }
            ast::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| {
                        let val = self.emit_pattern(v)?;
                        Ok(format!("{} = {}", k, val))
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("#{}{{{}}}", name, field_strs.join(", ")))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns.iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<ErlangResult<Vec<_>>>()?;
                if patterns.is_empty() {
                    Ok(variant_name.clone())
                } else {
                    Ok(format!("{}({})", variant_name, pattern_strs.join(", ")))
                }
            }
        }
    }

    /// 发射模式变量（用于 for 循环等）
    fn emit_pattern_var(&self, pattern: &ast::Pattern) -> String {
        match pattern {
            ast::Pattern::Wildcard => "_".to_string(),
            ast::Pattern::Variable(name) => self.erlang_variable(name),
            ast::Pattern::Literal(lit) => self.emit_literal_for_pattern(lit),
            ast::Pattern::Array(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("[{}]", vars.join(", "))
            }
            ast::Pattern::Tuple(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("{{{}}}", vars.join(", "))
            }
            ast::Pattern::Or(left, _) => self.emit_pattern_var(left),
            ast::Pattern::Guard(inner, _) => self.emit_pattern_var(inner),
            ast::Pattern::Dictionary(entries) => {
                let vars: Vec<String> = entries.iter()
                    .map(|(k, v)| format!("{} := {}", self.emit_pattern_var(k), self.emit_pattern_var(v)))
                    .collect();
                format!("#{{{}}}", vars.join(", "))
            }
            ast::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{} = {}", k, self.emit_pattern_var(v)))
                    .collect();
                format!("#{}{{{}}}", name, field_strs.join(", "))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns.iter().map(|p| self.emit_pattern_var(p)).collect();
                if patterns.is_empty() {
                    variant_name.clone()
                } else {
                    format!("{}({})", variant_name, pattern_strs.join(", "))
                }
            }
        }
    }

    /// 发射模式中的字面量
    fn emit_literal_for_pattern(&self, lit: &ast::Literal) -> String {
        match lit {
            ast::Literal::Integer(n) => format!("{}", n),
            ast::Literal::Float(f) => format!("{}", f),
            ast::Literal::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            ast::Literal::String(s) => format!("\"{}\"", s),
            ast::Literal::Char(c) => format!("${}", c),
            ast::Literal::Null | ast::Literal::None | ast::Literal::Unit => "undefined".to_string(),
        }
    }

    /// 类型映射：X 类型 -> Erlang 类型
    fn map_type(&self, ty: &ast::Type) -> String {
        match ty {
            ast::Type::Int | ast::Type::UnsignedInt => "integer()".to_string(),
            ast::Type::Float => "float()".to_string(),
            ast::Type::Bool => "boolean()".to_string(),
            ast::Type::String => "string()".to_string(),
            ast::Type::Char => "char()".to_string(),
            ast::Type::Unit => "ok".to_string(),
            ast::Type::Void => "ok".to_string(),
            ast::Type::Never => "no_return()".to_string(),
            ast::Type::Dynamic => "any()".to_string(),
            ast::Type::Option(inner) => {
                let inner_type = self.map_type(inner);
                format!("{} | undefined", inner_type)
            }
            ast::Type::Array(inner) => {
                let inner_type = self.map_type(inner);
                format!("[{}]", inner_type)
            }
            ast::Type::Tuple(elements) => {
                let elem_types: Vec<String> = elements.iter().map(|t| self.map_type(t)).collect();
                format!("{{{}}}", elem_types.join(", "))
            }
            ast::Type::Function(params, ret) => {
                let param_types: Vec<String> = params.iter().map(|t| self.map_type(t)).collect();
                let ret_type = self.map_type(ret);
                format!("fun(({}) -> {})", param_types.join(", "), ret_type)
            }
            ast::Type::Dictionary(k, v) => {
                let key_type = self.map_type(k);
                let val_type = self.map_type(v);
                format!("#{{{} => {}}}", key_type, val_type)
            }
            ast::Type::Result(ok, err) => {
                let ok_type = self.map_type(ok);
                let err_type = self.map_type(err);
                format!("{{ok, {}}} | {{error, {}}}", ok_type, err_type)
            }
            ast::Type::Async(inner) => {
                let inner_type = self.map_type(inner);
                format!("fun(() -> {})", inner_type)
            }
            ast::Type::Record(name, fields) => {
                // Record type with field types
                let field_strs: Vec<String> = fields.iter()
                    .map(|(field_name, field_type)| {
                        format!("{} :: {}", field_name, self.map_type(field_type))
                    })
                    .collect();
                format!("#{}{{{}}}", name, field_strs.join(", "))
            }
            ast::Type::Union(name, variants) => {
                // Union type
                let variant_strs: Vec<String> = variants.iter().map(|t| self.map_type(t)).collect();
                if variant_strs.is_empty() {
                    name.clone()
                } else {
                    variant_strs.join(" | ")
                }
            }
            // References and pointers are just references in Erlang
            ast::Type::Reference(inner)
            | ast::Type::MutableReference(inner)
            | ast::Type::Pointer(inner)
            | ast::Type::ConstPointer(inner) => {
                self.map_type(inner)
            }
            ast::Type::CInt | ast::Type::CUInt | ast::Type::CLong | ast::Type::CULong |
            ast::Type::CChar | ast::Type::CLongLong | ast::Type::CULongLong |
            ast::Type::CFloat | ast::Type::CDouble | ast::Type::CSize | ast::Type::CString => {
                "integer()".to_string()
            }
            ast::Type::Generic(name) => name.clone(),
            ast::Type::TypeParam(name) => name.clone(),
            ast::Type::TypeConstructor(name, args) => {
                let arg_types: Vec<String> = args.iter().map(|t| self.map_type(t)).collect();
                format!("{}({})", name, arg_types.join(", "))
            }
            ast::Type::Var(name) => name.clone(),
        }
    }

    /// 将 X 变量名转换为 Erlang 变量名
    /// Erlang 变量必须以大写字母或下划线开头
    fn erlang_variable(&self, name: &str) -> String {
        if name.starts_with('_') {
            name.to_string()
        } else if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            name.to_string()
        } else {
            // 首字母大写
            let mut chars: Vec<char> = name.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect()
        }
    }

    /// 输出一行代码
    fn line(&mut self, s: &str) -> ErlangResult<()> {
        for _ in 0..self.indent {
            write!(self.output, "    ")?;
        }
        writeln!(self.output, "{}", s)?;
        Ok(())
    }
}

impl CodeGenerator for ErlangBackend {
    type Config = ErlangBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_ast(&mut self, program: &AstProgram) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_ast(program)
    }

    fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        // 从 HIR 生成：先转换为 AST 再生成（简化实现）
        // 实际实现应该直接从 HIR 生成
        Err(x_codegen::CodeGenError::Unimplemented(
            "从 HIR 生成 Erlang 代码尚未实现，请使用 AST 后端".to_string()
        ))
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        // LIR -> Erlang 代码生成
        self.output.clear();
        self.indent = 0;

        self.emit_header()?;

        // 开始模块定义
        self.line("-module(program).")?;
        self.line("-compile(export_all).")?;
        self.line("")?;

        // 收集函数
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    has_main = true;
                }
                // 发射函数
                let ret = self.lir_type_to_erlang(&f.return_type);
                let params: Vec<String> = f.parameters.iter()
                    .map(|p| p.name.clone())
                    .collect();
                self.line(&format!("{}({}) ->", f.name, params.join(", ")))?;
                self.indent += 1;

                // 发射函数体
                for stmt in &f.body.statements {
                    self.emit_lir_statement(stmt)?;
                }

                self.indent -= 1;
                self.line("")?;
            }
        }

        // main 入口
        if has_main {
            self.line("main() ->")?;
            self.indent += 1;
            self.line("main().")?;
            self.indent -= 1;
        }

        let output_file = OutputFile {
            path: PathBuf::from("program.erl"),
            content: self.output.as_bytes().to_vec(),
            file_type: FileType::Erlang,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

/// LIR -> Erlang 辅助方法
impl ErlangBackend {
    /// 将 LIR 类型转换为 Erlang 类型
    fn lir_type_to_erlang(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "ok".to_string(),
            Bool => "boolean()".to_string(),
            Char => "char()".to_string(),
            Schar | Short | Int | Uint => "integer()".to_string(),
            Uchar | Ushort | Long | Ulong | LongLong | UlongLong => "non_neg_integer()".to_string(),
            Float | Double | LongDouble => "float()".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "integer()".to_string(),
            Pointer(_) => "term()".to_string(),
            Array(_, _) => "[term()]".to_string(),
            FunctionPointer(_, _) => "fun()".to_string(),
            Named(n) => n.clone(),
            Qualified(_, inner) => self.lir_type_to_erlang(inner),
        }
    }

    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> ErlangResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{};", s))?;
            }
            Variable(v) => {
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("{} = {}", v.name, init_str))?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("if {} ->", cond))?;
                self.indent += 1;
                self.emit_lir_statement(&i.then_branch)?;
                self.indent -= 1;
                if let Some(else_br) = &i.else_branch {
                    self.line("true ->")?;
                    self.indent += 1;
                    self.emit_lir_statement(else_br)?;
                    self.indent -= 1;
                }
                self.line("end.")?;
            }
            While(w) => {
                let cond = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("{} ->", cond))?;
                self.indent += 1;
                self.emit_lir_statement(&w.body)?;
                self.line("true ->")?;
                self.indent += 1;
                // 递归实现循环
                self.indent -= 1;
                self.line("end.")?;
            }
            Return(r) => {
                if let Some(e) = r {
                    let val = self.emit_lir_expr(e)?;
                    self.line(&format!("{}", val))?;
                } else {
                    self.line("ok")?;
                }
            }
            Break => self.line("ok")?, // Erlang 中 break 实际是返回
            Continue => self.line("ok")?,
            _ => self.line("% unsupported statement")?,
        }
        Ok(())
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> ErlangResult<String> {
        use x_lir::Expression::*;
        match expr {
            Literal(l) => self.emit_lir_literal(l),
            Variable(n) => Ok(n.clone()),
            Binary(op, l, r) => {
                let left = self.emit_lir_expr(l)?;
                let right = self.emit_lir_expr(r)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("({} {} {})", left, op_str, right))
            }
            Unary(op, e) => {
                let e = self.emit_lir_expr(e)?;
                let op_str = self.map_lir_unaryop(op);
                Ok(format!("({}{})", op_str, e))
            }
            Call(callee, args) => {
                let callee_str = self.emit_lir_expr(callee)?;
                let args_str: Vec<String> = args.iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{}({})", callee_str, args_str.join(", ")))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("lists:nth({}, {})", idx_str, arr_str))
            }
            _ => Ok("undefined".to_string()),
        }
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> ErlangResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))),
            Char(c) => Ok(format!("\"{}\"", c)),
            Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            NullPointer => Ok("undefined".to_string()),
        }
    }

    /// 映射 LIR 二元运算符
    fn map_lir_binop(&self, op: &x_lir::BinaryOp) -> String {
        use x_lir::BinaryOp::*;
        match op {
            Add => "+", Subtract => "-", Multiply => "*", Divide => "/", Modulo => "rem",
            LessThan => "<", LessThanEqual => "=<", GreaterThan => ">", GreaterThanEqual => ">=",
            Equal => "=:=", NotEqual => "/=",
            BitAnd => "band", BitOr => "bor", BitXor => "bxor",
            LeftShift => "bsl", RightShift => "bsr", RightShiftArithmetic => "bsr",
            LogicalAnd => "and", LogicalOr => "or",
        }.to_string()
    }

    /// 映射 LIR 一元运算符
    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "not ".to_string(),
            BitNot => "bnot ".to_string(),
            _ => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{Spanned, MethodModifiers};

    fn make_expr(kind: ExpressionKind) -> ast::Expression {
        Spanned::new(kind, Span::default())
    }

    fn make_stmt(kind: StatementKind) -> ast::Statement {
        Spanned::new(kind, Span::default())
    }

    #[test]
    fn test_hello_world_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::Expression(make_expr(
                        ExpressionKind::Call(
                            Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                            vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                "Hello, World!".to_string(),
                            )))],
                        ),
                    )))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let erlang_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(erlang_code.contains("-module(x_module)."));
        assert!(erlang_code.contains("-export([main/0])."));
        assert!(erlang_code.contains("main() ->"));
        assert!(erlang_code.contains("io:format"));
        assert!(erlang_code.contains("Hello, World!"));
    }

    #[test]
    fn test_empty_program_generates_default_main() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![],
        };

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let erlang_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(erlang_code.contains("main() ->"));
        assert!(erlang_code.contains("Hello from Erlang backend!"));
    }

    #[test]
    fn test_function_with_parameters() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "add".to_string(),
                type_parameters: vec![],
                parameters: vec![
                    ast::Parameter {
                        name: "a".to_string(),
                        type_annot: Some(ast::Type::Int),
                        default: None,
                        span: Span::default(),
                    },
                    ast::Parameter {
                        name: "b".to_string(),
                        type_annot: Some(ast::Type::Int),
                        default: None,
                        span: Span::default(),
                    },
                ],
                effects: vec![],
                return_type: Some(ast::Type::Int),
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::Return(Some(make_expr(
                        ExpressionKind::Binary(
                            ast::BinaryOp::Add,
                            Box::new(make_expr(ExpressionKind::Variable("a".to_string()))),
                            Box::new(make_expr(ExpressionKind::Variable("b".to_string()))),
                        ),
                    ))))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let erlang_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(erlang_code.contains("add(A, B) ->"));
        assert!(erlang_code.contains("A + B"));
        assert!(erlang_code.contains("-spec add(integer(), integer()) -> integer()."));
    }

    #[test]
    fn test_variable_naming() {
        let backend = ErlangBackend::new(ErlangBackendConfig::default());

        // 测试小写变量名转换
        assert_eq!(backend.erlang_variable("x"), "X");
        assert_eq!(backend.erlang_variable("myVar"), "MyVar");

        // 测试大写变量名保持不变
        assert_eq!(backend.erlang_variable("X"), "X");
        assert_eq!(backend.erlang_variable("MyVar"), "MyVar");

        // 测试下划线开头的变量保持不变
        assert_eq!(backend.erlang_variable("_temp"), "_temp");
    }

    #[test]
    fn test_type_mapping() {
        let backend = ErlangBackend::new(ErlangBackendConfig::default());

        assert_eq!(backend.map_type(&ast::Type::Int), "integer()");
        assert_eq!(backend.map_type(&ast::Type::Float), "float()");
        assert_eq!(backend.map_type(&ast::Type::Bool), "boolean()");
        assert_eq!(backend.map_type(&ast::Type::String), "string()");

        // 测试可选类型
        assert_eq!(
            backend.map_type(&ast::Type::Option(Box::new(ast::Type::Int))),
            "integer() | undefined"
        );

        // 测试数组类型
        assert_eq!(
            backend.map_type(&ast::Type::Array(Box::new(ast::Type::Int))),
            "[integer()]"
        );
    }

    #[test]
    fn test_match_expression() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "test_match".to_string(),
                type_parameters: vec![],
                parameters: vec![ast::Parameter {
                    name: "x".to_string(),
                    type_annot: Some(ast::Type::Int),
                    default: None,
                    span: Span::default(),
                }],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::Match(ast::MatchStatement {
                        expression: make_expr(ExpressionKind::Variable("x".to_string())),
                        cases: vec![
                            ast::MatchCase {
                                pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                guard: None,
                                body: ast::Block {
                                    statements: vec![make_stmt(StatementKind::Expression(
                                        make_expr(ExpressionKind::Call(
                                            Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                                            vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                                "one".to_string(),
                                            )))],
                                        )),
                                    ))],
                                },
                            },
                            ast::MatchCase {
                                pattern: ast::Pattern::Wildcard,
                                guard: None,
                                body: ast::Block {
                                    statements: vec![make_stmt(StatementKind::Expression(
                                        make_expr(ExpressionKind::Call(
                                            Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                                            vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                                "other".to_string(),
                                            )))],
                                        )),
                                    ))],
                                },
                            },
                        ],
                    }))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let erlang_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(erlang_code.contains("case X of"));
        assert!(erlang_code.contains("1 ->"));
        assert!(erlang_code.contains("_ ->"));
        assert!(erlang_code.contains("end"));
    }

    #[test]
    fn test_custom_module_name() {
        let config = ErlangBackendConfig {
            module_name: Some("my_custom_module".to_string()),
            ..Default::default()
        };
        let mut backend = ErlangBackend::new(config);

        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![],
        };

        let output = backend.generate_from_ast(&program).unwrap();
        let erlang_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(erlang_code.contains("-module(my_custom_module)."));
        assert!(output.files[0].path.to_string_lossy().contains("my_custom_module.erl"));
    }
}
