use crate::ast::{
    BinaryOp, Block, Declaration, Expression, FunctionDecl, IfStatement, Literal, Parameter,
    Program, Statement, Type, UnaryOp, VariableDecl, WhileStatement,
};
use crate::errors::ParseError;
use x_lexer::token::Token;
use x_lexer::TokenIterator;

pub struct XParser;

impl XParser {
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

    fn expect_token(
        &self,
        ti: &mut TokenIterator,
        expected: &str,
    ) -> Result<Token, ParseError> {
        match ti.next() {
            Some(Ok((tok, _))) => Ok(tok),
            Some(Err(e)) => Err(self.err(e.to_string(), ti)),
            None => Err(self.err(format!("期望 {}，但到达文件末尾", expected), ti)),
        }
    }

    fn parse_program(&self, ti: &mut TokenIterator) -> Result<Program, ParseError> {
        let mut declarations = Vec::new();
        while let Some(token_result) = ti.next() {
            match token_result {
                Ok((Token::Fun, _)) => {
                    declarations.push(Declaration::Function(self.parse_function(ti)?));
                }
                Ok((Token::Let, _)) => {
                    let m = self.eat_mut(ti);
                    declarations.push(Declaration::Variable(self.parse_variable(ti, m)?));
                }
                Ok((Token::Val, _)) => {
                    declarations.push(Declaration::Variable(self.parse_variable(ti, false)?));
                }
                Ok((Token::Var, _)) => {
                    declarations.push(Declaration::Variable(self.parse_variable(ti, true)?));
                }
                Ok((_, _)) => continue,
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }
        Ok(Program { declarations })
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

        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let body = self.parse_block(ti)?;

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
                Some(Ok((Token::Comma, _))) => { ti.next(); }
                Some(Ok((Token::RightParen, _))) => { ti.next(); break; }
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

        match ti.peek() {
            Some(Ok((Token::Return, _))) => {
                ti.next();
                let expr = if matches!(
                    ti.peek(),
                    Some(Ok((Token::Semicolon, _)))
                        | Some(Ok((Token::RightBrace, _)))
                        | None
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
            _ => {
                let expr = self.parse_expression(ti)?;
                self.eat_semi(ti);
                Ok(Statement::Expression(expr))
            }
        }
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

    // ── Expression parsing (precedence climbing) ──

    fn parse_expression(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        self.parse_assignment(ti)
    }

    fn parse_assignment(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let expr = self.parse_or(ti)?;
        if matches!(ti.peek(), Some(Ok((Token::Equals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            return Ok(Expression::Assign(Box::new(expr), Box::new(rhs)));
        }
        if matches!(ti.peek(), Some(Ok((Token::PlusEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(
                BinaryOp::Add,
                Box::new(expr.clone()),
                Box::new(rhs),
            );
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::MinusEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(
                BinaryOp::Sub,
                Box::new(expr.clone()),
                Box::new(rhs),
            );
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::AsteriskEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(
                BinaryOp::Mul,
                Box::new(expr.clone()),
                Box::new(rhs),
            );
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::SlashEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(
                BinaryOp::Div,
                Box::new(expr.clone()),
                Box::new(rhs),
            );
            return Ok(Expression::Assign(Box::new(expr), Box::new(expanded)));
        }
        if matches!(ti.peek(), Some(Ok((Token::PercentEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = Expression::Binary(
                BinaryOp::Mod,
                Box::new(expr.clone()),
                Box::new(rhs),
            );
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
            let op = match ti.peek() {
                Some(Ok((Token::Plus, _))) => BinaryOp::Add,
                Some(Ok((Token::Minus, _))) => BinaryOp::Sub,
                _ => break,
            };
            ti.next();
            let right = self.parse_multiplicative(ti)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
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
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let tok = self
            .expect_token(ti, "表达式")?;
        match tok {
            Token::DecimalInt(s) => {
                let n: i64 = s
                    .parse()
                    .map_err(|_| self.err("无效整数", ti))?;
                Ok(Expression::Literal(Literal::Integer(n)))
            }
            Token::Float(s) => {
                let f: f64 = s
                    .parse()
                    .map_err(|_| self.err("无效浮点数", ti))?;
                Ok(Expression::Literal(Literal::Float(f)))
            }
            Token::True => Ok(Expression::Literal(Literal::Boolean(true))),
            Token::False => Ok(Expression::Literal(Literal::Boolean(false))),
            Token::Null => Ok(Expression::Literal(Literal::Null)),
            Token::NoneKeyword => Ok(Expression::Literal(Literal::None)),
            Token::StringQuote => self.parse_string(ti),
            Token::StringContent(c) => Ok(Expression::Literal(Literal::String(c))),
            Token::Ident(name) => Ok(Expression::Variable(name)),
            Token::LeftParen => {
                let inner = self.parse_expression(ti)?;
                match self.expect_token(ti, ")")? {
                    Token::RightParen => Ok(Expression::Parenthesized(Box::new(inner))),
                    t => Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                }
            }
            Token::LeftBracket => {
                let mut elems = Vec::new();
                if !matches!(ti.peek(), Some(Ok((Token::RightBracket, _)))) {
                    loop {
                        elems.push(self.parse_expression(ti)?);
                        match ti.peek() {
                            Some(Ok((Token::Comma, _))) => { ti.next(); }
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
            t => Err(self.err(format!("期望表达式，但得到 {:?}", t), ti)),
        }
    }

    fn parse_call_arguments(
        &self,
        ti: &mut TokenIterator,
    ) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
            ti.next();
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression(ti)?);
            match ti.peek() {
                Some(Ok((Token::Comma, _))) => { ti.next(); }
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
        match self.expect_token(ti, "类型名")? {
            Token::Ident(name) => match name.as_str() {
                "Int" => Ok(Type::Int),
                "Float" => Ok(Type::Float),
                "Bool" => Ok(Type::Bool),
                "String" => Ok(Type::String),
                "Char" => Ok(Type::Char),
                "Unit" => Ok(Type::Unit),
                "Never" => Ok(Type::Never),
                _ => Ok(Type::Generic(name)),
            },
            t => Err(self.err(format!("期望类型名，但得到 {:?}", t), ti)),
        }
    }

    fn parse_string(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut content = String::new();
        while let Some(token_result) = ti.next() {
            match token_result {
                Ok((Token::StringQuote, _)) => break,
                Ok((Token::StringContent(s), _)) => content.push_str(&s),
                Ok((t, _)) => {
                    return Err(self.err(
                        format!("期望字符串内容，但得到 {:?}", t),
                        ti,
                    ))
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }
        Ok(Expression::Literal(Literal::String(content)))
    }
}
