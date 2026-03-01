use crate::ast::{Program, Declaration, VariableDecl, FunctionDecl, Parameter, Block, Statement, Expression, Literal, Type, IfStatement, BinaryOp};
use crate::errors::ParseError;
use x_lexer::token::Token;
use x_lexer::TokenIterator;

/// X语言语法分析器
pub struct XParser;

impl XParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<Program, ParseError> {
        // 创建词法分析器
        let mut token_iter = TokenIterator::new(input);

        // 简单的解析逻辑，用于解析Hello World示例
        let program = self.parse_program(&mut token_iter)?;

        Ok(program)
    }

    fn parse_program(&self, token_iter: &mut TokenIterator) -> Result<Program, ParseError> {
        let mut declarations = Vec::new();

        // 简单地解析一个main函数
        while let Some(token_result) = token_iter.next() {
            match token_result {
                Ok((Token::Fun, _)) => {
                    let func = self.parse_function(token_iter)?;
                    declarations.push(Declaration::Function(func));
                }
                Ok((Token::Val, _)) => {
                    let var = self.parse_variable(token_iter, false)?;
                    declarations.push(Declaration::Variable(var));
                }
                Ok((Token::Var, _)) => {
                    let var = self.parse_variable(token_iter, true)?;
                    declarations.push(Declaration::Variable(var));
                }
                Ok((_, _)) => continue,
                Err(e) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            }
        }

        Ok(Program { declarations })
    }

    fn parse_function(&self, token_iter: &mut TokenIterator) -> Result<FunctionDecl, ParseError> {
        // 解析函数名
        let name = match token_iter.next() {
            Some(Ok((Token::Ident(name), _))) => name,
            Some(Ok((token, _))) => return Err(ParseError::SyntaxError { message: format!("期望函数名，但得到 {:?}", token), span: token_iter.last_span }),
            Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            None => return Err(ParseError::SyntaxError { message: "函数定义不完整".to_string(), span: token_iter.last_span }),
        };

        // 解析参数
        let mut parameters = Vec::new();
        match token_iter.next() {
            Some(Ok((Token::LeftParen, _))) => {
                loop {
                    match token_iter.next() {
                        Some(Ok((Token::RightParen, _))) => break,
                        Some(Ok((Token::Ident(name), _))) => {
                            parameters.push(Parameter {
                                name,
                                type_annot: None,
                                default: None,
                            });
                            match token_iter.next() {
                                Some(Ok((Token::Comma, _))) => continue,
                                Some(Ok((Token::RightParen, _))) => break,
                                Some(Ok((t, _))) => return Err(ParseError::SyntaxError { message: format!("期望 , 或 )，但得到 {:?}", t), span: token_iter.last_span }),
                                Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
                                None => return Err(ParseError::SyntaxError { message: "参数列表不完整".to_string(), span: token_iter.last_span }),
                            }
                        }
                        Some(Ok((t, _))) => return Err(ParseError::SyntaxError { message: format!("期望参数名，但得到 {:?}", t), span: token_iter.last_span }),
                        Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
                        None => return Err(ParseError::SyntaxError { message: "参数列表不完整".to_string(), span: token_iter.last_span }),
                    }
                }
            }
            Some(Ok((token, _))) => return Err(ParseError::SyntaxError { message: format!("期望左括号，但得到 {:?}", token), span: token_iter.last_span }),
            Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            None => return Err(ParseError::SyntaxError { message: "函数定义不完整".to_string(), span: token_iter.last_span }),
        }

        // 解析函数体
        let body = match token_iter.next() {
            Some(Ok((Token::LeftBrace, _))) => self.parse_block(token_iter)?,
            Some(Ok((token, _))) => return Err(ParseError::SyntaxError { message: format!("期望左大括号，但得到 {:?}", token), span: token_iter.last_span }),
            Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            None => return Err(ParseError::SyntaxError { message: "函数定义不完整".to_string(), span: token_iter.last_span }),
        };

        Ok(FunctionDecl {
            name,
            parameters,
            return_type: None,
            body,
            is_async: false,
        })
    }

    fn parse_variable(&self, token_iter: &mut TokenIterator, is_mutable: bool) -> Result<VariableDecl, ParseError> {
        // 解析变量名
        let name = match token_iter.next() {
            Some(Ok((Token::Ident(name), _))) => name,
            Some(Ok((token, _))) => return Err(ParseError::SyntaxError { message: format!("期望变量名，但得到 {:?}", token), span: token_iter.last_span }),
            Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            None => return Err(ParseError::SyntaxError { message: "变量声明不完整".to_string(), span: token_iter.last_span }),
        };

        // 解析类型注解（可选）
        let mut type_annot = None;
        if let Some(Ok((Token::Colon, _))) = token_iter.peek() {
            token_iter.next(); // 消耗掉 colon
            type_annot = Some(Type::String); // 简单地默认是字符串类型
        }

        // 解析初始化值（可选）
        let mut initializer = None;
        if let Some(Ok((Token::Equals, _))) = token_iter.peek() {
            token_iter.next(); // 消耗掉 equals
            initializer = Some(self.parse_expression(token_iter)?);
        }

        Ok(VariableDecl {
            name,
            is_mutable,
            type_annot,
            initializer,
        })
    }

    fn parse_block(&self, token_iter: &mut TokenIterator) -> Result<Block, ParseError> {
        let mut statements = Vec::new();

        while let Some(token_result) = token_iter.next() {
            match token_result {
                Ok((Token::RightBrace, _)) => break,
                Ok((Token::Semicolon, _)) => continue,
                Ok((Token::Return, _)) => {
                    let ret_expr = if matches!(token_iter.peek(), Some(Ok((Token::Semicolon, _))) | Some(Ok((Token::RightBrace, _))) | None) {
                        None
                    } else {
                        Some(self.parse_expression(token_iter)?)
                    };
                    statements.push(Statement::Return(ret_expr));
                }
                Ok((Token::If, _)) => {
                    let condition = self.parse_expression(token_iter)?;
                    match token_iter.next() {
                        Some(Ok((Token::LeftBrace, _))) => {}
                        Some(Ok((t, _))) => return Err(ParseError::SyntaxError { message: format!("期望 {{，但得到 {:?}", t), span: token_iter.last_span }),
                        Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
                        None => return Err(ParseError::SyntaxError { message: "if 不完整".to_string(), span: token_iter.last_span }),
                    }
                    let then_block = self.parse_block(token_iter)?;
                    let else_block = if matches!(token_iter.peek(), Some(Ok((Token::Else, _)))) {
                        token_iter.next();
                        match token_iter.next() {
                            Some(Ok((Token::LeftBrace, _))) => {}
                            Some(Ok((t, _))) => return Err(ParseError::SyntaxError { message: format!("期望 {{，但得到 {:?}", t), span: token_iter.last_span }),
                            _ => return Err(ParseError::SyntaxError { message: "else 不完整".to_string(), span: token_iter.last_span }),
                        }
                        let blk = self.parse_block(token_iter)?;
                        Some(blk)
                    } else {
                        None
                    };
                    statements.push(Statement::If(IfStatement {
                        condition,
                        then_block,
                        else_block,
                    }));
                }
                Ok((Token::Val, _)) => {
                    let var = self.parse_variable(token_iter, false)?;
                    statements.push(Statement::Variable(var));
                }
                Ok((Token::Var, _)) => {
                    let var = self.parse_variable(token_iter, true)?;
                    statements.push(Statement::Variable(var));
                }
                Ok((Token::Ident(name), _)) => {
                    if matches!(token_iter.peek(), Some(Ok((Token::LeftParen, _)))) {
                        token_iter.next();
                        let arguments = self.parse_call_arguments(token_iter)?;
                        let call = Expression::Call(Box::new(Expression::Variable(name)), arguments);
                        statements.push(Statement::Expression(call));
                    } else {
                        let expr = Expression::Variable(name);
                        statements.push(Statement::Expression(expr));
                    }
                }
                Ok((token, _)) => {
                    return Err(ParseError::SyntaxError { message: format!("期望语句，但得到 {:?}", token), span: token_iter.last_span });
                }
                Err(e) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            }
        }

        Ok(Block { statements })
    }

    fn parse_call_arguments(&self, token_iter: &mut TokenIterator) -> Result<Vec<Expression>, ParseError> {
        let mut arguments = Vec::new();
        while !matches!(token_iter.peek(), Some(Ok((Token::RightParen, _))) | None) {
            arguments.push(self.parse_expression(token_iter)?);
            if matches!(token_iter.peek(), Some(Ok((Token::Comma, _)))) {
                token_iter.next();
            }
        }
        match token_iter.next() {
            Some(Ok((Token::RightParen, _))) => {}
            Some(Ok((t, _))) => return Err(ParseError::SyntaxError { message: format!("期望 )，但得到 {:?}", t), span: token_iter.last_span }),
            Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            None => return Err(ParseError::SyntaxError { message: "参数列表不完整".to_string(), span: token_iter.last_span }),
        }
        Ok(arguments)
    }

    fn parse_expression(&self, token_iter: &mut TokenIterator) -> Result<Expression, ParseError> {
        self.parse_comparison(token_iter)
    }

    fn parse_comparison(&self, token_iter: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive(token_iter)?;
        loop {
            let op = match token_iter.peek() {
                Some(Ok((Token::LessThanEquals, _))) => BinaryOp::LessEqual,
                Some(Ok((Token::GreaterThanEquals, _))) => BinaryOp::GreaterEqual,
                Some(Ok((Token::LessThan, _))) => BinaryOp::Less,
                Some(Ok((Token::GreaterThan, _))) => BinaryOp::Greater,
                Some(Ok((Token::DoubleEquals, _))) => BinaryOp::Equal,
                Some(Ok((Token::NotEquals, _))) => BinaryOp::NotEqual,
                _ => break,
            };
            token_iter.next(); // consume op
            let right = self.parse_additive(token_iter)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_additive(&self, token_iter: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative(token_iter)?;
        loop {
            let op = match token_iter.peek() {
                Some(Ok((Token::Plus, _))) => BinaryOp::Add,
                Some(Ok((Token::Minus, _))) => BinaryOp::Sub,
                _ => break,
            };
            token_iter.next();
            let right = self.parse_multiplicative(token_iter)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplicative(&self, token_iter: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_primary(token_iter)?;
        loop {
            let op = match token_iter.peek() {
                Some(Ok((Token::Asterisk, _))) => BinaryOp::Mul,
                Some(Ok((Token::Slash, _))) => BinaryOp::Div,
                Some(Ok((Token::Percent, _))) => BinaryOp::Mod,
                _ => break,
            };
            token_iter.next();
            let right = self.parse_primary(token_iter)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_primary(&self, token_iter: &mut TokenIterator) -> Result<Expression, ParseError> {
        let token_result = token_iter.next().ok_or_else(|| ParseError::SyntaxError { message: "缺少表达式".to_string(), span: None })?;
        let expr = match token_result {
            Ok((Token::DecimalInt(s), _)) => {
                let n = s.parse().map_err(|_| ParseError::SyntaxError { message: "无效整数".to_string(), span: token_iter.last_span })?;
                Expression::Literal(Literal::Integer(n))
            }
            Ok((Token::Float(s), _)) => {
                let f = s.parse().map_err(|_| ParseError::SyntaxError { message: "无效浮点数".to_string(), span: token_iter.last_span })?;
                Expression::Literal(Literal::Float(f))
            }
            Ok((Token::True, _)) => Expression::Literal(Literal::Boolean(true)),
            Ok((Token::False, _)) => Expression::Literal(Literal::Boolean(false)),
            Ok((Token::StringQuote, _)) => self.parse_string(token_iter)?,
            Ok((Token::StringContent(content), _)) => Expression::Literal(Literal::String(content)),
            Ok((Token::Ident(name), _)) => {
                if matches!(token_iter.peek(), Some(Ok((Token::LeftParen, _)))) {
                    token_iter.next();
                    let arguments = self.parse_call_arguments(token_iter)?;
                    Expression::Call(Box::new(Expression::Variable(name)), arguments)
                } else {
                    Expression::Variable(name)
                }
            }
            Ok((Token::LeftParen, _)) => {
                let inner = self.parse_expression(token_iter)?;
                match token_iter.next() {
                    Some(Ok((Token::RightParen, _))) => Expression::Parenthesized(Box::new(inner)),
                    Some(Ok((t, _))) => return Err(ParseError::SyntaxError { message: format!("期望 )，但得到 {:?}", t), span: token_iter.last_span }),
                    Some(Err(e)) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
                    None => return Err(ParseError::SyntaxError { message: "括号不匹配".to_string(), span: token_iter.last_span }),
                }
            }
            Ok((t, _)) => return Err(ParseError::SyntaxError { message: format!("期望表达式，但得到 {:?}", t), span: token_iter.last_span }),
            Err(e) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
        };
        Ok(expr)
    }

    fn parse_string(&self, token_iter: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut content = String::new();

        while let Some(token_result) = token_iter.next() {
            match token_result {
                Ok((Token::StringQuote, _)) => {
                    break;
                }
                Ok((Token::StringContent(s), _)) => {
                    content.push_str(&s);
                }
                Ok((token, _)) => return Err(ParseError::SyntaxError { message: format!("期望字符串内容，但得到 {:?}", token), span: token_iter.last_span }),
                Err(e) => return Err(ParseError::SyntaxError { message: e.to_string(), span: token_iter.last_span }),
            }
        }

        Ok(Expression::Literal(Literal::String(content)))
    }
}