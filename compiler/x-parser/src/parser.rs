use crate::ast::{
    BinaryOp, Block, CatchClause, Declaration, DoWhileStatement, ExportDecl, Expression,
    ForStatement, FunctionDecl, IfStatement, ImportDecl, ImportSymbol, Literal, MatchCase,
    MatchStatement, ModuleDecl, Parameter, Pattern, Program, Statement, TryStatement, Type,
    TypeAlias, UnaryOp, VariableDecl, WhileStatement,
};
use crate::errors::ParseError;
use x_lexer::token::Token;
use x_lexer::TokenIterator;

pub struct XParser;

impl XParser {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<Program, ParseError> {
        let mut token_iter = TokenIterator::new(input);
        self.parse_program(&mut token_iter)
    }

    fn err(&self, msg: impl Into<String>, ti: &TokenIterator) -> ParseError {
        ParseError::SyntaxError {
            message: msg.into(),
            span: ti.last_span,
        }
    }

    fn expect_token(&self, ti: &mut TokenIterator, expected: &str) -> Result<Token, ParseError> {
        match ti.next() {
            Some(Ok((tok, _))) => Ok(tok),
            Some(Err(e)) => Err(self.err(e.to_string(), ti)),
            None => Err(self.err(format!("期望 {}，但到达文件末尾", expected), ti)),
        }
    }

    fn parse_program(&self, ti: &mut TokenIterator) -> Result<Program, ParseError> {
        let mut declarations = Vec::new();
        let mut statements = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::Function, _)) => {
                    ti.next();
                    declarations.push(Declaration::Function(self.parse_function(ti)?));
                }
                Ok((Token::Let, _)) => {
                    ti.next();
                    let m = self.eat_mut(ti);
                    declarations.push(Declaration::Variable(self.parse_variable(ti, m)?));
                }
                Ok((Token::Val, _)) => {
                    ti.next();
                    declarations.push(Declaration::Variable(self.parse_variable(ti, false)?));
                }
                Ok((Token::Var, _)) => {
                    ti.next();
                    declarations.push(Declaration::Variable(self.parse_variable(ti, true)?));
                }
                Ok((Token::Const, _)) => {
                    ti.next();
                    declarations.push(Declaration::Variable(self.parse_variable(ti, false)?));
                }
                Ok((Token::Type, _)) => {
                    ti.next();
                    declarations.push(Declaration::TypeAlias(self.parse_type_alias(ti)?));
                }
                Ok((Token::Import, _)) => {
                    ti.next();
                    declarations.push(Declaration::Import(self.parse_import(ti)?));
                }
                Ok((Token::Module, _)) => {
                    ti.next();
                    declarations.push(Declaration::Module(self.parse_module(ti)?));
                }
                Ok((Token::Export, _)) => {
                    ti.next();
                    declarations.push(Declaration::Export(self.parse_export(ti)?));
                }
                Ok((Token::RightBrace, _)) => break,
                Ok(_) => {
                    // 尝试解析为顶级语句
                    let stmt = self.parse_statement(ti)?;
                    statements.push(stmt);
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }
        Ok(Program {
            declarations,
            statements,
        })
    }

    fn parse_module(&self, ti: &mut TokenIterator) -> Result<ModuleDecl, ParseError> {
        let name = match self.expect_token(ti, "模块名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望模块名，但得到 {:?}", t), ti)),
        };
        self.eat_semi(ti);
        Ok(ModuleDecl { name })
    }

    fn parse_export(&self, ti: &mut TokenIterator) -> Result<ExportDecl, ParseError> {
        let symbol = match self.expect_token(ti, "导出符号")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望导出符号，但得到 {:?}", t), ti)),
        };
        self.eat_semi(ti);
        Ok(ExportDecl { symbol })
    }

    fn parse_import(&self, ti: &mut TokenIterator) -> Result<ImportDecl, ParseError> {
        let mut module_path = String::new();

        // 解析模块路径
        let token = self.expect_token(ti, "标识符或字符串")?;
        match token {
            Token::Ident(name) => {
                module_path.push_str(&name);

                // 处理点号或双冒号分隔的路径
                loop {
                    match ti.peek() {
                        Some(Ok((Token::Dot, _))) => {
                            ti.next();
                            module_path.push('.');

                            let next_token = self.expect_token(ti, "标识符")?;
                            match next_token {
                                Token::Ident(name) => {
                                    module_path.push_str(&name);
                                }
                                _ => {
                                    return Err(self
                                        .err(format!("期望标识符，但得到 {:?}", next_token), ti));
                                }
                            }
                        }
                        Some(Ok((Token::DoubleColon, _))) => {
                            ti.next();
                            module_path.push_str("::");

                            let next_token = self.expect_token(ti, "标识符")?;
                            match next_token {
                                Token::Ident(name) => {
                                    module_path.push_str(&name);
                                }
                                _ => {
                                    return Err(self
                                        .err(format!("期望标识符，但得到 {:?}", next_token), ti));
                                }
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
            Token::StringContent(path) => {
                // 处理字符串形式的路径
                module_path = path;
            }
            _ => {
                return Err(self.err(format!("期望标识符或字符串，但得到 {:?}", token), ti));
            }
        }

        let mut symbols = Vec::new();

        // 解析导入符号
        match ti.peek() {
            Some(Ok((Token::Dot, _))) => {
                ti.next();

                match ti.peek() {
                    Some(Ok((Token::Asterisk, _))) => {
                        // 通配导入: import module.*
                        ti.next();
                        symbols.push(ImportSymbol::All);
                    }
                    Some(Ok((Token::LeftBrace, _))) => {
                        // 选择导入: import module.{a, b, c}
                        ti.next();

                        loop {
                            let token = self.expect_token(ti, "标识符")?;
                            match token {
                                Token::Ident(name) => {
                                    let mut alias = None;

                                    match ti.peek() {
                                        Some(Ok((Token::Ident(ref s), _))) if s == "as" => {
                                            ti.next();
                                            let alias_token = self.expect_token(ti, "标识符")?;
                                            match alias_token {
                                                Token::Ident(alias_name) => {
                                                    alias = Some(alias_name);
                                                }
                                                _ => {
                                                    return Err(self.err(
                                                        format!(
                                                            "期望标识符，但得到 {:?}",
                                                            alias_token
                                                        ),
                                                        ti,
                                                    ));
                                                }
                                            }
                                        }
                                        _ => {}
                                    }

                                    symbols.push(ImportSymbol::Named(name, alias));

                                    match ti.peek() {
                                        Some(Ok((Token::Comma, _))) => {
                                            ti.next();
                                        }
                                        Some(Ok((Token::RightBrace, _))) => {
                                            ti.next();
                                            break;
                                        }
                                        _ => {
                                            return Err(self.err("期望 , 或 }", ti));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(
                                        self.err(format!("期望标识符，但得到 {:?}", token), ti)
                                    );
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(self.err("期望 * 或 {", ti));
                    }
                }
            }
            _ => {
                // 单一导入: import module
                symbols.push(ImportSymbol::All);
            }
        }

        // 吃掉分号
        self.eat_semi(ti);

        Ok(ImportDecl {
            module_path,
            symbols,
        })
    }

    fn eat_mut(&self, ti: &mut TokenIterator) -> bool {
        if matches!(ti.peek(), Some(Ok((Token::Mut, _)))) {
            ti.next();
            true
        } else {
            false
        }
    }

    fn parse_function(&self, ti: &mut TokenIterator) -> Result<FunctionDecl, ParseError> {
        let name = match self.expect_token(ti, "函数名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望函数名，但得到 {:?}", t), ti)),
        };

        match self.expect_token(ti, "(")? {
            Token::LeftParen => {}
            t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
        }

        let parameters = self.parse_param_list(ti)?;

        let return_type = if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
            ti.next();
            Some(self.parse_type(ti)?)
        } else {
            None
        };

        let body = if matches!(ti.peek(), Some(Ok((Token::Equals, _)))) {
            ti.next();
            let expr = self.parse_expression(ti)?;
            self.eat_semi(ti);
            Block {
                statements: vec![Statement::Expression(expr)],
            }
        } else {
            match self.expect_token(ti, "{")? {
                Token::LeftBrace => {}
                t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
            }
            self.parse_block(ti)?
        };

        Ok(FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            is_async: false,
        })
    }

    fn parse_param_list(&self, ti: &mut TokenIterator) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
            ti.next();
            return Ok(params);
        }
        loop {
            let name = match self.expect_token(ti, "参数名")? {
                Token::Ident(n) => n,
                Token::RightParen => break,
                t => return Err(self.err(format!("期望参数名，但得到 {:?}", t), ti)),
            };
            let type_annot = if matches!(ti.peek(), Some(Ok((Token::Colon, _)))) {
                ti.next();
                Some(self.parse_type(ti)?)
            } else {
                None
            };
            params.push(Parameter {
                name,
                type_annot,
                default: None,
            });
            match ti.peek() {
                Some(Ok((Token::Comma, _))) => {
                    ti.next();
                }
                Some(Ok((Token::RightParen, _))) => {
                    ti.next();
                    break;
                }
                _ => return Err(self.err("期望 , 或 )", ti)),
            }
        }
        Ok(params)
    }

    fn parse_variable(
        &self,
        ti: &mut TokenIterator,
        is_mutable: bool,
    ) -> Result<VariableDecl, ParseError> {
        let name = match self.expect_token(ti, "变量名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望变量名，但得到 {:?}", t), ti)),
        };
        let type_annot = if matches!(ti.peek(), Some(Ok((Token::Colon, _)))) {
            ti.next();
            Some(self.parse_type(ti)?)
        } else {
            None
        };
        let initializer = if matches!(ti.peek(), Some(Ok((Token::Equals, _)))) {
            ti.next();
            Some(self.parse_expression(ti)?)
        } else {
            None
        };
        Ok(VariableDecl {
            name,
            is_mutable,
            type_annot,
            initializer,
        })
    }

    fn parse_block(&self, ti: &mut TokenIterator) -> Result<Block, ParseError> {
        let mut statements = Vec::new();
        let mut closed = false;
        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    closed = true;
                    break;
                }
                _ => {
                    let stmt = self.parse_statement(ti)?;
                    statements.push(stmt);
                }
            }
        }
        if !closed {
            return Err(self.err("语法错误: 未闭合的大括号 `{`", ti));
        }
        Ok(Block { statements })
    }

    fn parse_statement(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        while matches!(ti.peek(), Some(Ok((Token::Semicolon, _)))) {
            ti.next();
        }

        // 允许尾随分号：`;` 之后直接 EOF（或 block 结束）不应报 “期望表达式”
        if ti.peek().is_none() || matches!(ti.peek(), Some(Ok((Token::RightBrace, _)))) {
            return Ok(Statement::Expression(Expression::Literal(Literal::Unit)));
        }

        match ti.peek() {
            Some(Ok((Token::Return, _))) => {
                ti.next();
                let expr = if matches!(
                    ti.peek(),
                    Some(Ok((Token::Semicolon, _))) | Some(Ok((Token::RightBrace, _))) | None
                ) {
                    None
                } else {
                    Some(self.parse_expression(ti)?)
                };
                self.eat_semi(ti);
                Ok(Statement::Return(expr))
            }
            Some(Ok((Token::If, _))) => {
                ti.next();
                self.parse_if(ti)
            }
            Some(Ok((Token::While, _))) => {
                ti.next();
                self.parse_while(ti)
            }
            Some(Ok((Token::For, _))) => {
                ti.next();
                self.parse_for(ti)
            }
            Some(Ok((Token::Let, _))) => {
                ti.next();
                let m = self.eat_mut(ti);
                let var = self.parse_variable(ti, m)?;
                self.eat_semi(ti);
                Ok(Statement::Variable(var))
            }
            Some(Ok((Token::Val, _))) => {
                ti.next();
                let var = self.parse_variable(ti, false)?;
                self.eat_semi(ti);
                Ok(Statement::Variable(var))
            }
            Some(Ok((Token::Var, _))) => {
                ti.next();
                let var = self.parse_variable(ti, true)?;
                self.eat_semi(ti);
                Ok(Statement::Variable(var))
            }
            Some(Ok((Token::Const, _))) => {
                ti.next();
                let var = self.parse_variable(ti, false)?;
                self.eat_semi(ti);
                Ok(Statement::Variable(var))
            }
            Some(Ok((Token::Try, _))) => {
                ti.next();
                self.parse_try(ti)
            }
            Some(Ok((Token::Ident(ref s), _))) if s == "match" => {
                ti.next();
                self.parse_match(ti)
            }
            Some(Ok((Token::Ident(ref s), _))) if s == "break" => {
                ti.next();
                self.eat_semi(ti);
                Ok(Statement::Break)
            }
            Some(Ok((Token::Ident(ref s), _))) if s == "continue" => {
                ti.next();
                self.eat_semi(ti);
                Ok(Statement::Continue)
            }
            Some(Ok((Token::Ident(ref s), _))) if s == "do" => {
                ti.next();
                self.parse_do_while(ti)
            }
            _ => {
                let expr = self.parse_expression(ti)?;
                self.eat_semi(ti);
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_try(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }
        let body = self.parse_block(ti)?;

        let mut catch_clauses = Vec::new();
        while matches!(ti.peek(), Some(Ok((Token::Catch, _)))) {
            ti.next();
            let mut exception_type = None;
            let mut variable_name = None;

            if matches!(ti.peek(), Some(Ok((Token::LeftParen, _)))) {
                ti.next();
                let t1 = self.expect_token(ti, "异常类型或变量名")?;
                match t1 {
                    Token::Ident(n) => {
                        exception_type = Some(n);
                    }
                    t => return Err(self.err(format!("期望标识符，但得到 {:?}", t), ti)),
                }

                if matches!(ti.peek(), Some(Ok((Token::Ident(_), _)))) {
                    let t2 = self.expect_token(ti, "变量名")?;
                    match t2 {
                        Token::Ident(n) => variable_name = Some(n),
                        t => return Err(self.err(format!("期望标识符，但得到 {:?}", t), ti)),
                    }
                }

                match self.expect_token(ti, ")")? {
                    Token::RightParen => {}
                    t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                }
            }

            match self.expect_token(ti, "{")? {
                Token::LeftBrace => {}
                t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
            }
            let cb = self.parse_block(ti)?;
            catch_clauses.push(CatchClause {
                exception_type,
                variable_name,
                body: cb,
            });
        }

        let finally_block = if matches!(ti.peek(), Some(Ok((Token::Finally, _)))) {
            ti.next();
            match self.expect_token(ti, "{")? {
                Token::LeftBrace => {}
                t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
            }
            Some(self.parse_block(ti)?)
        } else {
            None
        };

        Ok(Statement::Try(TryStatement {
            body,
            catch_clauses,
            finally_block,
        }))
    }

    fn parse_match(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        let expression = self.parse_expression(ti)?;
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let mut cases = Vec::new();
        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    break;
                }
                Ok(_) => {
                    let pattern = self.parse_pattern(ti)?;
                    let guard = if matches!(ti.peek(), Some(Ok((Token::When, _)))) {
                        ti.next();
                        Some(self.parse_expression(ti)?)
                    } else {
                        None
                    };
                    match self.expect_token(ti, "{")? {
                        Token::LeftBrace => {}
                        t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                    }
                    let body = self.parse_block(ti)?;
                    cases.push(MatchCase {
                        pattern,
                        body,
                        guard,
                    });
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(Statement::Match(MatchStatement { expression, cases }))
    }

    fn parse_pattern(&self, ti: &mut TokenIterator) -> Result<Pattern, ParseError> {
        // 仅实现最小子集：通配符/变量/字面量/或模式
        let mut pat = match self.expect_token(ti, "pattern")? {
            Token::Ident(name) if name == "_" => Pattern::Wildcard,
            Token::Ident(name) => Pattern::Variable(name),
            Token::True => Pattern::Literal(Literal::Boolean(true)),
            Token::False => Pattern::Literal(Literal::Boolean(false)),
            Token::Null => Pattern::Literal(Literal::Null),
            Token::DecimalInt(s) => Pattern::Literal(Literal::Integer(s.parse().unwrap_or(0))),
            Token::Float(s) => Pattern::Literal(Literal::Float(s.parse().unwrap_or(0.0))),
            Token::StringContent(s) => Pattern::Literal(Literal::String(s)),
            Token::CharContent(s) => {
                let c = s.chars().next().unwrap_or('\0');
                Pattern::Literal(Literal::Char(c))
            }
            t => return Err(self.err(format!("不支持的 pattern 起始标记: {:?}", t), ti)),
        };

        while matches!(ti.peek(), Some(Ok((Token::VerticalBar, _)))) {
            ti.next();
            let rhs = self.parse_pattern(ti)?;
            pat = Pattern::Or(Box::new(pat), Box::new(rhs));
        }
        Ok(pat)
    }

    fn eat_semi(&self, ti: &mut TokenIterator) {
        if matches!(ti.peek(), Some(Ok((Token::Semicolon, _)))) {
            ti.next();
        }
    }

    fn parse_if(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        let condition = self.parse_expression(ti)?;
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }
        let then_block = self.parse_block(ti)?;
        let else_block = if matches!(ti.peek(), Some(Ok((Token::Else, _)))) {
            ti.next();
            if matches!(ti.peek(), Some(Ok((Token::If, _)))) {
                ti.next();
                let nested = self.parse_if(ti)?;
                Some(Block {
                    statements: vec![nested],
                })
            } else {
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                Some(self.parse_block(ti)?)
            }
        } else {
            None
        };
        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            else_block,
        }))
    }

    fn parse_while(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        let condition = self.parse_expression(ti)?;
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }
        let body = self.parse_block(ti)?;
        Ok(Statement::While(WhileStatement { condition, body }))
    }

    fn parse_do_while(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }
        let body = self.parse_block(ti)?;
        match self.expect_token(ti, "while")? {
            Token::While => {}
            Token::Ident(ref s) if s == "while" => {}
            t => return Err(self.err(format!("期望 while，但得到 {:?}", t), ti)),
        }
        match self.expect_token(ti, "(")? {
            Token::LeftParen => {}
            t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
        }
        let condition = self.parse_expression(ti)?;
        match self.expect_token(ti, ")")? {
            Token::RightParen => {}
            t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
        }
        self.eat_semi(ti);
        Ok(Statement::DoWhile(DoWhileStatement { body, condition }))
    }

    fn parse_type_alias(&self, ti: &mut TokenIterator) -> Result<TypeAlias, ParseError> {
        let name = match self.expect_token(ti, "类型名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望类型名，但得到 {:?}", t), ti)),
        };

        // 处理泛型参数
        if matches!(ti.peek(), Some(Ok((Token::LessThan, _)))) {
            ti.next();
            // 暂时跳过泛型参数
            while !matches!(ti.peek(), Some(Ok((Token::GreaterThan, _)))) {
                ti.next();
            }
            ti.next();
        }

        match self.expect_token(ti, "=")? {
            Token::Equals => {}
            t => return Err(self.err(format!("期望 =，但得到 {:?}", t), ti)),
        }

        // 处理类型定义
        // 暂时简单处理，直接跳过直到分号或文件结束
        while !matches!(ti.peek(), Some(Ok((Token::Semicolon, _)))) && ti.peek().is_some() {
            ti.next();
        }
        if matches!(ti.peek(), Some(Ok((Token::Semicolon, _)))) {
            ti.next();
        }

        Ok(TypeAlias {
            name: name.clone(),
            type_: Type::Generic(name),
        })
    }

    fn parse_for(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        let name = match self.expect_token(ti, "变量名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望变量名，但得到 {:?}", t), ti)),
        };
        match self.expect_token(ti, "in")? {
            Token::Ident(ref s) if s == "in" => {}
            Token::In => {}
            t => return Err(self.err(format!("期望 in，但得到 {:?}", t), ti)),
        }
        let iterator = self.parse_expression(ti)?;
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }
        let body = self.parse_block(ti)?;
        let pattern = Pattern::Variable(name);
        Ok(Statement::For(ForStatement {
            pattern,
            iterator,
            body,
        }))
    }

    // ── Expression parsing (precedence climbing) ──

    fn parse_expression(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        self.parse_assignment(ti)
    }

    fn parse_pipe(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut expr = self.parse_or(ti)?;
        while matches!(ti.peek(), Some(Ok((Token::Pipe, _)))) {
            ti.next();
            let right = self.parse_or(ti)?;
            expr = Expression::Pipe(Box::new(expr), vec![Box::new(right)]);
        }
        Ok(expr)
    }

    fn parse_assignment(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let expr = self.parse_pipe(ti)?;
        if matches!(ti.peek(), Some(Ok((Token::Equals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            return Ok(Expression::Assign(Box::new(expr), Box::new(rhs)));
        }
        if matches!(ti.peek(), Some(Ok((Token::PlusEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(BinaryOp::Add, Box::new(expr.clone()), Box::new(rhs));
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::MinusEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(BinaryOp::Sub, Box::new(expr.clone()), Box::new(rhs));
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::AsteriskEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(BinaryOp::Mul, Box::new(expr.clone()), Box::new(rhs));
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::SlashEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(BinaryOp::Div, Box::new(expr.clone()), Box::new(rhs));
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::PercentEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(BinaryOp::Mod, Box::new(expr.clone()), Box::new(rhs));
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        Ok(expr)
    }

    fn parse_or(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_and(ti)?;
        while matches!(ti.peek(), Some(Ok((Token::OrOr, _)))) {
            ti.next();
            let right = self.parse_and(ti)?;
            left = Expression::Binary(BinaryOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison(ti)?;
        while matches!(ti.peek(), Some(Ok((Token::AndAnd, _)))) {
            ti.next();
            let right = self.parse_comparison(ti)?;
            left = Expression::Binary(BinaryOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive(ti)?;
        loop {
            let op = match ti.peek() {
                Some(Ok((Token::LessThanEquals, _))) => BinaryOp::LessEqual,
                Some(Ok((Token::GreaterThanEquals, _))) => BinaryOp::GreaterEqual,
                Some(Ok((Token::LessThan, _))) => BinaryOp::Less,
                Some(Ok((Token::GreaterThan, _))) => BinaryOp::Greater,
                Some(Ok((Token::DoubleEquals, _))) => BinaryOp::Equal,
                Some(Ok((Token::NotEquals, _))) => BinaryOp::NotEqual,
                _ => break,
            };
            ti.next();
            let right = self.parse_additive(ti)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_additive(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative(ti)?;
        loop {
            match ti.peek() {
                Some(Ok((Token::Plus, _))) => {
                    ti.next();
                    let right = self.parse_multiplicative(ti)?;
                    left = Expression::Binary(BinaryOp::Add, Box::new(left), Box::new(right));
                }
                Some(Ok((Token::Minus, _))) => {
                    ti.next();
                    let right = self.parse_multiplicative(ti)?;
                    left = Expression::Binary(BinaryOp::Sub, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary(ti)?;
        loop {
            let op = match ti.peek() {
                Some(Ok((Token::Asterisk, _))) => BinaryOp::Mul,
                Some(Ok((Token::Slash, _))) => BinaryOp::Div,
                Some(Ok((Token::Percent, _))) => BinaryOp::Mod,
                _ => break,
            };
            ti.next();
            let right = self.parse_unary(ti)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        match ti.peek() {
            Some(Ok((Token::Minus, _))) => {
                ti.next();
                let operand = self.parse_unary(ti)?;
                Ok(Expression::Unary(UnaryOp::Negate, Box::new(operand)))
            }
            Some(Ok((Token::NotOperator, _))) | Some(Ok((Token::Not, _))) => {
                ti.next();
                let operand = self.parse_unary(ti)?;
                Ok(Expression::Unary(UnaryOp::Not, Box::new(operand)))
            }
            _ => self.parse_postfix(ti),
        }
    }

    fn parse_postfix(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary(ti)?;
        loop {
            match ti.peek() {
                Some(Ok((Token::LeftParen, _))) => {
                    ti.next();
                    let args = self.parse_call_arguments(ti)?;
                    expr = Expression::Call(Box::new(expr), args);
                }
                Some(Ok((Token::LeftBracket, _))) => {
                    ti.next();
                    let index = self.parse_expression(ti)?;
                    match self.expect_token(ti, "]")? {
                        Token::RightBracket => {}
                        t => return Err(self.err(format!("期望 ]，但得到 {:?}", t), ti)),
                    }
                    expr = Expression::Member(Box::new(expr), format!("__index__{}", 0));
                    expr = Expression::Call(
                        Box::new(Expression::Variable("__index__".to_string())),
                        vec![
                            {
                                if let Expression::Member(inner, _) = expr {
                                    *inner
                                } else {
                                    unreachable!()
                                }
                            },
                            index,
                        ],
                    );
                }
                Some(Ok((Token::Dot, _))) => {
                    ti.next();
                    let member = match self.expect_token(ti, "成员名")? {
                        Token::Ident(n) => n,
                        t => return Err(self.err(format!("期望成员名，但得到 {:?}", t), ti)),
                    };
                    expr = Expression::Member(Box::new(expr), member);
                }
                Some(Ok((Token::RangeExclusive, _))) => {
                    ti.next();
                    let right = self.parse_primary(ti)?;
                    expr = Expression::Range(Box::new(expr), Box::new(right), false);
                }
                Some(Ok((Token::RangeInclusive, _))) => {
                    ti.next();
                    let right = self.parse_primary(ti)?;
                    expr = Expression::Range(Box::new(expr), Box::new(right), true);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let tok = self.expect_token(ti, "表达式")?;
        match tok {
            Token::DecimalInt(s) => {
                let n: i64 = s.parse().map_err(|_| self.err("无效整数", ti))?;
                Ok(Expression::Literal(Literal::Integer(n)))
            }
            Token::Float(s) => {
                let f: f64 = s.parse().map_err(|_| self.err("无效浮点数", ti))?;
                Ok(Expression::Literal(Literal::Float(f)))
            }
            Token::True => Ok(Expression::Literal(Literal::Boolean(true))),
            Token::False => Ok(Expression::Literal(Literal::Boolean(false))),
            Token::Null => Ok(Expression::Literal(Literal::Null)),

            Token::StringQuote => self.parse_string(ti),
            Token::StringContent(c) => Ok(Expression::Literal(Literal::String(c))),
            Token::Ident(name) => {
                // 检查是否是函数调用
                if matches!(ti.peek(), Some(Ok((Token::LeftParen, _)))) {
                    // 特殊处理Some和None模式
                    if name == "Some" || name == "None" {
                        // 这里应该解析为模式，但暂时返回变量
                        Ok(Expression::Variable(name))
                    } else {
                        ti.next();
                        let args = self.parse_call_arguments(ti)?;
                        Ok(Expression::Call(Box::new(Expression::Variable(name)), args))
                    }
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Token::When => self.parse_when(ti),
            Token::LeftParen => {
                if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
                    // 处理空括号 () 作为 Unit 类型
                    ti.next();
                    Ok(Expression::Literal(Literal::Unit))
                } else {
                    // 处理带表达式的括号
                    let inner = self.parse_expression(ti)?;
                    match self.expect_token(ti, ")")? {
                        Token::RightParen => Ok(Expression::Parenthesized(Box::new(inner))),
                        t => Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                    }
                }
            }
            Token::LeftBracket => {
                let mut elems = Vec::new();
                if !matches!(ti.peek(), Some(Ok((Token::RightBracket, _)))) {
                    loop {
                        elems.push(self.parse_expression(ti)?);
                        match ti.peek() {
                            Some(Ok((Token::Comma, _))) => {
                                ti.next();
                            }
                            _ => break,
                        }
                    }
                }
                match self.expect_token(ti, "]")? {
                    Token::RightBracket => {}
                    t => return Err(self.err(format!("期望 ]，但得到 {:?}", t), ti)),
                }
                Ok(Expression::Array(elems))
            }
            Token::LeftBrace => {
                // 处理对象字面量 (map)
                let mut pairs = Vec::new();
                if !matches!(ti.peek(), Some(Ok((Token::RightBrace, _)))) {
                    loop {
                        // 解析键
                        let key = self.expect_token(ti, "键")?;
                        let key_expr = match key {
                            Token::Ident(name) => Expression::Literal(Literal::String(name)),
                            Token::StringContent(s) => Expression::Literal(Literal::String(s)),
                            _ => return Err(self.err(format!("期望键，但得到 {:?}", key), ti)),
                        };

                        // 解析 =>
                        match self.expect_token(ti, "=>")? {
                            Token::FatArrow => {}
                            t => return Err(self.err(format!("期望 =>，但得到 {:?}", t), ti)),
                        }

                        // 解析值
                        let value = self.parse_expression(ti)?;
                        pairs.push((key_expr, value));

                        match ti.peek() {
                            Some(Ok((Token::Comma, _))) => {
                                ti.next();
                            }
                            _ => break,
                        }
                    }
                }
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}, 但得到 {:?}", t), ti)),
                }
                Ok(Expression::Dictionary(pairs))
            }
            t => Err(self.err(format!("期望表达式，但得到 {:?}", t), ti)),
        }
    }

    fn parse_when(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let condition = self.parse_expression(ti)?;
        match self.expect_token(ti, "then")? {
            Token::Ident(ref s) if s == "then" => {}
            t => return Err(self.err(format!("期望 then，但得到 {:?}", t), ti)),
        }
        let then_expr = self.parse_expression(ti)?;
        match self.expect_token(ti, "else")? {
            Token::Ident(ref s) if s == "else" => {}
            Token::Else => {}
            t => return Err(self.err(format!("期望 else，但得到 {:?}", t), ti)),
        }
        let else_expr = self.parse_expression(ti)?;
        Ok(Expression::If(
            Box::new(condition),
            Box::new(then_expr),
            Box::new(else_expr),
        ))
    }

    fn parse_call_arguments(&self, ti: &mut TokenIterator) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
            ti.next();
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression(ti)?);
            match ti.peek() {
                Some(Ok((Token::Comma, _))) => {
                    ti.next();
                }
                _ => break,
            }
        }
        match self.expect_token(ti, ")")? {
            Token::RightParen => {}
            t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
        }
        Ok(args)
    }

    fn parse_type(&self, ti: &mut TokenIterator) -> Result<Type, ParseError> {
        let tok = self.expect_token(ti, "类型名")?;
        let base_type = match tok {
            Token::Ident(name) => match name.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Bool" => Type::Bool,
                "String" => Type::String,
                "Char" => Type::Char,
                "Unit" => Type::Unit,
                "Never" => Type::Never,
                _ => Type::Generic(name),
            },
            t => return Err(self.err(format!("期望类型名，但得到 {:?}", t), ti)),
        };

        // 处理泛型类型参数
        if matches!(ti.peek(), Some(Ok((Token::LessThan, _)))) {
            ti.next();
            let mut type_args = Vec::new();
            loop {
                type_args.push(self.parse_type(ti)?);
                if matches!(ti.peek(), Some(Ok((Token::GreaterThan, _)))) {
                    ti.next();
                    break;
                }
                if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                    ti.next();
                } else {
                    return Err(self.err("期望 , 或 >", ti));
                }
            }
            // 为了兼容现有的Type枚举，我们将泛型类型参数信息存储在Generic类型中
            // 实际使用时，标准库会提供这些类型的定义
            Ok(Type::Generic(format!(
                "{}<{}",
                base_type,
                type_args
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )))
        } else {
            Ok(base_type)
        }
    }

    fn parse_string(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut content = String::new();
        while let Some(token_result) = ti.next() {
            match token_result {
                Ok((Token::StringQuote, _)) => break,
                Ok((Token::StringContent(s), _)) => content.push_str(&s),
                Ok((t, _)) => return Err(self.err(format!("期望字符串内容，但得到 {:?}", t), ti)),
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }
        Ok(Expression::Literal(Literal::String(content)))
    }
}
