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

#![allow(
    clippy::if_same_then_else,
    clippy::only_used_in_recursion,
    clippy::useless_format
)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;
use x_parser::ast::{self, ExpressionKind, Program as AstProgram, StatementKind};

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
    #[allow(dead_code)]
    config: ErlangBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
    module_name: String,
    exports: Vec<String>,
    /// 用于生成唯一的 while/do-while/loop 辅助函数名
    loop_counter: usize,
}

pub type ErlangResult<T> = Result<T, x_codegen::CodeGenError>;

impl ErlangBackend {
    pub fn new(config: ErlangBackendConfig) -> Self {
        let module_name = config
            .module_name
            .clone()
            .unwrap_or_else(|| "x_module".to_string());
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            module_name,
            exports: Vec::new(),
            loop_counter: 0,
        }
    }

    fn next_loop_id(&mut self) -> usize {
        self.loop_counter += 1;
        self.loop_counter
    }

    fn line(&mut self, s: &str) -> ErlangResult<()> {
        self.buffer
            .line(s)
            .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
    }

    fn indent(&mut self) {
        self.buffer.indent();
    }
    fn dedent(&mut self) {
        self.buffer.dedent();
    }
    fn output(&self) -> &str {
        self.buffer.as_str()
    }

    /// 从 AST 生成 Erlang 代码
    pub fn generate_from_ast(&mut self, program: &AstProgram) -> ErlangResult<CodegenOutput> {
        self.buffer.clear();
        self.exports.clear();

        // Single pass to collect exports and categorize declarations
        let mut has_main = false;
        let mut functions = Vec::new();
        let mut global_vars = Vec::new();
        let mut classes = Vec::new();

        for decl in &program.declarations {
            match decl {
                ast::Declaration::Function(f) => {
                    let arity = f.parameters.len();
                    self.exports.push(format!("{}/{}", f.name, arity));
                    if f.name == "main" {
                        has_main = true;
                    }
                    functions.push(f);
                }
                ast::Declaration::Variable(v) => {
                    self.exports.push(format!("{}/0", v.name));
                    global_vars.push(v);
                }
                ast::Declaration::Class(class) => classes.push(class),
                _ => {}
            }
        }

        // 发射模块头
        self.emit_header()?;

        // 发射函数
        for f in &functions {
            self.emit_function(f)?;
            self.line("")?;
        }

        // 全局变量作为函数导出
        for v in &global_vars {
            self.emit_global_var(v)?;
        }

        // 类作为记录导出
        for class in &classes {
            self.emit_class_as_record(class)?;
        }

        // 如果没有 main 函数，生成默认的
        if !has_main {
            self.emit_default_main()?;
        }

        // 创建输出文件
        let output_file = OutputFile {
            path: std::path::PathBuf::from(format!("{}.erl", self.module_name)),
            content: self.output().as_bytes().to_vec(),
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
        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| self.erlang_variable(&p.name))
            .collect();
        let params_str = params.join(", ");

        // 函数头
        self.line(&format!("{}({}) ->", func_name, params_str))?;
        self.indent();

        // 函数体
        self.emit_block(&f.body)?;

        // 如果函数有返回类型但块为空，添加默认返回
        if f.body.statements.is_empty() && f.return_type.is_some() {
            self.line("ok.")?;
        } else {
            // 确保最后一个语句以句号结束
            // 在 emit_block 中处理
        }

        self.dedent();
        Ok(())
    }

    /// 发射类型规范
    fn emit_spec(&mut self, f: &ast::FunctionDecl, return_type: &ast::Type) -> ErlangResult<()> {
        let param_types: Vec<String> = f
            .parameters
            .iter()
            .filter_map(|p| p.type_annot.as_ref())
            .map(|t| self.map_type(t))
            .collect();
        let ret_type = self.map_type(return_type);

        let params_str = if param_types.is_empty() {
            "()".to_string()
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
        let fields: Vec<String> = class
            .members
            .iter()
            .filter_map(|m| {
                if let ast::ClassMember::Field(field) = m {
                    Some(field.name.clone())
                } else {
                    None
                }
            })
            .collect();

        if !fields.is_empty() {
            self.line(&format!(
                "-record({}, {{{}}}).",
                class.name,
                fields.join(", ")
            ))?;
            self.line("")?;
        }

        Ok(())
    }

    /// 发射默认 main 函数
    fn emit_default_main(&mut self) -> ErlangResult<()> {
        self.line("main() ->")?;
        self.indent();
        self.line("io:format(\"Hello from Erlang backend!~n\", []).")?;
        self.dedent();
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
                let id = self.next_loop_id();
                let label = format!("__x_loop_{}", id);
                self.line(&format!("{}() ->", label))?;
                self.indent();
                for stmt in &body.statements {
                    self.emit_statement(stmt, false)?;
                }
                self.line(&format!("{}().", label))?;
                self.dedent();
                self.line(&format!("{}().", label))?;
                if is_last {
                    self.line("ok.")?;
                } else {
                    self.line("ok,")?;
                }
            }
            StatementKind::WhenGuard(condition, body_expr) => {
                let cond = self.emit_expr(condition)?;
                let body_str = self.emit_expr(body_expr)?;
                self.line(&format!("case {} of", cond))?;
                self.indent();
                self.line(&format!("true -> {};", body_str))?;
                self.line("false -> ok")?;
                self.dedent();
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
        self.indent();
        self.line("true ->")?;
        self.indent();
        self.emit_block(&if_stmt.then_block)?;
        self.dedent();

        if let Some(else_block) = &if_stmt.else_block {
            self.line("false ->")?;
            self.indent();
            self.emit_block(else_block)?;
            self.dedent();
        } else {
            self.line("false ->")?;
            self.indent();
            self.line("ok.")?;
            self.dedent();
        }

        self.dedent();
        self.line("end")?;

        if is_last {
            self.line(".")?;
        } else {
            self.line(",")?;
        }

        Ok(())
    }

    /// 发射 while 循环（尾递归命名函数，条件每次重新求值）
    fn emit_while(&mut self, while_stmt: &ast::WhileStatement, is_last: bool) -> ErlangResult<()> {
        let id = self.next_loop_id();
        let label = format!("__x_while_{}", id);
        let cond = self.emit_expr(&while_stmt.condition)?;

        self.line(&format!("{}() ->", label))?;
        self.indent();
        self.line(&format!("case {} of", cond))?;
        self.indent();
        self.line("true ->")?;
        self.indent();
        for stmt in &while_stmt.body.statements {
            self.emit_statement(stmt, false)?;
        }
        self.line(&format!("{}();", label))?;
        self.dedent();
        self.line("false ->")?;
        self.indent();
        self.line("ok")?;
        self.dedent();
        self.dedent();
        self.line("end.")?;
        self.dedent();
        self.line(&format!("{}().", label))?;

        if is_last {
            self.line("ok.")?;
        } else {
            self.line("ok,")?;
        }
        Ok(())
    }

    /// 发射 for 循环（`lists:foreach/2`）
    fn emit_for(&mut self, for_stmt: &ast::ForStatement, is_last: bool) -> ErlangResult<()> {
        let iter = self.emit_expr(&for_stmt.iterator)?;
        let pattern_var = self.emit_pattern_var(&for_stmt.pattern);

        self.line(&format!("lists:foreach(fun({}) ->", pattern_var))?;
        self.indent();
        for stmt in &for_stmt.body.statements {
            self.emit_statement(stmt, false)?;
        }
        self.line("ok")?;
        self.dedent();
        self.line(&format!("end, {}),", iter))?;

        if is_last {
            self.line("ok.")?;
        } else {
            self.line("ok,")?;
        }
        Ok(())
    }

    /// 发射 match 语句
    fn emit_match(&mut self, match_stmt: &ast::MatchStatement, is_last: bool) -> ErlangResult<()> {
        let expr = self.emit_expr(&match_stmt.expression)?;

        self.line(&format!("case {} of", expr))?;
        self.indent();

        for case in &match_stmt.cases {
            let pattern = self.emit_pattern(&case.pattern)?;

            if let Some(guard) = &case.guard {
                let guard_expr = self.emit_expr(guard)?;
                self.line(&format!("{} when {} ->", pattern, guard_expr))?;
            } else {
                self.line(&format!("{} ->", pattern))?;
            }

            self.indent();

            self.emit_block(&case.body)?;
            self.dedent();
        }

        // 默认分支
        self.line("_ ->")?;
        self.indent();
        self.line("ok.")?;
        self.dedent();

        self.dedent();
        self.line("end")?;

        if is_last {
            self.line(".")?;
        } else {
            self.line(",")?;
        }

        Ok(())
    }

    /// 发射 try 语句（Erlang `try ... catch ... after ... end`）
    fn emit_try(&mut self, try_stmt: &ast::TryStatement, is_last: bool) -> ErlangResult<()> {
        self.line("try")?;
        self.indent();
        self.emit_begin_block_comma(&try_stmt.body)?;
        self.dedent();

        if !try_stmt.catch_clauses.is_empty() {
            self.line("catch")?;
            self.indent();
            for (ci, catch) in try_stmt.catch_clauses.iter().enumerate() {
                let class = catch
                    .exception_type
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("_");
                let var_raw = catch.variable_name.as_deref().unwrap_or("_");
                let var_erl = self.erlang_variable(var_raw);
                self.line(&format!("{}:{} ->", class, var_erl))?;
                self.indent();
                self.emit_begin_block_comma(&catch.body)?;
                self.dedent();
                if ci + 1 < try_stmt.catch_clauses.len() {
                    self.line(";")?;
                }
            }
            self.dedent();
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.line("after")?;
            self.indent();
            self.emit_begin_block_comma(finally)?;
            self.dedent();
        }

        self.line("end")?;

        if is_last {
            self.line(".")?;
        } else {
            self.line(",")?;
        }

        Ok(())
    }

    /// `begin Stmt1, Stmt2, ok end` — 用于 try/catch 等需要逗号序列的上下文
    fn emit_begin_block_comma(&mut self, block: &ast::Block) -> ErlangResult<()> {
        if block.statements.is_empty() {
            self.line("ok")?;
            return Ok(());
        }
        self.line("begin")?;
        self.indent();
        for stmt in &block.statements {
            self.emit_statement(stmt, false)?;
        }
        self.line("ok")?;
        self.dedent();
        self.line("end")?;
        Ok(())
    }

    /// 发射 do-while 循环
    fn emit_do_while(&mut self, d: &ast::DoWhileStatement, is_last: bool) -> ErlangResult<()> {
        let id = self.next_loop_id();
        let label = format!("__x_do_while_{}", id);
        let cond = self.emit_expr(&d.condition)?;

        self.line(&format!("{}() ->", label))?;
        self.indent();
        for stmt in &d.body.statements {
            self.emit_statement(stmt, false)?;
        }
        self.line(&format!("case {} of", cond))?;
        self.indent();
        self.line(&format!("true -> {}();", label))?;
        self.line("false ->")?;
        self.indent();
        self.line("ok")?;
        self.dedent();
        self.dedent();
        self.line("end.")?;
        self.dedent();
        self.line(&format!("{}().", label))?;

        if is_last {
            self.line("ok.")?;
        } else {
            self.line("ok,")?;
        }
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
            ExpressionKind::Cast(inner, _ty) => self.emit_expr(inner),
            ExpressionKind::Tuple(elements) => {
                let parts: Vec<String> = elements
                    .iter()
                    .map(|e| self.emit_expr(e))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("{{{}}}", parts.join(", ")))
            }
            ExpressionKind::Record(name, fields) => {
                let pairs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| {
                        let val = self.emit_expr(v)?;
                        Ok(format!("{} = {}", k, val))
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("#{}{{{}}}", name, pairs.join(", ")))
            }
            ExpressionKind::Range(start, end, inclusive) => {
                let a = self.emit_expr(start)?;
                let b = self.emit_expr(end)?;
                if *inclusive {
                    Ok(format!("lists:seq({}, {})", a, b))
                } else {
                    Ok(format!("lists:seq({}, {} - 1)", a, b))
                }
            }
            ExpressionKind::Pipe(first, rest) => {
                let mut acc = self.emit_expr(first)?;
                for step in rest {
                    let f = self.emit_expr(step)?;
                    acc = format!("{}({})", f, acc);
                }
                Ok(acc)
            }
            ExpressionKind::Await(inner) => self.emit_expr(inner),
            ExpressionKind::Needs(_) => Ok("ok".to_string()),
            ExpressionKind::Given(_, inner) => self.emit_expr(inner),
            ExpressionKind::Handle(expr, _handlers) => self.emit_expr(expr),
            ExpressionKind::TryPropagate(inner) => {
                let e = self.emit_expr(inner)?;
                Ok(format!(
                    "case {} of {{ok, __V}} -> __V; {{error, __E}} -> erlang:error(__E); __Other -> __Other end",
                    e
                ))
            }
            ExpressionKind::OptionalChain(obj, field) => {
                let o = self.emit_expr(obj)?;
                let atom = self.erlang_field_atom(field);
                Ok(format!(
                    "case {} of undefined -> undefined; __O -> maps:get({}, __O) end",
                    o, atom
                ))
            }
            ExpressionKind::NullCoalescing(a, b) => {
                let left = self.emit_expr(a)?;
                let right = self.emit_expr(b)?;
                Ok(format!(
                    "case {} of undefined -> {}; __X -> __X end",
                    left, right
                ))
            }
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
                let atom = self.erlang_field_atom(field);
                Ok(format!("maps:get({}, {})", atom, o))
            }
            ExpressionKind::Dictionary(entries) => {
                // Erlang 使用 proplists 或 maps
                if entries.is_empty() {
                    Ok("#{}".to_string())
                } else {
                    let pairs: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            let key = self.emit_expr(k)?;
                            let val = self.emit_expr(v)?;
                            Ok(format!("{} => {}", key, val))
                        })
                        .collect::<ErlangResult<Vec<_>>>()?;
                    Ok(format!("#{{{}}}", pairs.join(", ")))
                }
            }
            ExpressionKind::Wait(wait_type, exprs) => self.emit_wait(wait_type, exprs),
            ExpressionKind::Lambda(params, _body) => {
                // Lambda body is a Block, we simplify to just emit the signature
                // Full implementation would need mutable state for block emission
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| self.erlang_variable(&p.name))
                    .collect();
                Ok(format!("fun({}) -> ok end", param_strs.join(", ")))
            }
            ExpressionKind::Match(expr, cases) => {
                let e = self.emit_expr(expr)?;
                let case_patterns: Vec<String> = cases
                    .iter()
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
                Ok(format!(
                    "case {} of {} -> true; _ -> false end",
                    e,
                    case_patterns.join("; ")
                ))
            }
            ExpressionKind::WhenGuard(condition, body_expr) => {
                let cond = self.emit_expr(condition)?;
                let body = self.emit_expr(body_expr)?;
                Ok(format!(
                    "case {} of true -> {}; false -> ok end",
                    cond, body
                ))
            }
            ExpressionKind::Block(_block) => Ok("ok".to_string()),
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
            ast::UnaryOp::Reference => format!("{}", e), // Erlang doesn't have references, just pass through
            ast::UnaryOp::MutableReference => format!("{}", e), // Erlang doesn't have references, just pass through
        }
    }

    /// 发射函数调用
    fn emit_call(
        &self,
        callee: &ast::Expression,
        args: &[ast::Expression],
    ) -> ErlangResult<String> {
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
                    let rest: Vec<String> = args[1..]
                        .iter()
                        .map(|a| self.emit_expr(a))
                        .collect::<ErlangResult<Vec<_>>>()?;
                    return Ok(format!("io:format({}, [{}])", fmt, rest.join(", ")));
                }
                _ => {}
            }
        }

        let callee_str = self.emit_expr(callee)?;
        let arg_strs: Vec<String> = args
            .iter()
            .map(|a| self.emit_expr(a))
            .collect::<ErlangResult<Vec<_>>>()?;

        Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
    }

    /// 发射赋值
    fn emit_assign(
        &self,
        target: &ast::Expression,
        value: &ast::Expression,
    ) -> ErlangResult<String> {
        let val = self.emit_expr(value)?;
        match &target.node {
            ExpressionKind::Variable(name) => {
                let var = self.erlang_variable(name);
                Ok(format!("{} = {}", var, val))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                let atom = self.erlang_field_atom(field);
                Ok(format!("maps:put({}, {}, {})", atom, val, o))
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
        let elem_strs: Vec<String> = elements
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<ErlangResult<Vec<_>>>()?;
        Ok(format!("[{}]", elem_strs.join(", ")))
    }

    /// 发射 wait 表达式
    fn emit_wait(
        &self,
        wait_type: &ast::WaitType,
        exprs: &[ast::Expression],
    ) -> ErlangResult<String> {
        let expr_strs: Vec<String> = exprs
            .iter()
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
            ast::WaitType::Concurrently => {
                // 等待所有消息
                Ok(format!("receive_all([{}])", expr_strs.join(", ")))
            }
            ast::WaitType::Race => {
                // 等待第一个消息
                Ok(format!("receive Msg -> Msg after 0 -> timeout end"))
            }
            ast::WaitType::Timeout(timeout_expr) => {
                let timeout = self.emit_expr(timeout_expr)?;
                Ok(format!(
                    "receive Msg -> Msg after {} -> timeout end",
                    timeout
                ))
            }
            ast::WaitType::Atomic => Ok("% atomic wait".to_string()),
            ast::WaitType::Retry => Ok("% retry wait".to_string()),
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
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("[{}]", elem_strs.join(", ")))
            }
            ast::Pattern::Tuple(elements) => {
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("{{{}}}", elem_strs.join(", ")))
            }
            ast::Pattern::Dictionary(entries) => {
                let pairs: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let key = self.emit_pattern(k)?;
                        let val = self.emit_pattern(v)?;
                        Ok(format!("{} := {}", key, val))
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("#{{{}}}", pairs.join(", ")))
            }
            ast::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| {
                        let val = self.emit_pattern(v)?;
                        Ok(format!("{} = {}", k, val))
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("#{}{{{}}}", name, field_strs.join(", ")))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
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
                let vars: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{} := {}",
                            self.emit_pattern_var(k),
                            self.emit_pattern_var(v)
                        )
                    })
                    .collect();
                format!("#{{{}}}", vars.join(", "))
            }
            ast::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, self.emit_pattern_var(v)))
                    .collect();
                format!("#{}{{{}}}", name, field_strs.join(", "))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> =
                    patterns.iter().map(|p| self.emit_pattern_var(p)).collect();
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
            ast::Type::TypeConstructor(name, args) if name == "Option" && args.len() == 1 => {
                let inner_type = self.map_type(&args[0]);
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
            ast::Type::TypeConstructor(name, args) if name == "Result" && args.len() == 2 => {
                let ok_type = self.map_type(&args[0]);
                let err_type = self.map_type(&args[1]);
                format!("{{ok, {}}} | {{error, {}}}", ok_type, err_type)
            }
            ast::Type::Async(inner) => {
                let inner_type = self.map_type(inner);
                format!("fun(() -> {})", inner_type)
            }
            ast::Type::Record(name, fields) => {
                // Record type with field types
                let field_strs: Vec<String> = fields
                    .iter()
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
            | ast::Type::ConstPointer(inner)
            | ast::Type::MutPointer(inner) => self.map_type(inner),
            ast::Type::CInt
            | ast::Type::CUInt
            | ast::Type::CLong
            | ast::Type::CULong
            | ast::Type::CChar
            | ast::Type::CLongLong
            | ast::Type::CULongLong
            | ast::Type::CFloat
            | ast::Type::CDouble
            | ast::Type::CSize
            | ast::Type::CString => "integer()".to_string(),
            ast::Type::Generic(name) => name.clone(),
            ast::Type::TypeParam(name) => name.clone(),
            ast::Type::TypeConstructor(name, args) => {
                let arg_types: Vec<String> = args.iter().map(|t| self.map_type(t)).collect();
                format!("{}({})", name, arg_types.join(", "))
            }
            ast::Type::Var(name) => name.clone(),
        }
    }

    /// 字段名映射为 Erlang map 键（小写原子，必要时引号）
    fn erlang_field_atom(&self, field: &str) -> String {
        if field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !field.is_empty() {
            field.to_lowercase()
        } else {
            format!("'{}'", field.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }

    /// 将 X 变量名转换为 Erlang 变量名
    /// Erlang 变量必须以大写字母或下划线开头
    fn erlang_variable(&self, name: &str) -> String {
        if name.starts_with('_') {
            name.to_string()
        } else if name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
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

    fn generate_from_hir(&mut self, _hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        // 从 HIR 生成：先转换为 AST 再生成（简化实现）
        // 实际实现应该直接从 HIR 生成
        Err(x_codegen::CodeGenError::Unimplemented(
            "从 HIR 生成 Erlang 代码尚未实现，请使用 AST 后端".to_string(),
        ))
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        self.buffer.clear();
        self.exports.clear();
        self.loop_counter = 0;

        let mut functions: Vec<&x_lir::Function> = Vec::new();
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                self.exports
                    .push(format!("{}/{}", f.name, f.parameters.len()));
                functions.push(f);
            }
        }

        let has_main = functions.iter().any(|f| f.name == "main");
        if !has_main {
            self.exports.push("main/0".to_string());
        }

        self.emit_header()?;

        for f in functions {
            self.emit_lir_function(f)?;
            self.line("")?;
        }

        if !has_main {
            self.emit_default_main()?;
        }

        let output_file = OutputFile {
            path: PathBuf::from(format!("{}.erl", self.module_name)),
            content: self.output().as_bytes().to_vec(),
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
    /// 将 LIR 类型转换为 Erlang 类型（用于 `-spec` 等扩展）
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

    fn emit_lir_function(&mut self, f: &x_lir::Function) -> ErlangResult<()> {
        let ret_spec = self.lir_type_to_erlang(&f.return_type);
        let param_specs: Vec<String> = f
            .parameters
            .iter()
            .map(|p| self.lir_type_to_erlang(&p.type_))
            .collect();
        let spec_params = if param_specs.is_empty() {
            "()".to_string()
        } else {
            format!("({})", param_specs.join(", "))
        };
        self.line(&format!("-spec {}{} -> {}.", f.name, spec_params, ret_spec))?;
        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| self.erlang_variable(&p.name))
            .collect();
        self.line(&format!("{}({}) ->", f.name, params.join(", ")))?;
        self.indent();
        let n = f.body.statements.len();
        if n == 0 {
            self.line("ok.")?;
        } else {
            for (i, stmt) in f.body.statements.iter().enumerate() {
                self.emit_lir_statement_seq(stmt, i + 1 == n)?;
            }
        }
        self.dedent();
        Ok(())
    }

    fn emit_lir_branch_boxed(&mut self, stmt: &x_lir::Statement) -> ErlangResult<()> {
        match stmt {
            x_lir::Statement::Compound(b) => {
                let n = b.statements.len();
                for (j, s) in b.statements.iter().enumerate() {
                    self.emit_lir_statement_seq(s, j + 1 == n)?;
                }
                Ok(())
            }
            s => self.emit_lir_statement_seq(s, true),
        }
    }

    fn emit_lir_loop_body(&mut self, stmt: &x_lir::Statement, label: &str) -> ErlangResult<()> {
        match stmt {
            x_lir::Statement::Compound(b) => {
                for s in &b.statements {
                    self.emit_lir_statement_seq(s, false)?;
                }
                self.line(&format!("{}();", label))?;
                Ok(())
            }
            s => {
                self.emit_lir_statement_seq(s, false)?;
                self.line(&format!("{}();", label))?;
                Ok(())
            }
        }
    }

    fn emit_lir_loop_body_postcond(
        &mut self,
        stmt: &x_lir::Statement,
        label: &str,
        cond: &str,
    ) -> ErlangResult<()> {
        match stmt {
            x_lir::Statement::Compound(b) => {
                for s in &b.statements {
                    self.emit_lir_statement_seq(s, false)?;
                }
            }
            s => {
                self.emit_lir_statement_seq(s, false)?;
            }
        }
        self.line(&format!("case {} of", cond))?;
        self.indent();
        self.line(&format!("true -> {}();", label))?;
        self.line("false -> ok")?;
        self.dedent();
        self.line("end.")?;
        Ok(())
    }

    fn emit_lir_statement_seq(
        &mut self,
        stmt: &x_lir::Statement,
        is_last: bool,
    ) -> ErlangResult<()> {
        use x_lir::Statement::*;
        let end = if is_last { "." } else { "," };
        match stmt {
            Expression(e) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{}{}", s, end))?;
            }
            Return(Some(e)) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{}{}", s, end))?;
            }
            Return(None) => {
                self.line(&format!("ok{}", end))?;
            }
            Variable(v) => {
                let init = v
                    .initializer
                    .as_ref()
                    .map(|e| self.emit_lir_expr(e))
                    .transpose()?
                    .unwrap_or_else(|| "undefined".to_string());
                let name = self.erlang_variable(&v.name);
                self.line(&format!("{} = {}{}", name, init, end))?;
            }
            Compound(b) => {
                let m = b.statements.len();
                for (j, inner) in b.statements.iter().enumerate() {
                    self.emit_lir_statement_seq(inner, is_last && j + 1 == m)?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("case {} of", cond))?;
                self.indent();
                self.line("true ->")?;
                self.indent();
                self.emit_lir_branch_boxed(&i.then_branch)?;
                self.dedent();
                self.line(";")?;
                self.line("false ->")?;
                self.indent();
                match &i.else_branch {
                    Some(el) => self.emit_lir_branch_boxed(el)?,
                    None => self.line("ok")?,
                }
                self.dedent();
                self.dedent();
                self.line(&format!("end{}", end))?;
            }
            While(w) => {
                let id = self.next_loop_id();
                let label = format!("__lir_while_{}", id);
                let c = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("{}() ->", label))?;
                self.indent();
                self.line(&format!("case {} of", c))?;
                self.indent();
                self.line("true ->")?;
                self.indent();
                self.emit_lir_loop_body(&w.body, &label)?;
                self.dedent();
                self.line("false -> ok")?;
                self.dedent();
                self.dedent();
                self.line("end.")?;
                self.dedent();
                self.line(&format!("{}(){}", label, end))?;
            }
            DoWhile(d) => {
                let id = self.next_loop_id();
                let label = format!("__lir_dowhile_{}", id);
                let c = self.emit_lir_expr(&d.condition)?;
                self.line(&format!("{}() ->", label))?;
                self.indent();
                self.emit_lir_loop_body_postcond(&d.body, &label, &c)?;
                self.dedent();
                self.line(&format!("{}(){}", label, end))?;
            }
            Empty => {}
            Break | Continue => {
                self.line(&format!("ok{}", end))?;
            }
            Declaration(_) | For(_) | Switch(_) | Match(_) | Try(_) | Goto(_) | Label(_) => {
                self.line(&format!("% unsupported LIR statement{}", end))?;
            }
        }
        Ok(())
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> ErlangResult<String> {
        use x_lir::Expression::*;
        match expr {
            Literal(l) => self.emit_lir_literal(l),
            Variable(n) => Ok(self.erlang_variable(n)),
            Binary(op, l, r) => {
                let left = self.emit_lir_expr(l)?;
                let right = self.emit_lir_expr(r)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("({} {} {})", left, op_str, right))
            }
            Unary(op, e) => {
                let e = self.emit_lir_expr(e)?;
                use x_lir::UnaryOp::*;
                match op {
                    Plus => Ok(format!("(+{})", e)),
                    Minus => Ok(format!("(-{})", e)),
                    Not => Ok(format!("(not {})", e)),
                    BitNot => Ok(format!("(bnot {})", e)),
                    PreIncrement | PostIncrement => Ok(format!("({} + 1)", e)),
                    PreDecrement | PostDecrement => Ok(format!("({} - 1)", e)),
                    Reference => Ok(format!("{}", e)), // Erlang doesn't have references, just pass through
                    MutableReference => Ok(format!("{}", e)), // Erlang doesn't have references, just pass through
                }
            }
            Ternary(c, t, el) => {
                let c = self.emit_lir_expr(c)?;
                let th = self.emit_lir_expr(t)?;
                let e = self.emit_lir_expr(el)?;
                Ok(format!(
                    "case ({} =/= 0) of true -> {}; false -> {} end",
                    c, th, e
                ))
            }
            Assign(t, v) => {
                let val = self.emit_lir_expr(v)?;
                match t.as_ref() {
                    x_lir::Expression::Variable(n) => {
                        let nv = self.erlang_variable(n);
                        Ok(format!("begin {} = {}, {} end", nv, val, nv))
                    }
                    _ => {
                        let ts = self.emit_lir_expr(t)?;
                        Ok(format!("begin _ = {}, {} end", val, ts))
                    }
                }
            }
            AssignOp(op, t, v) => {
                let tv = self.emit_lir_expr(t)?;
                let vv = self.emit_lir_expr(v)?;
                let sop = self.map_lir_binop(op);
                match t.as_ref() {
                    x_lir::Expression::Variable(n) => {
                        let nv = self.erlang_variable(n);
                        Ok(format!(
                            "begin {} = ({} {} {}), {} end",
                            nv, tv, sop, vv, nv
                        ))
                    }
                    _ => Ok(format!("({} {} {})", tv, sop, vv)),
                }
            }
            Call(callee, args) => self.emit_lir_call(callee, args),
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("lists:nth(({}) + 1, {})", idx_str, arr_str))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                let atom = self.erlang_field_atom(member);
                Ok(format!("maps:get({}, {})", atom, obj_str))
            }
            PointerMember(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                let atom = self.erlang_field_atom(member);
                Ok(format!("maps:get({}, {})", atom, obj_str))
            }
            AddressOf(inner) => {
                let _ = self.emit_lir_expr(inner)?;
                Ok("% addressof".to_string())
            }
            Dereference(e) => self.emit_lir_expr(e),
            Cast(_ty, e) => self.emit_lir_expr(e),
            SizeOf(_ty) => Ok("8".to_string()),
            SizeOfExpr(_e) => Ok("8".to_string()),
            AlignOf(_ty) => Ok("8".to_string()),
            Comma(exprs) => {
                let parts: Vec<String> = exprs
                    .iter()
                    .map(|ex| self.emit_lir_expr(ex))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("begin {} end", parts.join(", ")))
            }
            Parenthesized(e) => self.emit_lir_expr(e),
            InitializerList(inits) | CompoundLiteral(_, inits) => {
                let mut parts = Vec::new();
                for init in inits {
                    self.push_lir_init_expr(init, &mut parts)?;
                }
                Ok(format!("[{}]", parts.join(", ")))
            }
        }
    }

    fn push_lir_init_expr(
        &self,
        init: &x_lir::Initializer,
        out: &mut Vec<String>,
    ) -> ErlangResult<()> {
        match init {
            x_lir::Initializer::Expression(e) => {
                out.push(self.emit_lir_expr(e)?);
                Ok(())
            }
            x_lir::Initializer::List(list) => {
                for i in list {
                    self.push_lir_init_expr(i, out)?;
                }
                Ok(())
            }
            x_lir::Initializer::Named(_, inner) => self.push_lir_init_expr(inner, out),
            x_lir::Initializer::Indexed(_, inner) => self.push_lir_init_expr(inner, out),
        }
    }

    fn emit_lir_call(
        &self,
        callee: &x_lir::Expression,
        args: &[x_lir::Expression],
    ) -> ErlangResult<String> {
        if let x_lir::Expression::Variable(name) = callee {
            match name.as_str() {
                "print" | "println" => {
                    if args.is_empty() {
                        return Ok("io:format(\"~n\", [])".to_string());
                    }
                    let arg = self.emit_lir_expr(&args[0])?;
                    return Ok(format!("io:format(\"~p~n\", [{}])", arg));
                }
                "printf" => {
                    if args.is_empty() {
                        return Ok("io:format(\"\", [])".to_string());
                    }
                    let fmt = self.emit_lir_expr(&args[0])?;
                    let rest: Vec<String> = args[1..]
                        .iter()
                        .map(|a| self.emit_lir_expr(a))
                        .collect::<ErlangResult<Vec<_>>>()?;
                    return Ok(format!("io:format({}, [{}])", fmt, rest.join(", ")));
                }
                _ => {}
            }
        }
        let callee_str = self.emit_lir_expr(callee)?;
        let args_str: Vec<String> = args
            .iter()
            .map(|a| self.emit_lir_expr(a))
            .collect::<ErlangResult<Vec<_>>>()?;
        Ok(format!("{}({})", callee_str, args_str.join(", ")))
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> ErlangResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!(
                "\"{}\"",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            )),
            Char(c) => Ok(format!("\"{}\"", c)),
            Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            NullPointer => Ok("undefined".to_string()),
        }
    }

    /// 映射 LIR 二元运算符
    fn map_lir_binop(&self, op: &x_lir::BinaryOp) -> String {
        use x_lir::BinaryOp::*;
        match op {
            Add => "+",
            Subtract => "-",
            Multiply => "*",
            Divide => "/",
            Modulo => "rem",
            LessThan => "<",
            LessThanEqual => "=<",
            GreaterThan => ">",
            GreaterThanEqual => ">=",
            Equal => "=:=",
            NotEqual => "/=",
            BitAnd => "band",
            BitOr => "bor",
            BitXor => "bxor",
            LeftShift => "bsl",
            RightShift => "bsr",
            RightShiftArithmetic => "bsr",
            LogicalAnd => "andalso",
            LogicalOr => "orelse",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{MethodModifiers, Spanned};

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

        // 测试可选类型 (now via TypeConstructor)
        assert_eq!(
            backend.map_type(&ast::Type::TypeConstructor(
                "Option".to_string(),
                vec![ast::Type::Int]
            )),
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
                                            Box::new(make_expr(ExpressionKind::Variable(
                                                "print".to_string(),
                                            ))),
                                            vec![make_expr(ExpressionKind::Literal(
                                                ast::Literal::String("one".to_string()),
                                            ))],
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
                                            Box::new(make_expr(ExpressionKind::Variable(
                                                "print".to_string(),
                                            ))),
                                            vec![make_expr(ExpressionKind::Literal(
                                                ast::Literal::String("other".to_string()),
                                            ))],
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
        assert!(output.files[0]
            .path
            .to_string_lossy()
            .contains("my_custom_module.erl"));
    }

    #[test]
    fn test_lir_main_return_integer() {
        let mut prog = x_lir::Program::new();
        let mut main = x_lir::Function::new("main", x_lir::Type::Int);
        main.body
            .statements
            .push(x_lir::Statement::Return(Some(x_lir::Expression::int(42))));
        prog.add(x_lir::Declaration::Function(main));

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let out = backend.generate_from_lir(&prog).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);

        assert!(code.contains("-module(x_module)."));
        assert!(code.contains("-export([main/0])."));
        assert!(code.contains("-spec main() -> integer()."));
        assert!(code.contains("main() ->"));
        assert!(code.contains("42."));
    }

    #[test]
    fn test_ast_while_emits_tail_recursive_helper() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "f".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::While(ast::WhileStatement {
                        condition: make_expr(ExpressionKind::Literal(ast::Literal::Boolean(false))),
                        body: ast::Block {
                            statements: vec![make_stmt(StatementKind::Expression(make_expr(
                                ExpressionKind::Literal(ast::Literal::Integer(1)),
                            )))],
                        },
                    }))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let out = backend.generate_from_ast(&program).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);
        assert!(
            code.contains("__x_while_"),
            "while 应生成尾递归辅助函数: {code}"
        );
    }
}
