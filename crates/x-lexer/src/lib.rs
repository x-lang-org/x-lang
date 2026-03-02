// 词法分析器库

pub mod token;
pub mod errors;
pub mod span;

use token::Token;
use errors::LexError;
use span::Span;

/// 词法分析器状态
#[derive(Debug, PartialEq, Clone)]
pub enum LexerState {
    Normal,
    String,
    MultilineString,
    Char,
}

/// 词法分析器
pub struct Lexer<'a> {
    pub input: &'a str,
    pub chars: std::iter::Peekable<std::str::Chars<'a>>,
    pub position: usize,
    pub state: LexerState,
}

impl<'a> Lexer<'a> {
    /// 创建新的词法分析器
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            position: 0,
            state: LexerState::Normal,
        }
    }

    /// 获取当前位置的字符
    pub fn current_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// 向前移动一个字符（position 为字节偏移，便于与源码索引一致）
    fn next_char(&mut self) {
        if let Some(ch) = self.chars.next() {
            self.position += ch.len_utf8();
        }
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// 跳过单行注释 (//)，返回 true 如果跳过了注释
    fn skip_line_comment(&mut self) -> bool {
        let (a, b) = (self.current_char(), self.chars.clone().nth(1));
        if a == Some('/') && b == Some('/') {
            self.next_char();
            self.next_char();
            while let Some(ch) = self.current_char() {
                if ch == '\n' {
                    self.next_char();
                    break;
                }
                self.next_char();
            }
            true
        } else {
            false
        }
    }

    /// 跳过多行注释 /** ... */，返回 true 如果跳过了注释
    fn skip_block_comment(&mut self) -> bool {
        let a = self.current_char();
        let b = self.chars.clone().nth(1);
        let c = self.chars.clone().nth(2);
        if a != Some('/') || b != Some('*') || c != Some('*') {
            return false;
        }
        self.next_char();
        self.next_char();
        self.next_char();
        let mut depth = 1usize;
        while depth > 0 {
            match (self.current_char(), self.chars.clone().nth(1)) {
                (Some('/'), Some('*')) => {
                    self.next_char();
                    self.next_char();
                    if let Some(ch) = self.current_char() {
                        if ch == '*' {
                            self.next_char();
                        }
                    }
                    depth += 1;
                }
                (Some('*'), Some('/')) => {
                    self.next_char();
                    self.next_char();
                    depth -= 1;
                }
                (Some(_), _) => {
                    self.next_char();
                }
                (None, _) => break,
            }
        }
        true
    }

    /// 解析标识符
    fn parse_identifier(&mut self) -> Result<Token, LexError> {
        let mut ident = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                ident.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "let" => Ok(Token::Let),
            "mut" => Ok(Token::Mut),
            "val" => Ok(Token::Val),
            "var" => Ok(Token::Var),
            "fun" => Ok(Token::Fun),
            "async" => Ok(Token::Async),
            "class" => Ok(Token::Class),
            "extends" => Ok(Token::Extends),
            "trait" => Ok(Token::Trait),
            "type" => Ok(Token::Type),
            "new" => Ok(Token::New),
            "virtual" => Ok(Token::Virtual),
            "override" => Ok(Token::Override),
            "final" => Ok(Token::Final),
            "private" => Ok(Token::Private),
            "public" => Ok(Token::Public),
            "protected" => Ok(Token::Protected),
            "module" => Ok(Token::Module),
            "internal" => Ok(Token::Internal),
            "import" => Ok(Token::Import),
            "export" => Ok(Token::Export),
            "return" => Ok(Token::Return),
            "if" => Ok(Token::If),
            "else" => Ok(Token::Else),
            "for" => Ok(Token::For),
            "in" => Ok(Token::In),
            "while" => Ok(Token::While),
            "when" => Ok(Token::When),
            "is" => Ok(Token::Is),
            "where" => Ok(Token::Where),
            "and" => Ok(Token::And),
            "or" => Ok(Token::Or),
            "not" => Ok(Token::Not),
            "true" => Ok(Token::True),
            "false" => Ok(Token::False),
            "null" => Ok(Token::Null),
            "none" => Ok(Token::NoneKeyword),
            "some" => Ok(Token::Some),
            "ok" => Ok(Token::Ok),
            "err" => Ok(Token::Err),
            "needs" => Ok(Token::Needs),
            "given" => Ok(Token::Given),
            "wait" => Ok(Token::Wait),
            "together" => Ok(Token::Together),
            "race" => Ok(Token::Race),
            "timeout" => Ok(Token::Timeout),
            "atomic" => Ok(Token::Atomic),
            "retry" => Ok(Token::Retry),
            "use" => Ok(Token::Use),
            "with" => Ok(Token::With),
            "throws" => Ok(Token::Throws),
            "try" => Ok(Token::Try),
            "catch" => Ok(Token::Catch),
            "finally" => Ok(Token::Finally),
            "throw" => Ok(Token::Throw),
            _ => Ok(Token::Ident(ident)),
        }
    }

    /// 解析操作符
    fn parse_operator(&mut self) -> Result<Token, LexError> {
        if let Some(ch) = self.current_char() {
            self.next_char();
            let next = self.current_char();

            match ch {
                '=' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::DoubleEquals);
                    }
                    Ok(Token::Equals)
                }
                '!' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::NotEquals);
                    }
                    Ok(Token::NotOperator)
                }
                '<' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::LessThanEquals);
                    }
                    Ok(Token::LessThan)
                }
                '>' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::GreaterThanEquals);
                    }
                    Ok(Token::GreaterThan)
                }
                '+' => {
                    if next == Some('=') { self.next_char(); return Ok(Token::PlusEquals); }
                    Ok(Token::Plus)
                }
                '-' => {
                    if next == Some('>') { self.next_char(); return Ok(Token::Arrow); }
                    if next == Some('=') { self.next_char(); return Ok(Token::MinusEquals); }
                    Ok(Token::Minus)
                }
                '*' => {
                    if next == Some('=') { self.next_char(); return Ok(Token::AsteriskEquals); }
                    Ok(Token::Asterisk)
                }
                '/' => {
                    if next == Some('=') { self.next_char(); return Ok(Token::SlashEquals); }
                    Ok(Token::Slash)
                }
                '%' => {
                    if next == Some('=') { self.next_char(); return Ok(Token::PercentEquals); }
                    Ok(Token::Percent)
                }
                '^' => {
                    if next == Some('=') { self.next_char(); return Ok(Token::CaretEquals); }
                    Ok(Token::Caret)
                }
                ':' => {
                    if next == Some(':') { self.next_char(); return Ok(Token::DoubleColon); }
                    Ok(Token::Colon)
                }
                '.' => {
                    if next == Some('.') {
                        self.next_char();
                        if self.current_char() == Some('=') {
                            self.next_char();
                            return Ok(Token::RangeInclusive);
                        }
                        return Ok(Token::RangeExclusive);
                    }
                    Ok(Token::Dot)
                }
                ',' => Ok(Token::Comma),
                ';' => Ok(Token::Semicolon),
                '(' => Ok(Token::LeftParen),
                ')' => Ok(Token::RightParen),
                '{' => Ok(Token::LeftBrace),
                '}' => Ok(Token::RightBrace),
                '[' => Ok(Token::LeftBracket),
                ']' => Ok(Token::RightBracket),
                '|' => {
                    if next == Some('|') { self.next_char(); return Ok(Token::OrOr); }
                    if next == Some('>') { self.next_char(); return Ok(Token::Pipe); }
                    Ok(Token::VerticalBar)
                }
                '&' => {
                    if next == Some('&') { self.next_char(); return Ok(Token::AndAnd); }
                    Ok(Token::Ampersand)
                }
                '~' => Ok(Token::Tilde),
                '?' => Ok(Token::QuestionMark),
                '@' => Ok(Token::AtSign),
                '#' => Ok(Token::Hash),
                _ => Err(LexError::InvalidToken),
            }
        } else {
            Ok(Token::Eof)
        }
    }

    /// 获取下一个标记及其在源码中的 Span
    pub fn next_token(&mut self) -> Result<(Token, Span), LexError> {
        loop {
            match self.state {
                LexerState::Normal => {
                    self.skip_whitespace();

                    match self.current_char() {
                        Some('/') => {
                            if self.skip_line_comment() {
                                continue;
                            }
                            if self.skip_block_comment() {
                                continue;
                            }
                        }
                        _ => {}
                    }

                    let start = self.position;
                    let result = match self.current_char() {
                        Some('/') => {
                            self.next_char();
                            Ok(Token::Slash)
                        }
                        Some(ch) if ch.is_alphabetic() || ch == '_' => self.parse_identifier(),
                        Some(ch) if ch.is_ascii_digit() => self.parse_number(),
                        Some('"') => self.parse_string(),
                        Some('\'') => self.parse_char(),
                        Some(ch) if "~!@#$%^&*()_+{}[]|;:,.<>?/\\-=".contains(ch) => {
                            self.parse_operator()
                        }
                        Some(_ch) => {
                            self.next_char();
                            Err(LexError::InvalidToken)
                        }
                        None => Ok(Token::Eof),
                    };
                    let end = self.position;
                    return result.map(|t| (t, Span::new(start, end)));
                }

                LexerState::String | LexerState::MultilineString => {
                    return Err(LexError::InvalidToken);
                }

                LexerState::Char => {
                    let start = self.position;
                    let result = self.parse_char_content();
                    let end = self.position;
                    return result.map(|t| (t, Span::new(start, end)));
                }
            }
        }
    }

    /// 解析数字
    fn parse_number(&mut self) -> Result<Token, LexError> {
        let mut num_str = String::new();

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '_' || ch == '.' || ch == 'e' || ch == 'E' {
                num_str.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
            Ok(Token::Float(num_str))
        } else {
            Ok(Token::DecimalInt(num_str))
        }
    }

    /// 解析字符串
    fn parse_string(&mut self) -> Result<Token, LexError> {
        self.next_char(); // 跳过第一个 "
        // Check for """ (multiline string): current is " AND the char after it is also "
        if self.current_char() == Some('"') && self.input.as_bytes().get(self.position + 1) == Some(&b'"') {
            self.next_char(); // 跳过第二个 "
            self.next_char(); // 跳过第三个 "
            self.state = LexerState::MultilineString;
            Ok(Token::MultilineStringQuote)
        } else {
            // 解析单行字符串
            let mut content = String::new();
            while let Some(ch) = self.current_char() {
                if ch == '"' {
                    self.next_char(); // 跳过闭合的 "
                    return Ok(Token::StringContent(content));
                } else if ch == '\\' {
                    // 处理转义字符
                    self.next_char();
                    if let Some(escaped_ch) = self.current_char() {
                        match escaped_ch {
                            'n' => content.push('\n'),
                            't' => content.push('\t'),
                            'r' => content.push('\r'),
                            '"' => content.push('"'),
                            '\\' => content.push('\\'),
                            _ => content.push(escaped_ch),
                        }
                        self.next_char();
                    }
                } else {
                    content.push(ch);
                    self.next_char();
                }
            }
            // 如果没有找到闭合的 "，则返回错误
            Err(LexError::InvalidToken)
        }
    }

    /// 解析字符串内容
    fn parse_string_content(&mut self) -> Result<Token, LexError> {
        let mut content = String::new();

        while let Some(ch) = self.current_char() {
            match self.state {
                LexerState::String => {
                    if ch == '"' {
                        self.next_char();
                        self.state = LexerState::Normal;
                        return Ok(Token::StringQuote);
                    } else if ch == '\\' {
                        self.next_char();
                        if let Some(escaped_ch) = self.current_char() {
                            match escaped_ch {
                                'n' => content.push('\n'),
                                't' => content.push('\t'),
                                'r' => content.push('\r'),
                                '"' => content.push('"'),
                                '\\' => content.push('\\'),
                                _ => content.push(escaped_ch),
                            }
                            self.next_char();
                        }
                    } else {
                        content.push(ch);
                        self.next_char();
                    }
                }
                LexerState::MultilineString => {
                    if ch == '"' {
                        // 检查下两个字符是否也是 "
                        self.next_char();
                        if self.current_char() == Some('"') {
                            self.next_char();
                            if self.current_char() == Some('"') {
                                self.next_char();
                                self.state = LexerState::Normal;
                                return Ok(Token::MultilineStringQuote);
                            } else {
                                // 不是三个连续的 "，回退两个字符
                                content.push('"');
                                content.push('"');
                            }
                        } else {
                            // 不是三个连续的 "，回退一个字符
                            content.push('"');
                        }
                    } else {
                        content.push(ch);
                        self.next_char();
                    }
                }
                _ => break,
            }
        }

        Ok(Token::StringContent(content))
    }

    /// 解析字符
    fn parse_char(&mut self) -> Result<Token, LexError> {
        Ok(Token::CharQuote)
    }

    /// 解析字符内容
    fn parse_char_content(&mut self) -> Result<Token, LexError> {
        Ok(Token::CharContent("".to_string()))
    }
}

/// 词法分析器迭代器：产出带 Span 的 token，并保留 last_span 供解析错误使用。
pub struct TokenIterator<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Result<(Token, Span), LexError>>,
    /// 最近一次 next() 返回的 token 的 span，供 parser 报错时使用
    pub last_span: Option<Span>,
}

impl<'a> TokenIterator<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            peeked: None,
            last_span: None,
        }
    }

    /// 查看下一个 (token, span) 而不消耗；到达 EOF 返回 None
    pub fn peek(&mut self) -> Option<&Result<(Token, Span), LexError>> {
        if self.peeked.is_none() {
            self.peeked = match self.lexer.next_token() {
                Ok((Token::Eof, _)) => None,
                Ok(ok) => Some(Ok(ok)),
                Err(e) => Some(Err(e)),
            };
        }
        self.peeked.as_ref()
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<(Token, Span), LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = if let Some(peeked) = self.peeked.take() {
            Some(peeked)
        } else {
            match self.lexer.next_token() {
                Ok((Token::Eof, _)) => None,
                Ok(ok) => Some(Ok(ok)),
                Err(e) => Some(Err(e)),
            }
        };
        if let Some(Ok((_, span))) = item.as_ref() {
            self.last_span = Some(*span);
        }
        item
    }
}

/// 从字符串创建词法分析器
pub fn new_lexer(input: &str) -> TokenIterator<'_> {
    TokenIterator::new(input)
}
