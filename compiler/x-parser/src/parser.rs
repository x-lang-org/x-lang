use crate::ast::{
    spanned, BinaryOp, Block, CatchClause, ClassDecl, ClassMember, ClassModifiers, ConstructorDecl, Declaration,
    DoWhileStatement, Effect, EffectDecl, EnumDecl, EnumVariant, EnumVariantData, ExternFunctionDecl, ExportDecl, Expression, ExpressionKind, ForStatement, FunctionDecl,
    IfStatement, ImportDecl, ImportSymbol, ImplementDecl, Literal, MatchCase, MatchStatement, MethodModifiers, ModuleDecl,
    Parameter, Pattern, Program, RecordDecl, Statement, StatementKind, TraitDecl, TryStatement, Type,
    TypeAlias, TypeConstraint, TypeParameter, UnaryOp, VariableDecl, Visibility, WaitType, WhileStatement,
};
use crate::errors::ParseError;
use x_lexer::span::Span;
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

    /// 获取当前的 Span
    fn current_span(&self, ti: &TokenIterator) -> Span {
        ti.last_span.unwrap_or_default()
    }

    /// 创建带位置信息的表达式
    fn mk_expr(&self, ti: &TokenIterator, kind: ExpressionKind) -> Expression {
        spanned(kind, self.current_span(ti))
    }

    /// 创建带指定位置的表达式
    fn mk_expr_from_span(&self, kind: ExpressionKind, span: Span) -> Expression {
        spanned(kind, span)
    }

    /// 创建带位置信息的语句
    fn mk_stmt(&self, ti: &TokenIterator, kind: StatementKind) -> Statement {
        spanned(kind, self.current_span(ti))
    }

    fn parse_program(&self, ti: &mut TokenIterator) -> Result<Program, ParseError> {
        let mut declarations = Vec::new();
        let mut statements = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::Function, _)) => {
                    ti.next();
                    declarations.push(Declaration::Function(self.parse_function(ti, false, MethodModifiers::default())?));
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
                    // Check if export is a modifier followed by a declaration (export function, export class, etc.)
                    match ti.peek() {
                        Some(Ok((Token::Function, _))) => {
                            ti.next();
                            let func = self.parse_function(ti, false, MethodModifiers::default())?;
                            // Add export by adding an export declaration for the function name
                            let name = func.name.clone();
                            declarations.push(Declaration::Function(func));
                            declarations.push(Declaration::Export(ExportDecl {
                                symbol: name,
                                span: self.current_span(ti),
                            }));
                        }
                        Some(Ok((Token::Class, _))) => {
                            ti.next();
                            let class = self.parse_class(ti)?;
                            let name = class.name.clone();
                            declarations.push(Declaration::Class(class));
                            declarations.push(Declaration::Export(ExportDecl {
                                symbol: name,
                                span: self.current_span(ti),
                            }));
                        }
                        Some(Ok((Token::Enum, _))) => {
                            ti.next();
                            let enum_ = self.parse_enum(ti)?;
                            let name = enum_.name.clone();
                            declarations.push(Declaration::Enum(enum_));
                            declarations.push(Declaration::Export(ExportDecl {
                                symbol: name,
                                span: self.current_span(ti),
                            }));
                        }
                        Some(Ok((Token::Type, _))) => {
                            ti.next();
                            let alias = self.parse_type_alias(ti)?;
                            let name = alias.name.clone();
                            declarations.push(Declaration::TypeAlias(alias));
                            declarations.push(Declaration::Export(ExportDecl {
                                symbol: name,
                                span: self.current_span(ti),
                            }));
                        }
                        _ => {
                            // Traditional export declaration: export name;
                            declarations.push(Declaration::Export(self.parse_export(ti)?));
                        }
                    }
                }
                Ok((Token::Class, _)) => {
                    ti.next();
                    declarations.push(Declaration::Class(self.parse_class(ti)?));
                }
                Ok((Token::Struct, _)) => {
                    ti.next();
                    declarations.push(Declaration::Record(self.parse_record(ti)?));
                }
                Ok((Token::Trait, _)) => {
                    ti.next();
                    declarations.push(Declaration::Trait(self.parse_trait(ti)?));
                }
                Ok((Token::Implement, _)) => {
                    ti.next();
                    declarations.push(Declaration::Implement(self.parse_implement(ti)?));
                }
                Ok((Token::Interface, _)) => {
                    ti.next();
                    // interface 是 trait 的别名
                    declarations.push(Declaration::Trait(self.parse_trait(ti)?));
                }
                Ok((Token::Effect, _)) => {
                    ti.next();
                    declarations.push(Declaration::Effect(self.parse_effect(ti)?));
                }
                Ok((Token::Enum, _)) => {
                    ti.next();
                    declarations.push(Declaration::Enum(self.parse_enum(ti)?));
                }
                Ok((Token::Extern, _)) | Ok((Token::Foreign, _)) | Ok((Token::External, _)) => {
                    ti.next();
                    declarations.push(Declaration::ExternFunction(self.parse_extern_function(ti)?));
                }
                Ok((Token::Async, _)) => {
                    ti.next();
                    match ti.peek() {
                        Some(Ok((Token::Function, _))) => {
                            ti.next();
                            declarations.push(Declaration::Function(self.parse_function(ti, true, MethodModifiers::default())?));
                        }
                        _ => return Err(self.err("期望 'function' 在 'async' 之后", ti)),
                    }
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
        // Use a default span for now; in a more complete implementation,
        // we would track the start and end positions of the entire program
        let span = Span::default();
        Ok(Program {
            declarations,
            statements,
            span,
        })
    }

    fn parse_module(&self, ti: &mut TokenIterator) -> Result<ModuleDecl, ParseError> {
        let name = match self.expect_token(ti, "模块名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望模块名，但得到 {:?}", t), ti)),
        };
        self.eat_semi(ti);
        Ok(ModuleDecl {
            name,
            span: self.current_span(ti),
        })
    }

    fn parse_export(&self, ti: &mut TokenIterator) -> Result<ExportDecl, ParseError> {
        let symbol = match self.expect_token(ti, "导出符号")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望导出符号，但得到 {:?}", t), ti)),
        };
        self.eat_semi(ti);
        Ok(ExportDecl {
            symbol,
            span: self.current_span(ti),
        })
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
            span: self.current_span(ti),
        })
    }

    fn eat_mut(&self, ti: &mut TokenIterator) -> bool {
        if matches!(ti.peek(), Some(Ok((Token::Mut, _))))
            || matches!(ti.peek(), Some(Ok((Token::Mutable, _)))) {
            ti.next();
            true
        } else {
            false
        }
    }

    /// 解析类型参数列表 <T, U: Trait>
    fn parse_type_parameters(&self, ti: &mut TokenIterator) -> Result<Vec<TypeParameter>, ParseError> {
        // 检查是否有 < 开始类型参数列表
        if !matches!(ti.peek(), Some(Ok((Token::LessThan, _)))) {
            return Ok(Vec::new());
        }

        ti.next(); // 消费 <
        let mut type_params = Vec::new();

        loop {
            // 解析类型参数名
            let name = match self.expect_token(ti, "类型参数名")? {
                Token::Ident(n) => n,
                t => return Err(self.err(format!("期望类型参数名，但得到 {:?}", t), ti)),
            };

            // 解析可选的类型约束 T: Trait
            let constraints = if matches!(ti.peek(), Some(Ok((Token::Colon, _)))) {
                ti.next(); // 消费 :
                self.parse_type_constraints(ti)?
            } else {
                Vec::new()
            };

            type_params.push(TypeParameter {
                name,
                constraints,
                span: self.current_span(ti),
            });

            // 检查是继续还是结束
            match ti.peek() {
                Some(Ok((Token::Comma, _))) => {
                    ti.next();
                }
                Some(Ok((Token::GreaterThan, _))) => {
                    ti.next();
                    break;
                }
                _ => return Err(self.err("期望 , 或 >", ti)),
            }
        }

        Ok(type_params)
    }

    /// 解析类型约束 Trait1 + Trait2
    fn parse_type_constraints(&self, ti: &mut TokenIterator) -> Result<Vec<TypeConstraint>, ParseError> {
        let mut constraints = Vec::new();

        loop {
            let trait_name = match self.expect_token(ti, "Trait 名")? {
                Token::Ident(n) => n,
                t => return Err(self.err(format!("期望 Trait 名，但得到 {:?}", t), ti)),
            };

            constraints.push(TypeConstraint {
                trait_name,
                span: self.current_span(ti),
            });

            // 检查是否有 + 继续更多约束
            if matches!(ti.peek(), Some(Ok((Token::Plus, _)))) {
                ti.next();
            } else {
                break;
            }
        }

        Ok(constraints)
    }

    /// 解析效果列表 with IO, Async
    fn parse_effects(&self, ti: &mut TokenIterator) -> Result<Vec<Effect>, ParseError> {
        if !matches!(ti.peek(), Some(Ok((Token::With, _)))) {
            return Ok(Vec::new());
        }

        ti.next(); // 消费 with
        let mut effects = Vec::new();

        loop {
            let effect_name = match self.expect_token(ti, "效果名")? {
                Token::Ident(n) => n,
                t => return Err(self.err(format!("期望效果名，但得到 {:?}", t), ti)),
            };

            // 将效果名转换为 Effect 枚举
            let effect = match effect_name.as_str() {
                "IO" => Effect::IO,
                "Async" => Effect::Async,
                "NonDet" => Effect::NonDet,
                "State" => {
                    // 检查是否有 <Type> 参数
                    if matches!(ti.peek(), Some(Ok((Token::LessThan, _)))) {
                        ti.next();
                        let state_type = match self.expect_token(ti, "状态类型")? {
                            Token::Ident(n) => n,
                            t => return Err(self.err(format!("期望状态类型，但得到 {:?}", t), ti)),
                        };
                        if !matches!(ti.peek(), Some(Ok((Token::GreaterThan, _)))) {
                            return Err(self.err("期望 >", ti));
                        }
                        ti.next();
                        Effect::State(state_type)
                    } else {
                        Effect::Custom(effect_name)
                    }
                }
                "Throws" => {
                    // 检查是否有 <Type> 参数
                    if matches!(ti.peek(), Some(Ok((Token::LessThan, _)))) {
                        ti.next();
                        let error_type = match self.expect_token(ti, "错误类型")? {
                            Token::Ident(n) => n,
                            t => return Err(self.err(format!("期望错误类型，但得到 {:?}", t), ti)),
                        };
                        if !matches!(ti.peek(), Some(Ok((Token::GreaterThan, _)))) {
                            return Err(self.err("期望 >", ti));
                        }
                        ti.next();
                        Effect::Throws(error_type)
                    } else {
                        Effect::Custom(effect_name)
                    }
                }
                _ => Effect::Custom(effect_name),
            };

            effects.push(effect);

            // 检查是否有更多效果
            if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                ti.next();
            } else {
                break;
            }
        }

        Ok(effects)
    }

    fn parse_function(&self, ti: &mut TokenIterator, is_async: bool, modifiers: MethodModifiers) -> Result<FunctionDecl, ParseError> {
        let name = match self.expect_token(ti, "函数名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望函数名，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        match self.expect_token(ti, "(")? {
            Token::LeftParen => {}
            t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
        }

        let parameters = self.parse_param_list(ti)?;

        // 支持 -> 或 : 作为返回类型分隔符
        let return_type = if matches!(ti.peek(), Some(&Ok((Token::Arrow, _)))) {
            ti.next();
            Some(self.parse_type(ti)?)
        } else if matches!(ti.peek(), Some(&Ok((Token::Colon, _)))) {
            ti.next();
            Some(self.parse_type(ti)?)
        } else {
            None
        };

        // 解析效果注解 with IO, Async
        let effects = self.parse_effects(ti)?;

        let body = if matches!(ti.peek(), Some(&Ok((Token::Semicolon, _)))) {
            // 方法签名：没有方法体
            ti.next();
            Block { statements: vec![] }
        } else if matches!(ti.peek(), Some(&Ok((Token::RightBrace, _)))) {
            // 接口方法：没有方法体，也没有分号
            Block { statements: vec![] }
        } else if matches!(ti.peek(), Some(&Ok((Token::Equals, _)))) {
            ti.next();
            let expr = self.parse_expression(ti)?;
            self.eat_semi(ti);
            Block {
                statements: vec![self.mk_stmt(ti, StatementKind::Expression(expr))],
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
            type_parameters,
            parameters,
            return_type,
            effects,
            body,
            is_async,
            modifiers,
            span: self.current_span(ti),
        })
    }

    /// Parse extern function declaration
    /// extern "ABI" function name(params) -> return_type
    /// extern function name(params) -> return_type  (default to "C" ABI)
    fn parse_extern_function(&self, ti: &mut TokenIterator) -> Result<ExternFunctionDecl, ParseError> {
        // Parse optional ABI string
        let abi = if matches!(ti.peek(), Some(Ok((Token::StringContent(_), _)))) {
            match ti.next() {
                Some(Ok((Token::StringContent(s), _))) => s,
                _ => return Err(self.err("期望 ABI 字符串", ti)),
            }
        } else {
            "C".to_string() // default ABI
        };

        // Expect 'function' keyword
        match self.expect_token(ti, "function")? {
            Token::Function => {}
            t => return Err(self.err(format!("期望 'function'，但得到 {:?}", t), ti)),
        }

        // Parse function name
        let name = match self.expect_token(ti, "函数名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望函数名，但得到 {:?}", t), ti)),
        };

        // Parse type parameters <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        // Parse parameters
        match self.expect_token(ti, "(")? {
            Token::LeftParen => {}
            t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
        }

        let mut parameters = Vec::new();
        let mut is_variadic = false;

        // Check for empty parameter list
        if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
            ti.next();
        } else {
            loop {
                // Check for variadic marker: ... or .. followed by .
                // The lexer tokenizes ... as RangeExclusive (..) followed by Dot (.)
                if matches!(ti.peek(), Some(Ok((Token::RangeExclusive, _)))) {
                    // ... is tokenized as RangeExclusive (..) followed by Dot (.)
                    ti.next(); // consume ..
                    if matches!(ti.peek(), Some(Ok((Token::Dot, _)))) {
                        ti.next(); // consume final .
                        is_variadic = true;
                        // Expect closing paren
                        match self.expect_token(ti, ")")? {
                            Token::RightParen => {}
                            t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                        }
                        break;
                    } else {
                        return Err(self.err("期望 ...", ti));
                    }
                } else if matches!(ti.peek(), Some(Ok((Token::Dot, _)))) {
                    // Alternative: three separate dots
                    ti.next();
                    if matches!(ti.peek(), Some(Ok((Token::Dot, _)))) {
                        ti.next();
                        if matches!(ti.peek(), Some(Ok((Token::Dot, _)))) {
                            ti.next();
                            is_variadic = true;
                            // Expect closing paren
                            match self.expect_token(ti, ")")? {
                                Token::RightParen => {}
                                t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                            }
                            break;
                        } else {
                            return Err(self.err("期望 ...", ti));
                        }
                    } else {
                        return Err(self.err("期望 ...", ti));
                    }
                }

                let param_name = match self.expect_token(ti, "参数名")? {
                    Token::Ident(n) => n,
                    t => return Err(self.err(format!("期望参数名，但得到 {:?}", t), ti)),
                };

                let type_annot = if matches!(ti.peek(), Some(Ok((Token::Colon, _)))) {
                    ti.next();
                    Some(self.parse_type(ti)?)
                } else {
                    None
                };

                parameters.push(Parameter {
                    name: param_name,
                    type_annot,
                    default: None,
                    span: self.current_span(ti),
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
        }

        // Parse optional return type
        let return_type = if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
            ti.next();
            Some(self.parse_type(ti)?)
        } else if matches!(ti.peek(), Some(Ok((Token::Colon, _)))) {
            ti.next();
            Some(self.parse_type(ti)?)
        } else {
            None
        };

        // Expect semicolon
        self.eat_semi(ti);

        Ok(ExternFunctionDecl {
            abi,
            type_parameters,
            name,
            parameters,
            return_type,
            is_variadic,
            span: self.current_span(ti),
        })
    }

    fn parse_param_list(&self, ti: &mut TokenIterator) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        if matches!(ti.peek(), Some(&Ok((Token::RightParen, _)))) {
            ti.next();
            return Ok(params);
        }
        loop {
            let name = match self.expect_token(ti, "参数名")? {
                Token::Ident(n) => n,
                Token::RightParen => break,
                t => return Err(self.err(format!("期望参数名，但得到 {:?}", t), ti)),
            };
            let type_annot = if matches!(ti.peek(), Some(&Ok((Token::Colon, _)))) {
                ti.next();
                Some(self.parse_type(ti)?)
            } else {
                None
            };
            params.push(Parameter {
                name,
                type_annot,
                default: None,
                span: self.current_span(ti),
            });
            match ti.peek() {
                Some(&Ok((Token::Comma, _))) => {
                    ti.next();
                }
                Some(&Ok((Token::RightParen, _))) => {
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
            visibility: Visibility::default(),
            span: self.current_span(ti),
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

        // 允许尾随分号：`;` 之后直接 EOF（或 block 结束）不应报 "期望表达式"
        if ti.peek().is_none() || matches!(ti.peek(), Some(Ok((Token::RightBrace, _)))) {
            return Ok(self.mk_stmt(ti, StatementKind::Expression(
                self.mk_expr(ti, ExpressionKind::Literal(Literal::Unit))
            )));
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
                Ok(self.mk_stmt(ti, StatementKind::Return(expr)))
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
                Ok(self.mk_stmt(ti, StatementKind::Variable(var)))
            }
            Some(Ok((Token::Val, _))) => {
                ti.next();
                let var = self.parse_variable(ti, false)?;
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Variable(var)))
            }
            Some(Ok((Token::Var, _))) => {
                ti.next();
                let var = self.parse_variable(ti, true)?;
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Variable(var)))
            }
            Some(Ok((Token::Const, _))) => {
                ti.next();
                let var = self.parse_variable(ti, false)?;
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Variable(var)))
            }
            Some(Ok((Token::Try, _))) => {
                ti.next();
                self.parse_try(ti)
            }
            Some(Ok((Token::Match, _))) => {
                ti.next();
                self.parse_match(ti)
            }
            Some(Ok((Token::Break, _))) => {
                ti.next();
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Break))
            }
            Some(Ok((Token::Continue, _))) => {
                ti.next();
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Continue))
            }
            Some(Ok((Token::Ident(ref s), _))) if s == "do" => {
                ti.next();
                self.parse_do_while(ti)
            }
            Some(Ok((Token::Defer, _))) => {
                ti.next();
                let expr = self.parse_expression(ti)?;
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Defer(expr)))
            }
            Some(Ok((Token::Yield, _))) => {
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
                Ok(self.mk_stmt(ti, StatementKind::Yield(expr)))
            }
            Some(Ok((Token::Loop, _))) => {
                ti.next();
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let body = self.parse_block(ti)?;
                Ok(self.mk_stmt(ti, StatementKind::Loop(body)))
            }
            Some(Ok((Token::Unsafe, _))) => {
                ti.next();
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let body = self.parse_block(ti)?;
                Ok(self.mk_stmt(ti, StatementKind::Unsafe(body)))
            }
            _ => {
                let expr = self.parse_expression(ti)?;
                self.eat_semi(ti);
                Ok(self.mk_stmt(ti, StatementKind::Expression(expr)))
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

        Ok(self.mk_stmt(ti, StatementKind::Try(TryStatement {
            body,
            catch_clauses,
            finally_block,
        })))
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
                Ok((Token::Comma, _)) => {
                    // 跳过分隔逗号
                    ti.next();
                }
                Ok(_) => {
                    let pattern = self.parse_pattern(ti)?;
                    let guard = if matches!(ti.peek(), Some(Ok((Token::When, _)))) {
                        ti.next();
                        Some(self.parse_expression(ti)?)
                    } else {
                        None
                    };
                    // 支持 => 或 { 两种语法
                    let body = if matches!(ti.peek(), Some(Ok((Token::FatArrow, _)))) {
                        ti.next();
                        let expr = self.parse_expression(ti)?;
                        // 跳过可选的分隔符（逗号或分号）
                        if matches!(ti.peek(), Some(Ok((Token::Comma, _))) | Some(Ok((Token::Semicolon, _)))) {
                            ti.next();
                        }
                        Block {
                            statements: vec![self.mk_stmt(ti, StatementKind::Expression(expr))],
                        }
                    } else {
                        match self.expect_token(ti, "{")? {
                            Token::LeftBrace => {}
                            t => return Err(self.err(format!("期望 {{ 或 =>，但得到 {:?}", t), ti)),
                        }
                        self.parse_block(ti)?
                    };
                    cases.push(MatchCase {
                        pattern,
                        body,
                        guard,
                    });
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(self.mk_stmt(ti, StatementKind::Match(MatchStatement { expression, cases })))
    }

    fn parse_pattern(&self, ti: &mut TokenIterator) -> Result<Pattern, ParseError> {
        // 支持通配符/变量/字面量/枚举构造器/或模式
        let mut pat = match self.expect_token(ti, "pattern")? {
            Token::Ident(name) if name == "_" => Pattern::Wildcard,
            Token::Ident(name) => {
                // 检查是否是枚举构造器模式 TypeName.VariantName(...)
                if matches!(ti.peek(), Some(Ok((Token::Dot, _)))) {
                    ti.next(); // 消费 .
                    let variant_name = match self.expect_token(ti, "变体名")? {
                        Token::Ident(n) => n,
                        t => return Err(self.err(format!("期望变体名，但得到 {:?}", t), ti)),
                    };
                    // 检查是否有参数
                    let args = if matches!(ti.peek(), Some(Ok((Token::LeftParen, _)))) {
                        ti.next(); // 消费 (
                        let mut patterns = Vec::new();
                        loop {
                            if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
                                ti.next();
                                break;
                            }
                            let p = self.parse_pattern(ti)?;
                            patterns.push(p);
                            if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                                ti.next();
                            }
                        }
                        patterns
                    } else {
                        Vec::new()
                    };
                    Pattern::EnumConstructor(name, variant_name, args)
                } else {
                    Pattern::Variable(name)
                }
            }
            Token::True => Pattern::Literal(Literal::Boolean(true)),
            Token::False => Pattern::Literal(Literal::Boolean(false)),
            Token::Null => Pattern::Literal(Literal::Null),
            Token::DecimalInt(s) => Pattern::Literal(Literal::Integer(s.parse().unwrap_or(0))),
            Token::HexInt(s) => Pattern::Literal(Literal::Integer(i64::from_str_radix(&s, 16).unwrap_or(0))),
            Token::OctInt(s) => Pattern::Literal(Literal::Integer(i64::from_str_radix(&s, 8).unwrap_or(0))),
            Token::BinInt(s) => Pattern::Literal(Literal::Integer(i64::from_str_radix(&s, 2).unwrap_or(0))),
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

        // SPEC.md: 支持 if condition then { ... } 和 if condition { ... } 两种语法
        // 可选的 then 关键字
        if matches!(ti.peek(), Some(Ok((Token::Then, _)))) {
            ti.next(); // 消费 then
        }

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
                // 可选的 then 关键字在 else 分支
                if matches!(ti.peek(), Some(Ok((Token::Then, _)))) {
                    ti.next();
                }
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                Some(self.parse_block(ti)?)
            }
        } else {
            None
        };
        Ok(self.mk_stmt(ti, StatementKind::If(IfStatement {
            condition,
            then_block,
            else_block,
        })))
    }

    fn parse_while(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        let condition = self.parse_expression(ti)?;
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }
        let body = self.parse_block(ti)?;
        Ok(self.mk_stmt(ti, StatementKind::While(WhileStatement { condition, body })))
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
        Ok(self.mk_stmt(ti, StatementKind::DoWhile(DoWhileStatement { body, condition })))
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
            span: self.current_span(ti),
        })
    }

    fn parse_for(&self, ti: &mut TokenIterator) -> Result<Statement, ParseError> {
        // SPEC.md: for each item in collection { }
        // 也支持: for item in collection { } (向后兼容)
        // 可选的 each 关键字
        if matches!(ti.peek(), Some(Ok((Token::Each, _)))) {
            ti.next(); // 消费 each
        }

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
        Ok(self.mk_stmt(ti, StatementKind::For(ForStatement {
            pattern,
            iterator,
            body,
        })))
    }

    // ── Expression parsing (precedence climbing) ──

    fn parse_expression(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        self.parse_assignment(ti)
    }

    fn parse_pipe(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut expr = self.parse_null_coalescing(ti)?;
        while matches!(ti.peek(), Some(&Ok((Token::Pipe, _)))) {
            ti.next();
            let right = self.parse_null_coalescing(ti)?;
            expr = self.mk_expr(ti, ExpressionKind::Pipe(Box::new(expr), vec![Box::new(right)]));
        }
        Ok(expr)
    }

    fn parse_null_coalescing(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut expr = self.parse_or(ti)?;
        while matches!(ti.peek(), Some(&Ok((Token::DoubleQuestionMark, _)))) {
            ti.next();
            let right = self.parse_or(ti)?;
            expr = self.mk_expr(ti, ExpressionKind::NullCoalescing(Box::new(expr), Box::new(right)));
        }
        Ok(expr)
    }

    fn parse_assignment(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let expr = self.parse_pipe(ti)?;
        if matches!(ti.peek(), Some(&Ok((Token::Equals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(rhs))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::PlusEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::Add,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::MinusEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::Sub,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::AsteriskEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::Mul,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::SlashEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::Div,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::PercentEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::Mod,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        // Bitwise compound assignments
        if matches!(ti.peek(), Some(&Ok((Token::AmpersandEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::BitAnd,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::PipeEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::BitOr,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::CaretEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::BitXor,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::LeftShiftEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::LeftShift,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        if matches!(ti.peek(), Some(&Ok((Token::RightShiftEquals, _)))) {
            ti.next();
            let rhs = self.parse_assignment(ti)?;
            let expanded = self.mk_expr(ti, ExpressionKind::Binary(
                BinaryOp::RightShift,
                Box::new(expr.clone()),
                Box::new(rhs),
            ));
            return Ok(self.mk_expr(ti, ExpressionKind::Assign(Box::new(expr), Box::new(expanded))));
        }
        Ok(expr)
    }

    fn parse_or(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_and(ti)?;
        // Handle both || and 'or' keyword
        while matches!(ti.peek(), Some(&Ok((Token::OrOr, _))) | Some(&Ok((Token::Or, _)))) {
            ti.next();
            let right = self.parse_and(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Binary(BinaryOp::Or, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_and(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison(ti)?;
        // Handle both && and 'and' keyword
        while matches!(ti.peek(), Some(&Ok((Token::AndAnd, _))) | Some(&Ok((Token::And, _)))) {
            ti.next();
            let right = self.parse_comparison(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Binary(BinaryOp::And, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_comparison(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_bitwise(ti)?;
        loop {
            let op = match ti.peek() {
                Some(&Ok((Token::LessThanEquals, _))) => BinaryOp::LessEqual,
                Some(&Ok((Token::GreaterThanEquals, _))) => BinaryOp::GreaterEqual,
                Some(&Ok((Token::LessThan, _))) => BinaryOp::Less,
                Some(&Ok((Token::GreaterThan, _))) => BinaryOp::Greater,
                Some(&Ok((Token::DoubleEquals, _))) => BinaryOp::Equal,
                Some(&Ok((Token::NotEquals, _))) => BinaryOp::NotEqual,
                Some(&Ok((Token::Eq, _))) => BinaryOp::Equal,      // eq keyword
                Some(&Ok((Token::Ne, _))) => BinaryOp::NotEqual,   // ne keyword
                _ => break,
            };
            ti.next();
            let right = self.parse_bitwise(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Binary(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    /// Parse bitwise operations: &, |, ^
    /// Precedence: after shift, before comparison
    fn parse_bitwise(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_shift(ti)?;
        loop {
            let op = match ti.peek() {
                Some(&Ok((Token::Ampersand, _))) => BinaryOp::BitAnd,
                Some(&Ok((Token::VerticalBar, _))) => BinaryOp::BitOr,
                Some(&Ok((Token::Caret, _))) => BinaryOp::BitXor,
                _ => break,
            };
            ti.next();
            let right = self.parse_shift(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Binary(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    /// Parse shift operations: <<, >>
    /// Precedence: after additive, before bitwise
    fn parse_shift(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive(ti)?;
        loop {
            let op = match ti.peek() {
                Some(&Ok((Token::LeftShift, _))) => BinaryOp::LeftShift,
                Some(&Ok((Token::RightShift, _))) => BinaryOp::RightShift,
                _ => break,
            };
            ti.next();
            let right = self.parse_additive(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Binary(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_additive(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative(ti)?;
        loop {
            match ti.peek() {
                Some(&Ok((Token::Plus, _))) => {
                    ti.next();
                    let right = self.parse_multiplicative(ti)?;
                    left = self.mk_expr(ti, ExpressionKind::Binary(BinaryOp::Add, Box::new(left), Box::new(right)));
                }
                Some(&Ok((Token::Minus, _))) => {
                    ti.next();
                    let right = self.parse_multiplicative(ti)?;
                    left = self.mk_expr(ti, ExpressionKind::Binary(BinaryOp::Sub, Box::new(left), Box::new(right)));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_cast(ti)?;
        loop {
            let op = match ti.peek() {
                Some(&Ok((Token::Asterisk, _))) => BinaryOp::Mul,
                Some(&Ok((Token::Slash, _))) => BinaryOp::Div,
                Some(&Ok((Token::Percent, _))) => BinaryOp::Mod,
                _ => break,
            };
            ti.next();
            let right = self.parse_cast(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Binary(op, Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    /// Handle casting: expression `expr as Type`
    fn parse_cast(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary(ti)?;
        while matches!(ti.peek(), Some(&Ok((Token::As, _)))) {
            ti.next();
            // Right-hand side is a type
            let ty = self.parse_type(ti)?;
            left = self.mk_expr(ti, ExpressionKind::Cast(Box::new(left), ty));
        }
        Ok(left)
    }

    fn parse_unary(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        match ti.peek() {
            Some(&Ok((Token::Minus, _))) => {
                ti.next();
                let operand = self.parse_unary(ti)?;
                Ok(self.mk_expr(ti, ExpressionKind::Unary(UnaryOp::Negate, Box::new(operand))))
            }
            Some(&Ok((Token::NotOperator, _))) | Some(&Ok((Token::Not, _))) => {
                ti.next();
                let operand = self.parse_unary(ti)?;
                Ok(self.mk_expr(ti, ExpressionKind::Unary(UnaryOp::Not, Box::new(operand))))
            }
            Some(&Ok((Token::Tilde, _))) => {
                ti.next();
                let operand = self.parse_unary(ti)?;
                Ok(self.mk_expr(ti, ExpressionKind::Unary(UnaryOp::BitNot, Box::new(operand))))
            }
            Some(&Ok((Token::Wait, _))) => {
                ti.next();
                self.parse_wait_expression(ti)
            }
            _ => self.parse_postfix(ti),
        }
    }

    /// Parse wait expression: wait expr, wait together { ... }, wait race { ... }, wait timeout(n) { ... }
    fn parse_wait_expression(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        match ti.peek() {
            // wait together { e1, e2, ... }
            Some(&Ok((Token::Together, _))) => {
                ti.next();
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let exprs = self.parse_wait_expr_list(ti)?;
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}，但得到 {:?}", t), ti)),
                }
                Ok(self.mk_expr(ti, ExpressionKind::Wait(WaitType::Together, exprs)))
            }
            // wait race { e1, e2, ... }
            Some(&Ok((Token::Race, _))) => {
                ti.next();
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let exprs = self.parse_wait_expr_list(ti)?;
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}，但得到 {:?}", t), ti)),
                }
                Ok(self.mk_expr(ti, ExpressionKind::Wait(WaitType::Race, exprs)))
            }
            // wait timeout(duration) { expr }
            Some(&Ok((Token::Timeout, _))) => {
                ti.next();
                match self.expect_token(ti, "(")? {
                    Token::LeftParen => {}
                    t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
                }
                let timeout_expr = self.parse_expression(ti)?;
                match self.expect_token(ti, ")")? {
                    Token::RightParen => {}
                    t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                }
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let exprs = self.parse_wait_expr_list(ti)?;
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}，但得到 {:?}", t), ti)),
                }
                Ok(self.mk_expr(ti, ExpressionKind::Wait(WaitType::Timeout(Box::new(timeout_expr)), exprs)))
            }
            // wait atomic { expr } - 原子执行，不会被其他线程中断
            Some(&Ok((Token::Atomic, _))) => {
                ti.next();
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let exprs = self.parse_wait_expr_list(ti)?;
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}，但得到 {:?}", t), ti)),
                }
                Ok(self.mk_expr(ti, ExpressionKind::Wait(WaitType::Atomic, exprs)))
            }
            // wait retry { expr } - 自动重试
            Some(&Ok((Token::Retry, _))) => {
                ti.next();
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }
                let exprs = self.parse_wait_expr_list(ti)?;
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}，但得到 {:?}", t), ti)),
                }
                Ok(self.mk_expr(ti, ExpressionKind::Wait(WaitType::Retry, exprs)))
            }
            // wait expr (single await)
            _ => {
                let operand = self.parse_postfix(ti)?;
                Ok(self.mk_expr(ti, ExpressionKind::Wait(WaitType::Single, vec![operand])))
            }
        }
    }

    /// Parse comma-separated expressions inside wait { ... }
    fn parse_wait_expr_list(&self, ti: &mut TokenIterator) -> Result<Vec<Expression>, ParseError> {
        let mut exprs = Vec::new();
        // Handle empty list
        if matches!(ti.peek(), Some(&Ok((Token::RightBrace, _)))) {
            return Ok(exprs);
        }
        loop {
            let expr = self.parse_expression(ti)?;
            exprs.push(expr);
            match ti.peek() {
                Some(&Ok((Token::Comma, _))) => {
                    ti.next();
                }
                Some(&Ok((Token::RightBrace, _))) => break,
                _ => break,
            }
        }
        Ok(exprs)
    }

    fn parse_postfix(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary(ti)?;
        loop {
            match ti.peek() {
                Some(Ok((Token::LeftParen, _))) => {
                    ti.next();
                    let args = self.parse_call_arguments(ti)?;
                    expr = self.mk_expr(ti, ExpressionKind::Call(Box::new(expr), args));
                }
                Some(Ok((Token::LeftBracket, _))) => {
                    ti.next();
                    let index = self.parse_expression(ti)?;
                    match self.expect_token(ti, "]")? {
                        Token::RightBracket => {}
                        t => return Err(self.err(format!("期望 ]，但得到 {:?}", t), ti)),
                    }
                    expr = self.mk_expr(ti, ExpressionKind::Member(Box::new(expr), format!("__index__{}", 0)));
                    expr = self.mk_expr(ti, ExpressionKind::Call(
                        Box::new(self.mk_expr(ti, ExpressionKind::Variable("__index__".to_string()))),
                        vec![
                            {
                                if let ExpressionKind::Member(inner, _) = expr.node {
                                    *inner
                                } else {
                                    unreachable!()
                                }
                            },
                            index,
                        ],
                    ));
                }
                Some(Ok((Token::Dot, _))) => {
                    ti.next();
                    let member = match ti.peek() {
                        Some(Ok((Token::Ident(n), _))) => n.clone(),
                        // 允许关键字作为成员名（如 .null, .true, .false 等）
                        Some(Ok((Token::Null, _))) => "null".to_string(),
                        Some(Ok((Token::True, _))) => "true".to_string(),
                        Some(Ok((Token::False, _))) => "false".to_string(),
                        Some(Ok((t, _))) => {
                            return Err(self.err(format!("期望成员名，但得到 {:?}", t), ti));
                        }
                        None => return Err(self.err("意外的输入结束", ti)),
                        Some(Err(e)) => return Err(self.err(e.to_string(), ti)),
                    };
                    ti.next(); // 消费成员名 token
                    expr = self.mk_expr(ti, ExpressionKind::Member(Box::new(expr), member));
                }
                Some(Ok((Token::RangeExclusive, _))) => {
                    ti.next();
                    let right = self.parse_primary(ti)?;
                    expr = self.mk_expr(ti, ExpressionKind::Range(Box::new(expr), Box::new(right), false));
                }
                Some(Ok((Token::RangeInclusive, _))) => {
                    ti.next();
                    let right = self.parse_primary(ti)?;
                    expr = self.mk_expr(ti, ExpressionKind::Range(Box::new(expr), Box::new(right), true));
                }
                Some(Ok((Token::QuestionMark, _))) => {
                    // ? 错误传播运算符
                    ti.next();
                    expr = self.mk_expr(ti, ExpressionKind::TryPropagate(Box::new(expr)));
                }
                Some(Ok((Token::QuestionMarkDot, _))) => {
                    // ?. 可选链成员访问
                    ti.next();
                    let member = match ti.peek() {
                        Some(Ok((Token::Ident(n), _))) => n.clone(),
                        // 允许关键字作为成员名
                        Some(Ok((Token::Null, _))) => "null".to_string(),
                        Some(Ok((Token::True, _))) => "true".to_string(),
                        Some(Ok((Token::False, _))) => "false".to_string(),
                        Some(Ok((t, _))) => {
                            return Err(self.err(format!("期望成员名，但得到 {:?}", t), ti));
                        }
                        None => return Err(self.err("意外的输入结束", ti)),
                        Some(Err(e)) => return Err(self.err(e.to_string(), ti)),
                    };
                    ti.next();
                    expr = self.mk_expr(ti, ExpressionKind::OptionalChain(Box::new(expr), member));
                }
                Some(Ok((Token::DoubleColon, _))) => {
                    // Enum::Variant - enum variant constructor access
                    ti.next();
                    let variant = match ti.peek() {
                        Some(Ok((Token::Ident(n), _))) => n.clone(),
                        // Allow keywords as variant names
                        Some(Ok((Token::Null, _))) => "null".to_string(),
                        Some(Ok((Token::True, _))) => "true".to_string(),
                        Some(Ok((Token::False, _))) => "false".to_string(),
                        Some(Ok((t, _))) => {
                            return Err(self.err(format!("期望变体名，但得到 {:?}", t), ti));
                        }
                        None => return Err(self.err("意外的输入结束", ti)),
                        Some(Err(e)) => return Err(self.err(e.to_string(), ti)),
                    };
                    ti.next();
                    expr = self.mk_expr(ti, ExpressionKind::Member(Box::new(expr), variant));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// 从已有表达式继续解析后缀操作
    fn parse_postfix_continue(&self, ti: &mut TokenIterator, mut expr: Expression) -> Result<Expression, ParseError> {
        loop {
            match ti.peek() {
                Some(Ok((Token::LeftParen, _))) => {
                    ti.next();
                    let args = self.parse_call_arguments(ti)?;
                    expr = self.mk_expr(ti, ExpressionKind::Call(Box::new(expr), args));
                }
                Some(Ok((Token::LeftBracket, _))) => {
                    ti.next();
                    let index = self.parse_expression(ti)?;
                    match self.expect_token(ti, "]")? {
                        Token::RightBracket => {}
                        t => return Err(self.err(format!("期望 ]，但得到 {:?}", t), ti)),
                    }
                    expr = self.mk_expr(ti, ExpressionKind::Member(Box::new(expr), format!("__index__{}", 0)));
                    expr = self.mk_expr(ti, ExpressionKind::Call(
                        Box::new(self.mk_expr(ti, ExpressionKind::Variable("__index__".to_string()))),
                        vec![
                            {
                                if let ExpressionKind::Member(inner, _) = expr.node {
                                    *inner
                                } else {
                                    unreachable!()
                                }
                            },
                            index,
                        ],
                    ));
                }
                Some(Ok((Token::Dot, _))) => {
                    ti.next();
                    let member = match ti.peek() {
                        Some(Ok((Token::Ident(n), _))) => n.clone(),
                        // 允许关键字作为成员名（如 .null, .true, .false 等）
                        Some(Ok((Token::Null, _))) => "null".to_string(),
                        Some(Ok((Token::True, _))) => "true".to_string(),
                        Some(Ok((Token::False, _))) => "false".to_string(),
                        Some(Ok((t, _))) => {
                            return Err(self.err(format!("期望成员名，但得到 {:?}", t), ti));
                        }
                        None => return Err(self.err("意外的输入结束", ti)),
                        Some(Err(e)) => return Err(self.err(e.to_string(), ti)),
                    };
                    ti.next(); // 消费成员名 token
                    expr = self.mk_expr(ti, ExpressionKind::Member(Box::new(expr), member));
                }
                Some(Ok((Token::RangeExclusive, _))) => {
                    ti.next();
                    let right = self.parse_primary(ti)?;
                    expr = self.mk_expr(ti, ExpressionKind::Range(Box::new(expr), Box::new(right), false));
                }
                Some(Ok((Token::RangeInclusive, _))) => {
                    ti.next();
                    let right = self.parse_primary(ti)?;
                    expr = self.mk_expr(ti, ExpressionKind::Range(Box::new(expr), Box::new(right), true));
                }
                Some(Ok((Token::QuestionMark, _))) => {
                    ti.next();
                    expr = self.mk_expr(ti, ExpressionKind::TryPropagate(Box::new(expr)));
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
                Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Integer(n))))
            }
            Token::HexInt(s) => {
                let n: i64 = i64::from_str_radix(&s, 16).map_err(|_| self.err("无效十六进制整数", ti))?;
                Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Integer(n))))
            }
            Token::OctInt(s) => {
                let n: i64 = i64::from_str_radix(&s, 8).map_err(|_| self.err("无效八进制整数", ti))?;
                Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Integer(n))))
            }
            Token::BinInt(s) => {
                let n: i64 = i64::from_str_radix(&s, 2).map_err(|_| self.err("无效二进制整数", ti))?;
                Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Integer(n))))
            }
            Token::Float(s) => {
                let f: f64 = s.parse().map_err(|_| self.err("无效浮点数", ti))?;
                Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Float(f))))
            }
            Token::True => Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Boolean(true)))),
            Token::False => Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Boolean(false)))),
            Token::Null => Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Null))),

            Token::Await => {
                // await expression
                ti.next();
                let expr = self.parse_expression(ti)?;
                Ok(self.mk_expr(ti, ExpressionKind::Await(Box::new(expr))))
            }

            Token::StringQuote => self.parse_string(ti),
            Token::RawStringQuote => self.parse_raw_string(ti),
            Token::StringContent(c) => Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::String(c)))),
            Token::CharContent(c) => {
                let ch = c.chars().next().unwrap_or('\0');
                Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Char(ch))))
            }
            Token::Ident(name) => {
                // 检查是否是 lambda: identifier -> expr
                if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
                    ti.next(); // 消费 ->
                    let params = vec![Parameter {
                        name,
                        type_annot: None,
                        default: None,
                        span: self.current_span(ti),
                    }];
                    let body_expr = self.parse_expression(ti)?;
                    let block = Block {
                        statements: vec![self.mk_stmt(ti, StatementKind::Expression(body_expr))],
                    };
                    return Ok(self.mk_expr(ti, ExpressionKind::Lambda(params, block)));
                }
                // 检查是否是函数调用
                if matches!(ti.peek(), Some(Ok((Token::LeftParen, _)))) {
                    // Some/None/Ok/Err 等作为枚举构造函数处理
                    ti.next();
                    let (positional_args, named_fields) = self.parse_call_or_record_arguments(ti)?;
                    if let Some(fields) = named_fields {
                        // Record construction: TypeName(field1: value1, field2: value2)
                        Ok(self.mk_expr(ti, ExpressionKind::Record(name, fields)))
                    } else {
                        // Regular function call (including enum constructors like Some, None, Ok, Err)
                        Ok(self.mk_expr(ti, ExpressionKind::Call(
                            Box::new(self.mk_expr(ti, ExpressionKind::Variable(name))),
                            positional_args,
                        )))
                    }
                } else {
                    Ok(self.mk_expr(ti, ExpressionKind::Variable(name)))
                }
            }
            Token::When => self.parse_when(ti),
            Token::Given => {
                // given expression { ... match cases ... }
                ti.next();
                let discriminant = self.parse_expression(ti)?;
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
                        Ok((Token::Is, _)) => {
                            ti.next();
                            let pattern = self.parse_pattern(ti)?;
                            match self.expect_token(ti, "=>")? {
                                Token::FatArrow => {}
                                t => return Err(self.err(format!("期望 =>，但得到 {:?}", t), ti)),
                            }
                            let body_expr = self.parse_expression(ti)?;
                            // 将表达式包装为单个语句的块
                            let body = Block {
                                statements: vec![self.mk_stmt(ti, StatementKind::Expression(body_expr))],
                            };
                            cases.push(MatchCase {
                                pattern,
                                guard: None,
                                body,
                            });
                            // 可选逗号分隔
                            if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                                ti.next();
                            }
                        }
                        Ok((Token::When, _)) => {
                            // guard 模式: is pattern when condition => body
                            ti.next();
                            let pattern = self.parse_pattern(ti)?;
                            let guard = self.parse_expression(ti)?;
                            match self.expect_token(ti, "=>")? {
                                Token::FatArrow => {}
                                t => return Err(self.err(format!("期望 =>，但得到 {:?}", t), ti)),
                            }
                            let body_expr = self.parse_expression(ti)?;
                            // 将表达式包装为单个语句的块
                            let body = Block {
                                statements: vec![self.mk_stmt(ti, StatementKind::Expression(body_expr))],
                            };
                            cases.push(MatchCase {
                                pattern,
                                guard: Some(guard),
                                body,
                            });
                            if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                                ti.next();
                            }
                        }
                        _ => {
                            return Err(self.err("期望 is 开始匹配分支，或 } 结束匹配", ti));
                        }
                    }
                }
                Ok(self.mk_expr(ti, ExpressionKind::Match(Box::new(discriminant), cases)))
            }
            Token::LeftParen => {
                if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
                    // 处理空括号 () 作为 Unit 类型
                    ti.next();
                    // 检查是否是空参数 lambda: () -> expr
                    if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
                        ti.next(); // 消费 ->
                        let body_expr = self.parse_expression(ti)?;
                        let block = Block {
                            statements: vec![self.mk_stmt(ti, StatementKind::Expression(body_expr))],
                        };
                        return Ok(self.mk_expr(ti, ExpressionKind::Lambda(Vec::new(), block)));
                    }
                    Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::Unit)))
                } else {
                    // 尝试解析为 lambda 参数或表达式
                    let first = self.parse_expression(ti)?;

                    // 检查是否后面跟着逗号（多参数 lambda）或右括号
                    if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                        // 可能是多参数 lambda
                        let mut params = vec![];

                        // 检查第一个是否是变量
                        if let ExpressionKind::Variable(name) = first.node.clone() {
                            params.push(Parameter {
                                name,
                                type_annot: None,
                                default: None,
                                span: first.span,
                            });
                        } else {
                            // 不是参数列表，解析为元组/数组
                            // 回退：将 first 作为第一个元素，继续解析
                            let mut elements = vec![first];
                            while matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                                ti.next();
                                if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
                                    break;
                                }
                                elements.push(self.parse_expression(ti)?);
                            }
                            match self.expect_token(ti, ")")? {
                                Token::RightParen => {}
                                t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                            }
                            // 返回最后一个元素作为括号表达式的结果（简化处理）
                            if elements.len() == 1 {
                                return Ok(self.mk_expr(ti, ExpressionKind::Parenthesized(Box::new(elements.remove(0)))));
                            }
                            // 对于多个元素，返回最后一个
                            return Ok(self.mk_expr(ti, ExpressionKind::Parenthesized(Box::new(elements.pop().unwrap()))));
                        }

                        // 继续解析更多参数
                        while matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                            ti.next();
                            if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
                                break;
                            }
                            let param_expr = self.parse_expression(ti)?;
                            if let ExpressionKind::Variable(name) = param_expr.node.clone() {
                                params.push(Parameter {
                                    name,
                                    type_annot: None,
                                    default: None,
                                    span: param_expr.span,
                                });
                            } else {
                                // 不是参数列表，返回错误
                                return Err(self.err("期望参数名", ti));
                            }
                        }

                        // 消费右括号
                        match self.expect_token(ti, ")")? {
                            Token::RightParen => {}
                            t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                        }

                        // 检查是否是 lambda (后面跟着 ->)
                        if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
                            ti.next(); // 消费 ->
                            let body_expr = self.parse_expression(ti)?;
                            let block = Block {
                                statements: vec![self.mk_stmt(ti, StatementKind::Expression(body_expr))],
                            };
                            return Ok(self.mk_expr(ti, ExpressionKind::Lambda(params, block)));
                        } else {
                            // 不是 lambda，是语法错误（多参数必须有 ->）
                            return Err(self.err("期望 ->", ti));
                        }
                    } else {
                        // 单个表达式
                        match self.expect_token(ti, ")")? {
                            Token::RightParen => {}
                            t => return Err(self.err(format!("期望 )，但得到 {:?}", t), ti)),
                        }
                        // 检查是否是单参数 lambda
                        if let ExpressionKind::Variable(name) = first.node.clone() {
                            if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
                                ti.next(); // 消费 ->
                                let params = vec![Parameter {
                                    name,
                                    type_annot: None,
                                    default: None,
                                    span: first.span,
                                }];
                                let body_expr = self.parse_expression(ti)?;
                                let block = Block {
                                    statements: vec![self.mk_stmt(ti, StatementKind::Expression(body_expr))],
                                };
                                return Ok(self.mk_expr(ti, ExpressionKind::Lambda(params, block)));
                            }
                        }
                        Ok(self.mk_expr(ti, ExpressionKind::Parenthesized(Box::new(first))))
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
                Ok(self.mk_expr(ti, ExpressionKind::Array(elems)))
            }
            Token::LeftBrace => {
                // 处理对象字面量 (map)
                let mut pairs = Vec::new();
                if !matches!(ti.peek(), Some(Ok((Token::RightBrace, _)))) {
                    loop {
                        // 解析键
                        let key = self.expect_token(ti, "键")?;
                        let key_expr = match key {
                            Token::Ident(name) => self.mk_expr(ti, ExpressionKind::Literal(Literal::String(name))),
                            Token::StringContent(s) => self.mk_expr(ti, ExpressionKind::Literal(Literal::String(s))),
                            _ => return Err(self.err(format!("期望键，但得到 {:?}", key), ti)),
                        };

                        // 解析 => 或 : (支持两种分隔符)
                        match self.expect_token(ti, "=> 或 :")? {
                            Token::FatArrow | Token::Colon => {}
                            t => return Err(self.err(format!("期望 => 或 :，但得到 {:?}", t), ti)),
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
                Ok(self.mk_expr(ti, ExpressionKind::Dictionary(pairs)))
            }
            Token::Function => {
                // Lambda 表达式: function(params) => expr 或 function(params) { block }
                // 解析参数列表
                match self.expect_token(ti, "(")? {
                    Token::LeftParen => {}
                    t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
                }
                let params = self.parse_param_list(ti)?;

                // 检查是 => expr 还是 { block }
                if matches!(ti.peek(), Some(Ok((Token::FatArrow, _)))) {
                    // function(params) => expr 形式
                    ti.next();
                    let expr = self.parse_expression(ti)?;
                    // 将单个表达式包装为 Block
                    let block = Block {
                        statements: vec![self.mk_stmt(ti, StatementKind::Expression(expr))],
                    };
                    Ok(self.mk_expr(ti, ExpressionKind::Lambda(params, block)))
                } else if matches!(ti.peek(), Some(Ok((Token::LeftBrace, _)))) {
                    // function(params) { block } 形式
                    ti.next();
                    let block = self.parse_block(ti)?;
                    Ok(self.mk_expr(ti, ExpressionKind::Lambda(params, block)))
                } else {
                    Err(self.err("期望 => 或 { 在 lambda 参数之后", ti))
                }
            }
            Token::Handle => {
                // handle expr with { EffectName -> handler, ... }
                // 解析被处理的表达式
                let inner_expr = self.parse_expression(ti)?;

                // 期望 with 关键字
                match self.expect_token(ti, "with")? {
                    Token::With => {}
                    t => return Err(self.err(format!("期望 with，但得到 {:?}", t), ti)),
                }

                // 期望 {
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }

                // 解析 effect handlers
                let mut handlers = Vec::new();
                if !matches!(ti.peek(), Some(Ok((Token::RightBrace, _)))) {
                    loop {
                        // 解析 effect 名称
                        let effect_name = match self.expect_token(ti, "effect name")? {
                            Token::Ident(name) => name,
                            t => return Err(self.err(format!("期望 effect 名称，但得到 {:?}", t), ti)),
                        };

                        // 期望 ->
                        match self.expect_token(ti, "->")? {
                            Token::Arrow => {}
                            t => return Err(self.err(format!("期望 ->，但得到 {:?}", t), ti)),
                        }

                        // 解析 handler 表达式
                        let handler_expr = self.parse_expression(ti)?;
                        handlers.push((effect_name, handler_expr));

                        match ti.peek() {
                            Some(Ok((Token::Comma, _))) => {
                                ti.next();
                            }
                            _ => break,
                        }
                    }
                }

                // 期望 }
                match self.expect_token(ti, "}")? {
                    Token::RightBrace => {}
                    t => return Err(self.err(format!("期望 }}，但得到 {:?}", t), ti)),
                }

                Ok(self.mk_expr(ti, ExpressionKind::Handle(Box::new(inner_expr), handlers)))
            }
            t => Err(self.err(format!("期望表达式，但得到 {:?}", t), ti)),
        }
    }

    fn parse_when(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let condition = self.parse_expression(ti)?;

        // Check if it's pattern matching form: when expr is { ... }
        // or ternary form: when expr then expr else expr
        match ti.peek() {
            Some(Ok((Token::Is, _))) => {
                // Pattern matching form: when expr is { pattern => result }
                // Convert to Match expression internally
                ti.next(); // consume 'is'
                match self.expect_token(ti, "{")? {
                    Token::LeftBrace => {}
                    t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
                }

                // Parse pattern matching cases
                let mut cases = Vec::new();
                while let Some(token_result) = ti.peek() {
                    match token_result {
                        Ok((Token::RightBrace, _)) => {
                            ti.next();
                            break;
                        }
                        Ok((Token::Comma, _)) => {
                            // 跳过分隔逗号
                            ti.next();
                        }
                        Ok(_) => {
                            // Parse pattern
                            let pattern = self.parse_pattern(ti)?;

                            // Check for optional guard: pattern if condition => result
                            let guard = if matches!(ti.peek(), Some(Ok((Token::If, _)))) {
                                ti.next(); // consume 'if'
                                Some(self.parse_expression(ti)?)
                            } else {
                                None
                            };

                            // Expect =>
                            match self.expect_token(ti, "=>")? {
                                Token::FatArrow => {}
                                t => return Err(self.err(format!("期望 =>，但得到 {:?}", t), ti)),
                            }

                            // Parse result expression and convert to Block
                            let result_expr = self.parse_expression(ti)?;
                            // 跳过可选的分隔符（逗号或分号）
                            if matches!(ti.peek(), Some(Ok((Token::Comma, _))) | Some(Ok((Token::Semicolon, _)))) {
                                ti.next();
                            }
                            let body = Block {
                                statements: vec![Statement {
                                    span: result_expr.span,
                                    node: StatementKind::Expression(result_expr),
                                }],
                            };

                            cases.push(MatchCase { pattern, body, guard });
                        }
                        Err(e) => return Err(self.err(e.to_string(), ti)),
                    }
                }

                return Ok(self.mk_expr(ti, ExpressionKind::Match(Box::new(condition), cases)));
            }
            _ => {
                // Ternary form: when expr then expr else expr
                // Accept either "then" keyword or expression followed by "else"
            }
        }

        // Try to parse as ternary: when expr then expr else expr
        // Check if next token looks like "then" (as identifier) or expression
        let then_expr = self.parse_expression(ti)?;

        // Look for "else" keyword
        match ti.peek() {
            Some(Ok((Token::Else, _))) => {
                ti.next();
            }
            Some(Ok((Token::Ident(ref s), _))) if s == "else" => {
                ti.next();
            }
            _ => {
                return Err(self.err(format!("期望 else，但得到 {:?}",
                    ti.peek().map(|t| format!("{:?}", t.as_ref().unwrap().0)).unwrap_or_default()), ti));
            }
        }
        let else_expr = self.parse_expression(ti)?;

        Ok(self.mk_expr(ti, ExpressionKind::If(
            Box::new(condition),
            Box::new(then_expr),
            Box::new(else_expr),
        )))
    }

    fn parse_call_arguments(&self, ti: &mut TokenIterator) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
            ti.next();
            return Ok(args);
        }
        loop {
            // 直接解析表达式，这样可以正确处理二元运算符等
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

    /// Parse call arguments that may be either positional or named (field: value)
    /// Returns a tuple of (positional_args, named_fields)
    /// If named_fields is Some, it's a record construction
    fn parse_call_or_record_arguments(
        &self,
        ti: &mut TokenIterator,
    ) -> Result<(Vec<Expression>, Option<Vec<(String, Expression)>>), ParseError> {
        let mut positional_args = Vec::new();
        let mut named_fields: Option<Vec<(String, Expression)>> = None;

        if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
            ti.next();
            return Ok((positional_args, None));
        }

        loop {
            // Check if this looks like a named argument (identifier followed by colon)
            // Parse expression and check if it's a binary expression with colon-like pattern
            let expr = self.parse_expression(ti)?;

            // Check if this was actually a named argument by looking for pattern:
            // We parsed an identifier, and now the next token is colon
            // Actually, we need to detect this BEFORE parsing the expression
            // Let's use a different approach: check if the expression is just an identifier
            // and if the NEXT token after parsing is a colon (which would be an error case)

            // Actually, let's detect it differently:
            // The expression we just parsed could be part of a "name: value" pair if:
            // 1. It's a simple variable reference
            // 2. The next token is a colon (but we already consumed past it in expression parsing)

            // Let me try yet another approach: check before parsing
            let is_named = if let ExpressionKind::Variable(name) = &expr.node {
                // This expression is just a variable - it might be the name in "name: value"
                // Check if there's a colon following (but we can't easily do this now)
                // Store and handle later
                Some(name.clone())
            } else {
                None
            };

            // If we have an identifier and the next token is colon, this is a named arg
            if let Some(name) = is_named {
                // Check if there's a colon (this would mean the expression parser didn't consume it)
                // But wait - the expression parser should have parsed "name: value" as a binary op
                // Actually no, colon is not a binary operator

                // Let's check if the colon is next
                if matches!(ti.peek(), Some(Ok((Token::Colon, _)))) {
                    // This is a named argument!
                    ti.next(); // consume colon
                    let value = self.parse_expression(ti)?;
                    if named_fields.is_none() {
                        named_fields = Some(Vec::new());
                    }
                    named_fields.as_mut().unwrap().push((name, value));
                } else {
                    // Not a named argument, it's a positional argument
                    if named_fields.is_some() {
                        return Err(self.err("命名参数后不能有位置参数", ti));
                    }
                    positional_args.push(expr);
                }
            } else {
                // Not an identifier, must be positional
                if named_fields.is_some() {
                    return Err(self.err("命名参数后不能有位置参数", ti));
                }
                positional_args.push(expr);
            }

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

        Ok((positional_args, named_fields))
    }

    fn parse_type(&self, ti: &mut TokenIterator) -> Result<Type, ParseError> {
        // Handle unit type: ()
        if matches!(ti.peek(), Some(&Ok((Token::LeftParen, _)))) {
            ti.next(); // consume (
            if matches!(ti.peek(), Some(&Ok((Token::RightParen, _)))) {
                ti.next(); // consume )
                return Ok(Type::Unit);
            } else {
                return Err(self.err("期望 ) 来形成 unit 类型", ti));
            }
        }

        // Handle reference types: &T or &mut T
        if matches!(ti.peek(), Some(&Ok((Token::Ampersand, _)))) {
            ti.next(); // consume &
            // Check for &mut (mut can be Token::Mut or Token::Mutable keyword)
            let is_mutable = matches!(ti.peek(), Some(&Ok((Token::Mut, _))))
                || matches!(ti.peek(), Some(&Ok((Token::Mutable, _))));
            if is_mutable {
                ti.next(); // consume mut/mutable
            }
            let inner_type = self.parse_type(ti)?;
            return if is_mutable {
                Ok(Type::MutableReference(Box::new(inner_type)))
            } else {
                Ok(Type::Reference(Box::new(inner_type)))
            };
        }

        // Handle pointer types: *T or *const T
        if matches!(ti.peek(), Some(&Ok((Token::Asterisk, _)))) {
            ti.next(); // consume *
            // Check for *const (const can be Token::Const keyword)
            let is_const = matches!(ti.peek(), Some(&Ok((Token::Const, _))))
                || matches!(ti.peek(), Some(&Ok((Token::Ident(ref name), _))) if name == "const");
            if is_const {
                ti.next(); // consume const
            }
            let inner_type = self.parse_type(ti)?;
            return if is_const {
                Ok(Type::ConstPointer(Box::new(inner_type)))
            } else {
                Ok(Type::Pointer(Box::new(inner_type)))
            };
        }

        let tok = self.expect_token(ti, "类型名")?;
        let base_type_name = match tok {
            Token::Ident(name) => name,
            t => return Err(self.err(format!("期望类型名，但得到 {:?}", t), ti)),
        };

        // 处理 signed / unsigned 类型（支持完整形式：signed 32-bit integer, unsigned 16-bit integer）
        if base_type_name == "unsigned" || base_type_name == "signed" {
            let next_tok = self.expect_token(ti, "类型名或数字")?;
            match next_tok {
                // short form: signed integer / unsigned integer
                Token::Ident(ref name) if name == "integer" => {
                    if base_type_name == "unsigned" {
                        return Ok(Type::UnsignedInt);
                    } else {
                        return Ok(Type::Int);
                    }
                }
                // full form: signed N-bit integer / unsigned N-bit integer
                // N is a number, then -, then bit, then integer
                Token::DecimalInt(_) => {
                    // consume N (already consumed by expect_token), consume '-', consume 'bit'
                    self.expect_token(ti, "'-' 分隔符")?; // the '-' in 32-bit
                    self.expect_token(ti, "'bit'")?;       // the 'bit' in 32-bit
                    self.expect_token(ti, "integer")?;     // the final 'integer' keyword
                    if base_type_name == "unsigned" {
                        return Ok(Type::UnsignedInt);
                    } else {
                        return Ok(Type::Int);
                    }
                }
                Token::Ident(_) => {
                    // could be something like "long" - we just ignore it and expect integer next
                    self.expect_token(ti, "integer")?;
                    if base_type_name == "unsigned" {
                        return Ok(Type::UnsignedInt);
                    } else {
                        return Ok(Type::Int);
                    }
                }
                t => return Err(self.err(format!("期望 integer 或 <N>-bit 在 {} 之后，但得到 {:?}", base_type_name, t), ti)),
            }
        }

        // 处理内置类型
        // 小写：值类型 (integer, float, boolean, string, character)
        // 大写：引用类型 (Integer, Float, Boolean, String, Character)
        let base_type = match base_type_name.as_str() {
            // 值类型（小写）
            "integer" | "Int" | "Int64" | "i32" | "i64" => Type::Int,
            "float" | "Float" | "Float64" | "f64" => Type::Float,
            "boolean" | "Bool" => Type::Bool,
            "string" | "String" => Type::String,
            "character" | "char" | "Char" => Type::Char,
            "unit" | "Unit" => Type::Unit,
            "never" | "Never" => Type::Never,
            // FFI 类型
            "void" | "Void" => Type::Void,
            "u32" | "u64" | "usize" | "U32" | "U64" | "Usize" => Type::UnsignedInt,
            // C FFI 类型
            "CInt" | "c_int" => Type::CInt,
            "CUInt" | "c_uint" => Type::CUInt,
            "CLong" | "c_long" => Type::CLong,
            "CULong" | "c_ulong" => Type::CULong,
            "CLongLong" | "c_longlong" => Type::CLongLong,
            "CULongLong" | "c_ulonglong" => Type::CULongLong,
            "CFloat" | "c_float" => Type::CFloat,
            "CDouble" | "c_double" => Type::CDouble,
            "CChar" | "c_char" => Type::CChar,
            "CSize" | "c_size_t" | "c_size" => Type::CSize,
            "CString" | "c_string" => Type::CString,
            _ => Type::Generic(base_type_name.clone()),
        };

        // Check for generic arguments
        // <T> - angle bracket syntax for Array<Int>, Option<Int>
        // (T) - tuple-style syntax only for specific types: Pointer(Void)
        let next_is_less = matches!(ti.peek(), Some(&Ok((Token::LessThan, _))));
        if !next_is_less {
            // Check for (T) syntax - only allowed for specific type names
            let next_is_lparen = matches!(ti.peek(), Some(&Ok((Token::LeftParen, _))));
            if next_is_lparen {
                let is_allowed_type = matches!(base_type_name.as_str(), "Pointer" | "ConstPointer" | "Option" | "Result" | "Array");
                if is_allowed_type {
                    // Parse generic arguments in parentheses
                    ti.next();
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type(ti)?);
                        if matches!(ti.peek(), Some(&Ok((Token::RightParen, _)))) {
                            ti.next();
                            break;
                        }
                        if matches!(ti.peek(), Some(&Ok((Token::Comma, _)))) {
                            ti.next();
                        } else {
                            return Err(self.err("期望 , 或 )", ti));
                        }
                    }

                    // Continue with generic type construction
                } else {
                    // Not allowed - just return the base type
                    return Ok(base_type);
                }
            } else {
                // No generic arguments - return base type directly
                return Ok(base_type);
            }
        }

        // Has <T> - parse angle bracket generics
        ti.next();
        let mut type_args = Vec::new();
        loop {
            type_args.push(self.parse_type(ti)?);
            if matches!(ti.peek(), Some(&Ok((Token::GreaterThan, _)))) {
                ti.next();
                break;
            }
            if matches!(ti.peek(), Some(&Ok((Token::Comma, _)))) {
                ti.next();
            } else {
                return Err(self.err("期望 , 或 >", ti));
            }
        }

        // 根据类型名决定如何构造结果类型
            match base_type_name.as_str() {
                // 内置泛型类型
                "Option" => {
                    if type_args.len() != 1 {
                        return Err(self.err("Option 类型需要一个类型参数", ti));
                    }
                    Ok(Type::Option(Box::new(type_args.remove(0))))
                }
                "Result" => {
                    if type_args.len() != 2 {
                        return Err(self.err("Result 类型需要两个类型参数", ti));
                    }
                    let err_type = type_args.remove(1);
                    let ok_type = type_args.remove(0);
                    Ok(Type::Result(Box::new(ok_type), Box::new(err_type)))
                }
                "Array" => {
                    if type_args.len() != 1 {
                        return Err(self.err("Array 类型需要一个类型参数", ti));
                    }
                    Ok(Type::Array(Box::new(type_args.remove(0))))
                }
                "Dictionary" => {
                    if type_args.len() != 2 {
                        return Err(self.err("Dictionary 类型需要两个类型参数", ti));
                    }
                    let val_type = type_args.remove(1);
                    let key_type = type_args.remove(0);
                    Ok(Type::Dictionary(Box::new(key_type), Box::new(val_type)))
                }
                "Async" => {
                    if type_args.len() != 1 {
                        return Err(self.err("Async 类型需要一个类型参数", ti));
                    }
                    Ok(Type::Async(Box::new(type_args.remove(0))))
                }
                // Pointer(T) 语法支持
                "Pointer" => {
                    if type_args.len() != 1 {
                        return Err(self.err("Pointer 类型需要一个类型参数", ti));
                    }
                    Ok(Type::Pointer(Box::new(type_args.remove(0))))
                }
                // 用户定义的泛型类型
                _ => Ok(Type::TypeConstructor(base_type_name, type_args)),
            }
    }

    fn parse_string(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        let mut parts: Vec<Expression> = Vec::new();

        while let Some(token_result) = ti.next() {
            match token_result {
                Ok((Token::StringQuote, _)) => break,
                Ok((Token::MultilineStringQuote, _)) => break,
                Ok((Token::StringContent(s), _)) => {
                    // Add string literal part
                    let expr = self.mk_expr(ti, ExpressionKind::Literal(Literal::String(s)));
                    parts.push(expr);
                }
                Ok((Token::InterpolateStart, _)) => {
                    // Parse the interpolated expression
                    let expr = self.parse_expression(ti)?;
                    // Expect InterpolateEnd after the expression
                    match ti.next() {
                        Some(Ok((Token::InterpolateEnd, _))) => {},
                        Some(Ok((_t, _))) => {
                            return Err(self.err("期望 }} 结束字符串插值".to_string(), ti));
                        }
                        Some(Err(e)) => return Err(ParseError::LexError(e)),
                        None => return Err(self.err("字符串插值未闭合".to_string(), ti)),
                    }
                    parts.push(expr);
                }
                Ok((t, _)) => return Err(self.err(format!("期望字符串内容或插值，但得到 {:?}", t), ti)),
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        // If no parts, return empty string
        if parts.is_empty() {
            return Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::String(String::new()))));
        }

        // If only one part, return it directly
        if parts.len() == 1 {
            return Ok(parts.remove(0));
        }

        // Desugar to a sequence of Binary(Add) operations: a + b + c + ...
        let mut result = parts.remove(0);
        for part in parts {
            let span = Span::new(result.span.start, part.span.end);
            result = spanned(
                ExpressionKind::Binary(BinaryOp::Add, Box::new(result), Box::new(part)),
                span,
            );
        }

        Ok(result)
    }

    /// 解析原始字符串（反引号包围）
    /// 原始字符串的内容在 lexer 中已经包含在 StringContent 中，且闭合反引号已被消费
    fn parse_raw_string(&self, ti: &mut TokenIterator) -> Result<Expression, ParseError> {
        // 下一个 token 应该是 StringContent，包含原始字符串内容
        let content = match ti.next() {
            Some(Ok((Token::StringContent(s), _))) => s,
            Some(Ok((t, _))) => return Err(self.err(format!("期望原始字符串内容，但得到 {:?}", t), ti)),
            Some(Err(e)) => return Err(self.err(e.to_string(), ti)),
            None => return Err(self.err("原始字符串未闭合".to_string(), ti)),
        };
        Ok(self.mk_expr(ti, ExpressionKind::Literal(Literal::String(content))))
    }

    // ── Class and Trait parsing ──

    /// 解析类声明
    /// class Name [extends ParentClass] [implement Trait1, Trait2] { ... }
    fn parse_class(&self, ti: &mut TokenIterator) -> Result<ClassDecl, ParseError> {
        // 解析类修饰符（abstract/final）
        let mut class_modifiers = ClassModifiers::default();

        if matches!(ti.peek(), Some(Ok((Token::Abstract, _)))) {
            ti.next();
            class_modifiers.is_abstract = true;
        }

        if matches!(ti.peek(), Some(Ok((Token::Final, _)))) {
            ti.next();
            class_modifiers.is_final = true;
        }

        let name = match self.expect_token(ti, "类名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望类名，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        // 可选的 extends
        let extends = if matches!(ti.peek(), Some(Ok((Token::Extends, _)))) {
            ti.next();
            match self.expect_token(ti, "父类名")? {
                Token::Ident(n) => Some(n),
                t => return Err(self.err(format!("期望父类名，但得到 {:?}", t), ti)),
            }
        } else {
            None
        };

        // 可选的 implement/implements
        let implements = if matches!(ti.peek(), Some(Ok((Token::Implement, _))) | Some(Ok((Token::Implements, _)))) {
            ti.next();
            let mut traits = Vec::new();
            loop {
                match self.expect_token(ti, "trait名")? {
                    Token::Ident(n) => traits.push(n),
                    t => return Err(self.err(format!("期望trait名，但得到 {:?}", t), ti)),
                }
                if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                    ti.next();
                } else {
                    break;
                }
            }
            traits
        } else {
            Vec::new()
        };

        // 类体
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let members = self.parse_class_body(ti)?;

        Ok(ClassDecl {
            name,
            type_parameters,
            extends,
            implements,
            members,
            modifiers: class_modifiers,
            span: self.current_span(ti),
        })
    }

    /// 解析类体
    fn parse_class_body(&self, ti: &mut TokenIterator) -> Result<Vec<ClassMember>, ParseError> {
        let mut members = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    break;
                }
                Ok(_) => {
                    let member = self.parse_class_member(ti)?;
                    members.push(member);
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(members)
    }

    /// 解析类成员
    fn parse_class_member(&self, ti: &mut TokenIterator) -> Result<ClassMember, ParseError> {
        // 检查是否是构造函数: new(...)
        if matches!(ti.peek(), Some(Ok((Token::New, _)))) {
            ti.next();
            return self.parse_constructor(ti);
        }

        // 检查是否是方法: [modifiers] function name(...) 或直接 name(...)
        // 先收集修饰符
        let mut modifiers = MethodModifiers::default();

        // 检查是否是 let 开头的字段定义
        if matches!(ti.peek(), Some(Ok((Token::Let, _)))) {
            ti.next();
            let field = self.parse_field_with_visibility(ti, modifiers.visibility)?;
            return Ok(ClassMember::Field(field));
        }

        // 解析 virtual/override/final/abstract/static 修饰符
        if matches!(ti.peek(), Some(Ok((Token::Virtual, _)))) {
            ti.next();
            modifiers.is_virtual = true;
        }

        if matches!(ti.peek(), Some(Ok((Token::Override, _)))) {
            ti.next();
            modifiers.is_override = true;
        }

        if matches!(ti.peek(), Some(Ok((Token::Final, _)))) {
            ti.next();
            modifiers.is_final = true;
        }

        if matches!(ti.peek(), Some(Ok((Token::Abstract, _)))) {
            ti.next();
            modifiers.is_abstract = true;
        }

        if matches!(ti.peek(), Some(Ok((Token::Static, _)))) {
            ti.next();
            modifiers.is_static = true;
        }

        // 解析可见性修饰符
        if matches!(ti.peek(), Some(Ok((Token::Private, _)))) {
            ti.next();
            modifiers.visibility = Visibility::Private;
        } else if matches!(ti.peek(), Some(Ok((Token::Public, _)))) {
            ti.next();
            modifiers.visibility = Visibility::Public;
        } else if matches!(ti.peek(), Some(Ok((Token::Protected, _)))) {
            ti.next();
            modifiers.visibility = Visibility::Protected;
        } else if matches!(ti.peek(), Some(Ok((Token::Internal, _)))) {
            ti.next();
            modifiers.visibility = Visibility::Internal;
        }

        // 检查是否是 function 关键字开头的方法
        if matches!(ti.peek(), Some(Ok((Token::Function, _)))) {
            ti.next();
            let method = self.parse_function(ti, false, modifiers)?;
            return Ok(ClassMember::Method(method));
        }

        // 检查是否是 new 关键字开头的构造函数（带可见性修饰符）
        if matches!(ti.peek(), Some(Ok((Token::New, _)))) {
            ti.next();
            return self.parse_constructor_with_visibility(ti, modifiers.visibility);
        }

        // 否则可能是字段声明: [mut] name: Type 或 name: Type = value
        let field = self.parse_field_with_visibility(ti, modifiers.visibility)?;
        Ok(ClassMember::Field(field))
    }

    /// 解析构造函数（不带可见性修饰符）
    /// new(params) { body }
    fn parse_constructor(&self, ti: &mut TokenIterator) -> Result<ClassMember, ParseError> {
        // 解析可见性修饰符
        let visibility = if matches!(ti.peek(), Some(Ok((Token::Private, _)))) {
            ti.next();
            Visibility::Private
        } else if matches!(ti.peek(), Some(Ok((Token::Public, _)))) {
            ti.next();
            Visibility::Public
        } else if matches!(ti.peek(), Some(Ok((Token::Protected, _)))) {
            ti.next();
            Visibility::Protected
        } else {
            Visibility::default()
        };

        self.parse_constructor_with_visibility(ti, visibility)
    }

    /// 解析构造函数（带已解析的可见性修饰符）
    fn parse_constructor_with_visibility(&self, ti: &mut TokenIterator, visibility: Visibility) -> Result<ClassMember, ParseError> {
        match self.expect_token(ti, "(")? {
            Token::LeftParen => {}
            t => return Err(self.err(format!("期望 (，但得到 {:?}", t), ti)),
        }

        let parameters = self.parse_param_list(ti)?;

        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let body = self.parse_block(ti)?;

        Ok(ClassMember::Constructor(ConstructorDecl { parameters, body, visibility }))
    }

    /// 解析字段声明
    /// name: Type [= value]
    fn parse_field_with_visibility(&self, ti: &mut TokenIterator, visibility: Visibility) -> Result<VariableDecl, ParseError> {
        let is_mutable = if matches!(ti.peek(), Some(Ok((Token::Mut, _))))
            || matches!(ti.peek(), Some(Ok((Token::Mutable, _)))) {
            ti.next();
            true
        } else {
            false
        };

        let name = match self.expect_token(ti, "字段名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望字段名，但得到 {:?}", t), ti)),
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

        self.eat_semi(ti);

        Ok(VariableDecl {
            name,
            is_mutable,
            type_annot,
            initializer,
            visibility,
            span: self.current_span(ti),
        })
    }

    /// 解析 trait 声明
    /// trait Name { [method signatures] }
    fn parse_trait(&self, ti: &mut TokenIterator) -> Result<TraitDecl, ParseError> {
        let name = match self.expect_token(ti, "trait名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望trait名，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        // 可选的 extends (trait 继承)
        let extends: Vec<String> = if matches!(ti.peek(), Some(Ok((Token::Extends, _)))) {
            ti.next();
            let mut traits = Vec::new();
            loop {
                match self.expect_token(ti, "父trait名")? {
                    Token::Ident(n) => traits.push(n),
                    t => return Err(self.err(format!("期望父trait名，但得到 {:?}", t), ti)),
                }
                if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                    ti.next();
                } else {
                    break;
                }
            }
            traits
        } else {
            Vec::new()
        };

        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let mut methods = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    break;
                }
                Ok((Token::Function, _)) => {
                    ti.next();
                    let method = self.parse_function(ti, false, MethodModifiers::default())?;
                    methods.push(method);
                }
                Ok((Token::Ident(_), _)) => {
                    // 可能是简写的方法签名: name(params) -> ReturnType;
                    let method = self.parse_trait_method(ti)?;
                    methods.push(method);
                }
                Ok(_) => {
                    return Err(self.err("期望方法声明或 }", ti));
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(TraitDecl {
            name,
            type_parameters,
            extends,
            methods,
            span: self.current_span(ti),
        })
    }

    /// 解析 trait 实现：implement Trait for Type { ... }
    fn parse_implement(&self, ti: &mut TokenIterator) -> Result<ImplementDecl, ParseError> {
        // 解析 trait 名称
        let trait_name = match self.expect_token(ti, "trait名称")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望trait名称，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        // 必须有 for 关键字
        match self.expect_token(ti, "for 关键字")? {
            Token::For => {}
            t => return Err(self.err(format!("期望关键字 'for'，但得到 {:?}", t), ti)),
        }

        // 解析目标类型
        let target_type = self.parse_type(ti)?;

        // 解析 where 子句（可选）
        let mut where_clause = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::Where, _)))) {
            ti.next();
            loop {
                // 期望类型参数名
                let _param_name = match ti.peek() {
                    Some(Ok((Token::Ident(name), _))) => {
                        let name = name.clone();
                        ti.next();
                        name
                    }
                    Some(Ok((Token::Colon, _))) => {
                        // 语法错误：缺少类型参数名
                        return Err(self.err("期望类型参数名在 ':' 之前", ti));
                    }
                    _ => break,
                };

                match self.expect_token(ti, ":")? {
                    Token::Colon => {}
                    t => return Err(self.err(format!("期望 ':'，但得到 {:?}", t), ti)),
                }

                // 读取 trait 约束
                match self.expect_token(ti, "trait约束名")? {
                    Token::Ident(trait_name) => {
                        where_clause.push(TypeConstraint {
                            trait_name,
                            span: self.current_span(ti),
                        });
                    }
                    t => return Err(self.err(format!("期望trait约束名，但得到 {:?}", t), ti)),
                }

                if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                    ti.next();
                } else {
                    break;
                }
            }
        }

        // 期望 { 开始方法实现
        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let mut methods = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    break;
                }
                Ok((Token::Function, _)) => {
                    ti.next();
                    let method = self.parse_function(ti, false, MethodModifiers::default())?;
                    methods.push(method);
                }
                Ok((Token::Mut, _)) | Ok((Token::Mutable, _)) => {
                    // mutable method
                    ti.next();
                    match ti.peek() {
                        Some(Ok((Token::Function, _))) => {
                            ti.next();
                            let method = self.parse_function(ti, false, MethodModifiers::default())?;
                            methods.push(method);
                        }
                        _ => return Err(self.err("期望 'function' 关键字", ti)),
                    }
                }
                Ok(_) => {
                    return Err(self.err("期望方法声明或 }", ti));
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(ImplementDecl {
            trait_name,
            target_type,
            type_parameters,
            where_clause,
            methods,
            span: self.current_span(ti),
        })
    }

    /// 解析 trait 方法签名
    /// name(params) -> ReturnType;
    fn parse_trait_method(&self, ti: &mut TokenIterator) -> Result<FunctionDecl, ParseError> {
        let name = match self.expect_token(ti, "方法名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望方法名，但得到 {:?}", t), ti)),
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

        self.eat_semi(ti);

        // trait 方法签名没有方法体，默认为抽象方法
        Ok(FunctionDecl {
            name,
            type_parameters: Vec::new(),
            parameters,
            return_type,
            effects: Vec::new(),
            body: Block { statements: vec![] },
            is_async: false,
            modifiers: MethodModifiers {
                is_abstract: true,
                ..MethodModifiers::default()
            },
            span: self.current_span(ti),
        })
    }

    /// 解析枚举声明
    /// enum Name<T> { Variant1, Variant2(T), Variant3 { field: Type } }
    fn parse_enum(&self, ti: &mut TokenIterator) -> Result<EnumDecl, ParseError> {
        let name = match self.expect_token(ti, "enum名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望enum名，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let mut variants = Vec::new();

        loop {
            let peek_result = ti.peek().cloned();
            match peek_result {
                Some(Ok((Token::RightBrace, _))) => {
                    ti.next();
                    break;
                }
                Some(Ok((Token::Comma, _))) => {
                    ti.next();
                }
                Some(Ok((Token::Ident(name), span))) => {
                    ti.next();
                    let variant = self.parse_enum_variant(ti, name, None, span)?;
                    variants.push(variant);
                }
                Some(Ok(_)) => {
                    return Err(self.err("期望枚举变体或 }", ti));
                }
                Some(Err(e)) => return Err(self.err(e.to_string(), ti)),
                None => break,
            }
        }

        Ok(EnumDecl {
            name,
            type_parameters,
            variants,
            span: self.current_span(ti),
        })
    }

    /// 解析枚举变体
    fn parse_enum_variant(
        &self,
        ti: &mut TokenIterator,
        name: String,
        doc: Option<String>,
        span: Span,
    ) -> Result<EnumVariant, ParseError> {
        let data = match ti.peek() {
            Some(Ok((Token::LeftParen, _))) => {
                // 元组式变体: Some(T, U)
                ti.next();
                let mut types = Vec::new();
                loop {
                    if matches!(ti.peek(), Some(Ok((Token::RightParen, _)))) {
                        ti.next();
                        break;
                    }
                    let ty = self.parse_type(ti)?;
                    types.push(ty);
                    if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                        ti.next();
                    }
                }
                EnumVariantData::Tuple(types)
            }
            Some(Ok((Token::LeftBrace, _))) => {
                // 记录式变体: Point { x: Int, y: Int }
                ti.next();
                let mut fields = Vec::new();
                loop {
                    if matches!(ti.peek(), Some(Ok((Token::RightBrace, _)))) {
                        ti.next();
                        break;
                    }
                    let field_name = match self.expect_token(ti, "字段名")? {
                        Token::Ident(n) => n,
                        t => return Err(self.err(format!("期望字段名，但得到 {:?}", t), ti)),
                    };
                    match self.expect_token(ti, ":")? {
                        Token::Colon => {}
                        t => return Err(self.err(format!("期望 :，但得到 {:?}", t), ti)),
                    }
                    let field_type = self.parse_type(ti)?;
                    fields.push((field_name, field_type));
                    if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                        ti.next();
                    }
                }
                EnumVariantData::Record(fields)
            }
            _ => EnumVariantData::Unit,
        };

        Ok(EnumVariant {
            name,
            data,
            doc,
            span,
        })
    }

    /// 解析记录声明：`record Name { field: Type, ... }`
    fn parse_record(&self, ti: &mut TokenIterator) -> Result<RecordDecl, ParseError> {
        let name = match self.expect_token(ti, "record名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望record名，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        // 解析 where 子句（可选）
        let mut where_clause = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::Where, _)))) {
            ti.next();
            loop {
                // 期望类型参数名
                let _param_name = match ti.peek() {
                    Some(Ok((Token::Ident(name), _))) => {
                        let name = name.clone();
                        ti.next();
                        name
                    }
                    Some(Ok((Token::Colon, _))) => {
                        return Err(self.err("期望类型参数名在 ':' 之前", ti));
                    }
                    _ => break,
                };

                match self.expect_token(ti, ":")? {
                    Token::Colon => {}
                    t => return Err(self.err(format!("期望 ':'，但得到 {:?}", t), ti)),
                }

                // 读取 trait 约束
                match self.expect_token(ti, "trait约束名")? {
                    Token::Ident(trait_name) => {
                        where_clause.push(TypeConstraint {
                            trait_name,
                            span: self.current_span(ti),
                        });
                    }
                    t => return Err(self.err(format!("期望trait约束名，但得到 {:?}", t), ti)),
                }

                if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                    ti.next();
                } else {
                    break;
                }
            }
        }

        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let mut fields = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    break;
                }
                Ok((Token::Comma, _)) => {
                    ti.next();
                    continue;
                }
                Ok((Token::Ident(field_name), _)) => {
                    let field_name = field_name.clone();
                    ti.next();
                    match self.expect_token(ti, ":")? {
                        Token::Colon => {}
                        t => return Err(self.err(format!("期望 :，但得到 {:?}", t), ti)),
                    }
                    let field_type = self.parse_type(ti)?;
                    fields.push((field_name, field_type));
                    if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                        ti.next();
                    }
                }
                Ok(_) => {
                    return Err(self.err("期望字段或 }", ti));
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(RecordDecl {
            name,
            type_parameters,
            fields,
            where_clause,
            span: self.current_span(ti),
        })
    }

    /// 解析效果声明：`effect Name<T> { op: Input -> Output, ... }`
    fn parse_effect(&self, ti: &mut TokenIterator) -> Result<EffectDecl, ParseError> {
        let name = match self.expect_token(ti, "effect名")? {
            Token::Ident(n) => n,
            t => return Err(self.err(format!("期望effect名，但得到 {:?}", t), ti)),
        };

        // 解析类型参数 <T, U: Trait>
        let type_parameters = self.parse_type_parameters(ti)?;

        // 解析 where 子句（可选）
        let mut where_clause = Vec::new();
        if matches!(ti.peek(), Some(Ok((Token::Where, _)))) {
            ti.next();
            loop {
                // 期望类型参数名
                let _param_name = match ti.peek() {
                    Some(Ok((Token::Ident(name), _))) => {
                        let name = name.clone();
                        ti.next();
                        name
                    }
                    Some(Ok((Token::Colon, _))) => {
                        return Err(self.err("期望类型参数名在 ':' 之前", ti));
                    }
                    _ => break,
                };

                match self.expect_token(ti, ":")? {
                    Token::Colon => {}
                    t => return Err(self.err(format!("期望 ':'，但得到 {:?}", t), ti)),
                }

                // 读取 trait 约束
                match self.expect_token(ti, "trait约束名")? {
                    Token::Ident(trait_name) => {
                        where_clause.push(TypeConstraint {
                            trait_name,
                            span: self.current_span(ti),
                        });
                    }
                    t => return Err(self.err(format!("期望trait约束名，但得到 {:?}", t), ti)),
                }

                if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                    ti.next();
                } else {
                    break;
                }
            }
        }

        match self.expect_token(ti, "{")? {
            Token::LeftBrace => {}
            t => return Err(self.err(format!("期望 {{，但得到 {:?}", t), ti)),
        }

        let mut operations = Vec::new();

        while let Some(token_result) = ti.peek() {
            match token_result {
                Ok((Token::RightBrace, _)) => {
                    ti.next();
                    break;
                }
                Ok((Token::Comma, _)) => {
                    ti.next();
                    continue;
                }
                Ok((Token::Ident(op_name), _)) => {
                    let op_name = op_name.clone();
                    ti.next();
                    match self.expect_token(ti, ":")? {
                        Token::Colon => {}
                        t => return Err(self.err(format!("期望 :，但得到 {:?}", t), ti)),
                    }
                    // 操作签名: Input -> Output 或 Input 或 -> Output
                    let input = if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
                        None
                    } else {
                        Some(self.parse_type(ti)?)
                    };
                    let output = if matches!(ti.peek(), Some(Ok((Token::Arrow, _)))) {
                        ti.next();
                        Some(self.parse_type(ti)?)
                    } else {
                        None
                    };
                    operations.push((op_name, input, output));
                    if matches!(ti.peek(), Some(Ok((Token::Comma, _)))) {
                        ti.next();
                    }
                }
                Ok(_) => {
                    return Err(self.err("期望操作声明或 }", ti));
                }
                Err(e) => return Err(self.err(e.to_string(), ti)),
            }
        }

        Ok(EffectDecl {
            name,
            type_parameters,
            operations,
            where_clause,
            span: self.current_span(ti),
        })
    }
}
