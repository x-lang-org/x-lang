// 词法分析器库

pub mod errors;
pub mod span;
pub mod token;

use errors::LexError;
use span::Span;
use token::Token;

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
    /// 错误恢复模式：跳过无效字符而不是返回错误
    pub recovery_mode: bool,
    /// 收集的无效字符位置（用于错误报告）
    pub skipped_positions: Vec<(usize, char)>,
}

impl<'a> Lexer<'a> {
    /// 创建新的词法分析器
    pub fn new(input: &'a str) -> Self {
        // 处理 UTF-8 BOM (0xEF 0xBB 0xBF)
        let input = if input.len() >= 3 {
            let bytes = input.as_bytes();
            if bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
                &input[3..]
            } else {
                input
            }
        } else {
            input
        };

        Self {
            input,
            chars: input.chars().peekable(),
            position: 0,
            state: LexerState::Normal,
            recovery_mode: false,
            skipped_positions: Vec::new(),
        }
    }

    /// 获取当前位置的字符
    pub fn current_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// 启用错误恢复模式：跳过无效字符而不是返回错误
    pub fn enable_recovery_mode(&mut self) {
        self.recovery_mode = true;
    }

    /// 检查是否是恢复模式
    pub fn is_recovery_mode(&self) -> bool {
        self.recovery_mode
    }

    /// 获取跳过的无效字符位置列表
    pub fn get_skipped_positions(&self) -> &[(usize, char)] {
        &self.skipped_positions
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
            match self.current_char() {
                Some('/') => {
                    let next = self.chars.clone().nth(1);
                    if next == Some('*') {
                        self.next_char();
                        self.next_char();
                        if let Some(ch) = self.current_char() {
                            if ch == '*' {
                                self.next_char();
                            }
                        }
                        depth += 1;
                    } else {
                        self.next_char();
                    }
                }
                Some('*') => {
                    let next = self.chars.clone().nth(1);
                    if next == Some('/') {
                        self.next_char();
                        self.next_char();
                        depth -= 1;
                    } else {
                        self.next_char();
                    }
                }
                Some(_) => {
                    self.next_char();
                }
                None => break,
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
            "mut" | "mutable" => Ok(Token::Mut),
            "val" => Ok(Token::Val),
            "var" => Ok(Token::Var),
            "const" => Ok(Token::Const),
            "function" => Ok(Token::Function),
            "async" => Ok(Token::Async),
            "class" => Ok(Token::Class),
            "struct" => Ok(Token::Struct),
            "enum" => Ok(Token::Enum),
            "extends" => Ok(Token::Extends),
            "trait" => Ok(Token::Trait),
            "interface" => Ok(Token::Interface),
            "implement" => Ok(Token::Implement),
            "implements" => Ok(Token::Implements),
            "abstract" => Ok(Token::Abstract),
            "super" => Ok(Token::Super),
            "type" => Ok(Token::Type),
            "new" => Ok(Token::New),
            "virtual" => Ok(Token::Virtual),
            "override" => Ok(Token::Override),
            "final" => Ok(Token::Final),
            "static" => Ok(Token::Static),
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
            "match" => Ok(Token::Match),
            "when" => Ok(Token::When),
            "is" => Ok(Token::Is),
            "where" => Ok(Token::Where),
            "and" => Ok(Token::And),
            "or" => Ok(Token::Or),
            "not" => Ok(Token::Not),
            "true" => Ok(Token::True),
            "false" => Ok(Token::False),
            "null" => Ok(Token::Null),
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
            "handle" => Ok(Token::Handle),
            "extern" => Ok(Token::Extern),
            "foreign" => Ok(Token::Foreign),
            "unsafe" => Ok(Token::Unsafe),
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
                    if next == Some('>') {
                        self.next_char();
                        return Ok(Token::FatArrow);
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
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::PlusEquals);
                    }
                    Ok(Token::Plus)
                }
                '-' => {
                    if next == Some('>') {
                        self.next_char();
                        return Ok(Token::Arrow);
                    }
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::MinusEquals);
                    }
                    Ok(Token::Minus)
                }
                '*' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::AsteriskEquals);
                    }
                    Ok(Token::Asterisk)
                }
                '/' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::SlashEquals);
                    }
                    Ok(Token::Slash)
                }
                '%' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::PercentEquals);
                    }
                    Ok(Token::Percent)
                }
                '^' => {
                    if next == Some('=') {
                        self.next_char();
                        return Ok(Token::CaretEquals);
                    }
                    Ok(Token::Caret)
                }
                ':' => {
                    if next == Some(':') {
                        self.next_char();
                        return Ok(Token::DoubleColon);
                    }
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
                    if next == Some('|') {
                        self.next_char();
                        return Ok(Token::OrOr);
                    }
                    if next == Some('>') {
                        self.next_char();
                        return Ok(Token::Pipe);
                    }
                    Ok(Token::VerticalBar)
                }
                '&' => {
                    if next == Some('&') {
                        self.next_char();
                        return Ok(Token::AndAnd);
                    }
                    Ok(Token::Ampersand)
                }
                '~' => Ok(Token::Tilde),
                '?' => Ok(Token::QuestionMark),
                '@' => Ok(Token::AtSign),
                '#' => Ok(Token::Hash),
                _ => Err(LexError::InvalidToken(ch, self.position - 1)),
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

                    // 检查 shebang (#!) - 只在文件开头检查一次
                    // 使用一个简单的检查：当前位置是否为0并且输入以 #! 开头
                    if self.position == 0 {
                        let a = self.input.chars().next();
                        let mut chars = self.input.chars();
                        let b = chars.nth(1);
                        if a == Some('#') && b == Some('!') {
                            // 跳过整行直到换行符
                            while let Some(ch) = self.current_char() {
                                if ch == '\n' {
                                    self.next_char();
                                    break;
                                }
                                self.next_char();
                            }
                            self.skip_whitespace();
                        }
                    }

                    let current = self.current_char();
                    match current {
                        Some('/') => {
                            let original_pos = self.position;
                            if self.skip_line_comment() {
                                continue;
                            }
                            if self.skip_block_comment() {
                                continue;
                            }
                            // 处理 '/' 作为操作符的情况
                            self.next_char();
                            let end = self.position;
                            return Ok((Token::Slash, Span::new(original_pos, end)));
                        }
                        Some(ch) if ch.is_alphabetic() || ch == '_' => {
                            let start = self.position;
                            let result = self.parse_identifier();
                            let end = self.position;
                            return result.map(|t| (t, Span::new(start, end)));
                        }
                        Some(ch) if ch.is_ascii_digit() => {
                            let start = self.position;
                            let result = self.parse_number();
                            let end = self.position;
                            return result.map(|t| (t, Span::new(start, end)));
                        }
                        Some('"') => {
                            let start = self.position;
                            let result = self.parse_string();
                            let end = self.position;
                            return result.map(|t| (t, Span::new(start, end)));
                        }
                        Some('\'') => {
                            let start = self.position;
                            let result = self.parse_char();
                            let end = self.position;
                            return result.map(|t| (t, Span::new(start, end)));
                        }
                        Some(ch) if "~!@#$%^&*()_+{}[]|;:,.<>?\\-=".contains(ch) => {
                            let start = self.position;
                            let result = self.parse_operator();
                            let end = self.position;
                            return result.map(|t| (t, Span::new(start, end)));
                        }
                        Some(_ch) => {
                            let start = self.position;
                            let ch = _ch;
                            self.next_char();
                            let end = self.position;
                            // 错误恢复模式：跳过无效字符并记录位置
                            if self.recovery_mode {
                                self.skipped_positions.push((start, ch));
                                continue; // 跳过这个字符，继续解析
                            }
                            return Err(LexError::InvalidToken(ch, start));
                        }
                        None => {
                            let start = self.position;
                            return Ok((Token::Eof, Span::new(start, start)));
                        }
                    }
                }

                LexerState::String | LexerState::MultilineString => {
                    // 这是一个内部错误状态 - 正常情况下不应该发生
                    return Err(LexError::InvalidToken(' ', self.position));
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
        // 二/八/十六进制前缀 0b / 0o / 0x
        let first = self.current_char();
        if first == Some('0') {
            let second = self.chars.clone().nth(1);
            match second {
                Some('x') | Some('X') => {
                    self.next_char();
                    self.next_char();
                    let mut num_str = String::new();
                    while let Some(ch) = self.current_char() {
                        if ch.is_ascii_hexdigit() || ch == '_' {
                            num_str.push(ch);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    if num_str.is_empty() || num_str.chars().all(|c| c == '_') {
                        return Err(LexError::InvalidNumber(self.position - 2, "0x".to_string()));
                    }
                    return Ok(Token::HexInt(num_str));
                }
                Some('o') | Some('O') => {
                    self.next_char();
                    self.next_char();
                    let mut num_str = String::new();
                    while let Some(ch) = self.current_char() {
                        if matches!(ch, '0'..='7') || ch == '_' {
                            num_str.push(ch);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    if num_str.is_empty() || num_str.chars().all(|c| c == '_') {
                        return Err(LexError::InvalidNumber(self.position - 2, "0o".to_string()));
                    }
                    return Ok(Token::OctInt(num_str));
                }
                Some('b') | Some('B') => {
                    self.next_char();
                    self.next_char();
                    let mut num_str = String::new();
                    while let Some(ch) = self.current_char() {
                        if ch == '0' || ch == '1' || ch == '_' {
                            num_str.push(ch);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    if num_str.is_empty() || num_str.chars().all(|c| c == '_') {
                        return Err(LexError::InvalidNumber(self.position - 2, "0b".to_string()));
                    }
                    return Ok(Token::BinInt(num_str));
                }
                _ => {}
            }
        }

        let mut num_str = String::new();

        // 解析整数部分
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '_' {
                num_str.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        // 检查是否有小数点
        if let Some('.') = self.current_char() {
            // 检查下一个字符是否也是点（范围表达式）
            let next = self.chars.clone().nth(1);
            if next == Some('.') {
                // 这是范围表达式的开始，返回整数
                return Ok(Token::DecimalInt(num_str));
            }

            // 这是浮点数的开始
            num_str.push('.');
            self.next_char();

            // 解析小数部分
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_digit() || ch == '_' {
                    num_str.push(ch);
                    self.next_char();
                } else {
                    break;
                }
            }

            // 检查是否有指数部分
            if let Some(ch) = self.current_char() {
                if ch == 'e' || ch == 'E' {
                    num_str.push(ch);
                    self.next_char();

                    // 解析指数符号
                    if let Some(ch) = self.current_char() {
                        if ch == '+' || ch == '-' {
                            num_str.push(ch);
                            self.next_char();
                        }
                    }

                    // 解析指数部分
                    while let Some(ch) = self.current_char() {
                        if ch.is_ascii_digit() || ch == '_' {
                            num_str.push(ch);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                }
            }

            return Ok(Token::Float(num_str));
        }

        // 检查是否有指数部分（整数形式的科学计数法）
        if let Some(ch) = self.current_char() {
            if ch == 'e' || ch == 'E' {
                num_str.push(ch);
                self.next_char();

                // 解析指数符号
                if let Some(ch) = self.current_char() {
                    if ch == '+' || ch == '-' {
                        num_str.push(ch);
                        self.next_char();
                    }
                }

                // 解析指数部分
                while let Some(ch) = self.current_char() {
                    if ch.is_ascii_digit() || ch == '_' {
                        num_str.push(ch);
                        self.next_char();
                    } else {
                        break;
                    }
                }

                return Ok(Token::Float(num_str));
            }
        }

        // 这是整数
        Ok(Token::DecimalInt(num_str))
    }

    /// 解析字符串
    fn parse_string(&mut self) -> Result<Token, LexError> {
        self.next_char(); // 跳过第一个 "

        // 检查是否是三引号多行字符串
        if self.current_char() == Some('"') {
            self.next_char();
            if self.current_char() == Some('"') {
                self.next_char();
                // 进入多行字符串模式
                self.state = LexerState::MultilineString;
                return self.parse_multiline_string();
            }
            // 空字符串 "" 或两个引号相邻的情况
            // 当前字符不是第三个引号，所以这是一个空字符串
            return Ok(Token::StringContent(String::new()));
        }

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
                        'n' => {
                            content.push('\n');
                            self.next_char();
                        }
                        't' => {
                            content.push('\t');
                            self.next_char();
                        }
                        'r' => {
                            content.push('\r');
                            self.next_char();
                        }
                        '"' => {
                            content.push('"');
                            self.next_char();
                        }
                        '\'' => {
                            content.push('\'');
                            self.next_char();
                        }
                        '\\' => {
                            content.push('\\');
                            self.next_char();
                        }
                        '0' => {
                            content.push('\0');
                            self.next_char();
                        }
                        'u' => {
                            // Unicode 转义: \u{...}
                            self.next_char();
                            if self.current_char() == Some('{') {
                                self.next_char();
                                let mut hex = String::new();
                                while let Some(h) = self.current_char() {
                                    if h.is_ascii_hexdigit() {
                                        hex.push(h);
                                        self.next_char();
                                    } else if h == '}' {
                                        self.next_char();
                                        break;
                                    } else {
                                        break;
                                    }
                                }
                                if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                                    if let Some(c) = char::from_u32(code_point) {
                                        content.push(c);
                                    }
                                }
                                // 已经消耗了 }，不需要再调用 next_char()
                            } else {
                                content.push('u');
                                self.next_char();
                            }
                        }
                        _ => {
                            content.push(escaped_ch);
                            self.next_char();
                        }
                    }
                }
            } else {
                content.push(ch);
                self.next_char();
            }
        }
        // 如果没有找到闭合的 "，则返回错误
        Err(LexError::UnclosedString)
    }

    /// 解析多行字符串内容
    fn parse_multiline_string(&mut self) -> Result<Token, LexError> {
        let mut content = String::new();
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                // 检查下两个字符是否也是 "
                let second = self.chars.clone().nth(1);
                let third = self.chars.clone().nth(2);
                if second == Some('"') && third == Some('"') {
                    // 跳过三个引号
                    self.next_char();
                    self.next_char();
                    self.next_char();
                    self.state = LexerState::Normal;
                    return Ok(Token::StringContent(content));
                }
                // 不是三引号，是普通字符
                content.push(ch);
                self.next_char();
            } else if ch == '\\' {
                // 处理转义字符
                self.next_char();
                if let Some(escaped_ch) = self.current_char() {
                    match escaped_ch {
                        'n' => {
                            content.push('\n');
                            self.next_char();
                        }
                        't' => {
                            content.push('\t');
                            self.next_char();
                        }
                        'r' => {
                            content.push('\r');
                            self.next_char();
                        }
                        '"' => {
                            content.push('"');
                            self.next_char();
                        }
                        '\'' => {
                            content.push('\'');
                            self.next_char();
                        }
                        '\\' => {
                            content.push('\\');
                            self.next_char();
                        }
                        '0' => {
                            content.push('\0');
                            self.next_char();
                        }
                        'u' => {
                            // Unicode 转义: \u{...}
                            self.next_char();
                            if self.current_char() == Some('{') {
                                self.next_char();
                                let mut hex = String::new();
                                while let Some(h) = self.current_char() {
                                    if h.is_ascii_hexdigit() {
                                        hex.push(h);
                                        self.next_char();
                                    } else if h == '}' {
                                        self.next_char();
                                        break;
                                    } else {
                                        break;
                                    }
                                }
                                if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                                    if let Some(c) = char::from_u32(code_point) {
                                        content.push(c);
                                    }
                                }
                                // 已经消耗了 }，不需要再调用 next_char()
                            } else {
                                content.push('u');
                                self.next_char();
                            }
                        }
                        _ => {
                            content.push(escaped_ch);
                            self.next_char();
                        }
                    }
                }
            } else {
                content.push(ch);
                self.next_char();
            }
        }
        // 多行字符串未闭合
        self.state = LexerState::Normal;
        Err(LexError::UnclosedString)
    }

    /// 解析字符串内容
    #[allow(dead_code)]
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
                                '\'' => content.push('\''),
                                '\\' => content.push('\\'),
                                '0' => content.push('\0'),
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

    /// 解析字符字面量：'x' 或 '\n' 等，返回 CharContent(s)。
    fn parse_char(&mut self) -> Result<Token, LexError> {
        self.next_char(); // 跳过开头的 '
        let ch = self.current_char();
        if ch.is_none() || ch == Some('\n') {
            return Err(LexError::UnclosedChar);
        }
        let content = if ch == Some('\\') {
            self.next_char(); // 消费反斜杠
            let escaped = self.current_char();
            if escaped.is_none() {
                return Err(LexError::UnclosedChar);
            }
            self.next_char();
            match escaped {
                Some('n') => '\n',
                Some('t') => '\t',
                Some('r') => '\r',
                Some('\'') => '\'',
                Some('\\') => '\\',
                Some('0') => '\0',
                Some('u') => {
                    // Unicode 转义: \u{...}
                    if self.current_char() == Some('{') {
                        self.next_char();
                        let mut hex = String::new();
                        while let Some(h) = self.current_char() {
                            if h.is_ascii_hexdigit() {
                                hex.push(h);
                                self.next_char();
                            } else if h == '}' {
                                self.next_char();
                                break;
                            } else {
                                break;
                            }
                        }
                        if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                            if let Some(c) = char::from_u32(code_point) {
                                return Ok(Token::CharContent(c.to_string()));
                            }
                        }
                        return Err(LexError::InvalidToken('u', self.position));
                    }
                    'u'
                }
                Some(c) => c,
                None => unreachable!(),
            }
        } else {
            let c = ch.unwrap();
            if c == '\'' {
                return Err(LexError::UnclosedChar); // 空字符字面量
            }
            self.next_char();
            c
        };
        if self.current_char() != Some('\'') {
            return Err(LexError::UnclosedChar);
        }
        self.next_char(); // 消费闭合的 '
        Ok(Token::CharContent(content.to_string()))
    }

    /// 解析字符内容（用于 LexerState::Char 状态，多行/复杂流程预留）
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

    /// 启用错误恢复模式：跳过无效字符而不是返回错误
    pub fn enable_recovery_mode(&mut self) {
        self.lexer.enable_recovery_mode();
    }

    /// 检查是否是恢复模式
    pub fn is_recovery_mode(&self) -> bool {
        self.lexer.is_recovery_mode()
    }

    /// 获取跳过的无效字符位置列表
    pub fn get_skipped_positions(&self) -> &[(usize, char)] {
        self.lexer.get_skipped_positions()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_new() {
        let input = "let x = 42;";
        let lexer = Lexer::new(input);
        assert_eq!(lexer.input, input);
        assert_eq!(lexer.position, 0);
        assert_eq!(lexer.state, LexerState::Normal);
    }

    #[test]
    fn test_token_iterator_new() {
        let input = "let x = 42;";
        let mut iter = TokenIterator::new(input);
        assert!(iter.peek().is_some());
    }

    #[test]
    fn test_new_lexer() {
        let input = "let x = 42;";
        let mut iter = new_lexer(input);
        assert!(iter.next().is_some());
    }

    #[test]
    fn test_lex_keywords() {
        let input = "let mut val var const function async class trait";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 9);
        assert!(matches!(tokens[0], Token::Let));
        assert!(matches!(tokens[1], Token::Mut));
        assert!(matches!(tokens[2], Token::Val));
        assert!(matches!(tokens[3], Token::Var));
        assert!(matches!(tokens[4], Token::Const));
        assert!(matches!(tokens[5], Token::Function));
        assert!(matches!(tokens[6], Token::Async));
        assert!(matches!(tokens[7], Token::Class));
        assert!(matches!(tokens[8], Token::Trait));
    }

    #[test]
    fn test_lex_identifiers() {
        let input = "foo bar_baz my-var";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Ident(s) if s == "foo"));
        assert!(matches!(&tokens[1], Token::Ident(s) if s == "bar_baz"));
        assert!(matches!(&tokens[2], Token::Ident(s) if s == "my-var"));
    }

    #[test]
    fn test_lex_integers() {
        let input = "42 123 0";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::DecimalInt(s) if s == "42"));
        assert!(matches!(&tokens[1], Token::DecimalInt(s) if s == "123"));
        assert!(matches!(&tokens[2], Token::DecimalInt(s) if s == "0"));
    }

    #[test]
    fn test_lex_floats() {
        let input = "3.14 0.5 2e10 1.5e-3";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0], Token::Float(s) if s == "3.14"));
        assert!(matches!(&tokens[1], Token::Float(s) if s == "0.5"));
        assert!(matches!(&tokens[2], Token::Float(s) if s == "2e10"));
        assert!(matches!(&tokens[3], Token::Float(s) if s == "1.5e-3"));
    }

    #[test]
    fn test_lex_operators() {
        let input = "+ - * / = == != < <= > >= && ||";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 13);
        assert!(matches!(tokens[0], Token::Plus));
        assert!(matches!(tokens[1], Token::Minus));
        assert!(matches!(tokens[2], Token::Asterisk));
        assert!(matches!(tokens[3], Token::Slash));
        assert!(matches!(tokens[4], Token::Equals));
        assert!(matches!(tokens[5], Token::DoubleEquals));
        assert!(matches!(tokens[6], Token::NotEquals));
        assert!(matches!(tokens[7], Token::LessThan));
        assert!(matches!(tokens[8], Token::LessThanEquals));
        assert!(matches!(tokens[9], Token::GreaterThan));
        assert!(matches!(tokens[10], Token::GreaterThanEquals));
        assert!(matches!(tokens[11], Token::AndAnd));
        assert!(matches!(tokens[12], Token::OrOr));
    }

    #[test]
    fn test_lex_punctuation() {
        let input = "( ) { } [ ] , . : ;";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 10);
        assert!(matches!(tokens[0], Token::LeftParen));
        assert!(matches!(tokens[1], Token::RightParen));
        assert!(matches!(tokens[2], Token::LeftBrace));
        assert!(matches!(tokens[3], Token::RightBrace));
        assert!(matches!(tokens[4], Token::LeftBracket));
        assert!(matches!(tokens[5], Token::RightBracket));
        assert!(matches!(tokens[6], Token::Comma));
        assert!(matches!(tokens[7], Token::Dot));
        assert!(matches!(tokens[8], Token::Colon));
        assert!(matches!(tokens[9], Token::Semicolon));
    }

    #[test]
    fn test_lex_boolean_literals() {
        let input = "true false";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::True));
        assert!(matches!(tokens[1], Token::False));
    }

    #[test]
    fn test_lex_null_literal() {
        let input = "null";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Null));
    }

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 10);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
    }

    #[test]
    fn test_span_line_col() {
        let source = "line1\nline2";
        let span = Span::new(6, 11);
        let (line, col) = span.line_col(source);
        assert_eq!(line, 2);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_span_snippet() {
        let source = "hello world";
        let span = Span::new(0, 5);
        assert_eq!(span.snippet(source), "hello");
    }

    #[test]
    fn test_lex_empty_input() {
        let input = "";
        let mut iter = new_lexer(input);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_lex_whitespace() {
        let input = "   \n\t  let   x   =   42   ;   ";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Let));
        assert!(matches!(&tokens[1], Token::Ident(s) if s == "x"));
        assert!(matches!(tokens[2], Token::Equals));
        assert!(matches!(&tokens[3], Token::DecimalInt(s) if s == "42"));
        assert!(matches!(tokens[4], Token::Semicolon));
    }

    #[test]
    fn test_lex_line_comment() {
        let input = "let x = 42; // this is a comment";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_lex_block_comment() {
        let input = "/** block comment */ let x = 42;";
        let mut iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
    }

    // ----- 字符串字面量 -----
    #[test]
    fn test_lex_string_content() {
        let input = r#" "a" "#;
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == "a"));
    }

    #[test]
    fn test_lex_string_escapes() {
        let input = r#" "\n\t\r\"\\" "#;
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == "\n\t\r\"\\"));
    }

    #[test]
    fn test_lex_string_unclosed() {
        let input = r#" "abc"#;
        let mut iter = new_lexer(input);
        let first = iter.next();
        assert!(matches!(first, Some(Err(LexError::UnclosedString))));
    }

    // ----- 字符字面量 -----
    #[test]
    fn test_lex_char_content() {
        let input = " 'a' ";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::CharContent(s) if s == "a"));
    }

    #[test]
    fn test_lex_char_escapes() {
        let input = r#" '\n' '\'' '\\' '\0' "#;
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0], Token::CharContent(s) if s == "\n"));
        assert!(matches!(&tokens[1], Token::CharContent(s) if s == "'"));
        assert!(matches!(&tokens[2], Token::CharContent(s) if s == "\\"));
        assert!(matches!(&tokens[3], Token::CharContent(s) if s == "\0"));
    }

    #[test]
    fn test_lex_char_unclosed() {
        let input = " 'a";
        let mut iter = new_lexer(input);
        let first = iter.next();
        assert!(matches!(first, Some(Err(LexError::UnclosedChar))));
    }

    // ----- 数字：十六进制 / 八进制 / 二进制 -----
    #[test]
    fn test_lex_hex_oct_bin() {
        let input = "0x1a 0o17 0b101 0x1A_b 0Xff";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0], Token::HexInt(s) if s == "1a"));
        assert!(matches!(&tokens[1], Token::OctInt(s) if s == "17"));
        assert!(matches!(&tokens[2], Token::BinInt(s) if s == "101"));
        assert!(matches!(&tokens[3], Token::HexInt(s) if s == "1A_b"));
        assert!(matches!(&tokens[4], Token::HexInt(s) if s == "ff"));
    }

    #[test]
    fn test_lex_invalid_number() {
        let input = "0x 0b 0o";
        let mut iter = new_lexer(input);
        let a = iter.next().unwrap();
        let b = iter.next().unwrap();
        let c = iter.next().unwrap();
        assert!(matches!(a, Err(LexError::InvalidNumber(_, _))));
        assert!(matches!(b, Err(LexError::InvalidNumber(_, _))));
        assert!(matches!(c, Err(LexError::InvalidNumber(_, _))));
    }

    // ----- 边界：peek 与 last_span -----
    #[test]
    fn test_peek_then_next() {
        let input = "let x";
        let mut iter = new_lexer(input);
        let p1 = iter.peek().cloned();
        let n1 = iter.next();
        let _n2 = iter.next();
        assert!(p1.as_ref().and_then(|r| r.as_ref().ok()).is_some());
        assert_eq!(p1, n1);
        assert!(iter.last_span.is_some());
    }

    #[test]
    fn test_lex_number_with_underscore() {
        let input = "1_000_000 0x1_a";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0], Token::DecimalInt(s) if s == "1_000_000"));
        assert!(matches!(&tokens[1], Token::HexInt(s) if s == "1_a"));
    }

    // ----- 多行字符串 -----
    #[test]
    fn test_multiline_string() {
        let input = "\"\"\"\nmultiline\nstring\ncontent\n\"\"\"";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == "\nmultiline\nstring\ncontent\n"));
    }

    #[test]
    fn test_multiline_string_empty() {
        // 6 quotes: 3 to open, 3 to close
        let input = "\"\"\"\"\"\"";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == ""));
    }

    #[test]
    fn test_multiline_string_with_quotes() {
        // Input: """hello """test""""""
        // Which is: """ + hello " + """ + test + """ + """
        // That's complex, let's simplify
        let input = "\"\"\"hello \"\"world\"\" test\"\"\"";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == "hello \"\"world\"\" test"));
    }

    #[test]
    fn test_multiline_string_unclosed() {
        let input = "\"\"\"hello ";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        // 第一个 """ 是多行字符串开始但未闭合，会返回错误
        assert!(tokens.is_empty());
    }

    // ----- Unicode 转义 -----
    #[test]
    fn test_string_unicode_escape() {
        let input = r#""\u{41}\u{42}\u{43}""#;
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == "ABC"));
    }

    #[test]
    fn test_string_unicode_escape_chinese() {
        let input = r#""\u{4E2D}\u{6587}""#;
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s == "中文"));
    }

    #[test]
    fn test_char_unicode_escape() {
        let input = " '\u{41}' ";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::CharContent(s) if s == "A"));
    }

    // ----- Shebang -----
    #[test]
    fn test_shebang() {
        let input = "#!/usr/bin/env x\nlet x = 42;";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Let));
    }

    #[test]
    fn test_shebang_no_newline() {
        let input = "#!/usr/bin/env x";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        // 没有换行符的 shebang 只返回 EOF
        assert_eq!(tokens.len(), 0);
    }

    // ----- BOM -----
    #[test]
    fn test_bom() {
        // UTF-8 BOM: 0xEF 0xBB 0xBF
        let bom = [0xEF, 0xBB, 0xBF];
        let input = std::str::from_utf8(&bom).unwrap();
        let input = format!("{}let x = 42;", input);
        let iter = new_lexer(&input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Let));
    }

    #[test]
    fn test_bom_with_shebang() {
        // BOM + shebang
        let bom = [0xEF, 0xBB, 0xBF];
        let input = std::str::from_utf8(&bom).unwrap();
        let input = format!("{}#!/usr/bin/env x\nlet x = 42;", input);
        let iter = new_lexer(&input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Let));
    }

    // ----- 错误恢复模式 -----
    #[test]
    fn test_error_recovery_mode() {
        // 恢复模式下跳过无效字符继续解析
        // 使用一个肯定无效的字符（控制字符 0x01）
        let input = "let \x01x=42;";
        let mut iter = new_lexer(input);
        iter.enable_recovery_mode();
        let tokens: Vec<_> = iter.by_ref().filter_map(Result::ok).map(|(t, _)| t).collect();
        // 应该成功解析出 let, x, =, 42, ;
        assert!(matches!(&tokens[0], Token::Let));

        // 检查有跳过的位置
        let skipped = iter.get_skipped_positions();
        assert!(!skipped.is_empty());
    }

    #[test]
    fn test_error_recovery_multiple_invalid_chars() {
        // 恢复模式下收集多个无效字符
        let input = "a \x01 \x02 b";
        let mut iter = new_lexer(input);
        iter.enable_recovery_mode();
        let tokens: Vec<_> = iter.by_ref().filter_map(Result::ok).map(|(t, _)| t).collect();
        // 应该有 a 和 b
        assert!(tokens.len() >= 2);

        let skipped = iter.get_skipped_positions();
        assert!(skipped.len() >= 2);
    }

    // ----- 插值 token -----
    #[test]
    fn test_interpolate_tokens_exist() {
        // 确保插值 token 类型存在
        let _ = Token::InterpolateStart;
        let _ = Token::InterpolateEnd;
    }

    // ----- 回归测试：边界条件 -----
    #[test]
    fn test_regression_empty_string_at_end() {
        let input = "let x=\"\"";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        // let, x, =
        assert!(tokens.len() >= 2);
    }

    #[test]
    fn test_regression_unicode_in_identifier() {
        // 后续可能支持 Unicode 标识符，当前应作为无效字符处理
        let input = "let\x01=42";
        let mut iter = new_lexer(input);
        iter.enable_recovery_mode();
        let tokens: Vec<_> = iter.by_ref().filter_map(Result::ok).map(|(t, _)| t).collect();
        // 在恢复模式下应跳过控制字符继续解析
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_regression_multiline_string_newline() {
        // 多行字符串可以包含换行
        let input = "\"\"\"\nhello\nworld\n\"\"\"";
        let iter = new_lexer(input);
        let tokens: Vec<_> = iter.filter_map(Result::ok).map(|(t, _)| t).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::StringContent(s) if s.contains("hello")));
    }
}
