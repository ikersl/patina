use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    StringLit(String),
    CharLit(char),
    True,
    False,

    // Identifiers & keywords
    Ident(String),
    Fn,
    Var,
    Mut,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Match,
    Class,
    Methods,
    Enum,
    Break,
    Continue,
    Self_,
    Trait,
    Use,
    Const,
    TypeKw,
    As,

    // Types
    IntType,
    FloatType,
    StringType,
    BoolType,
    CharType,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleStar,
    Eq,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Bang,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Arrow,
    FatArrow,
    Dot,
    DotDot,
    DotDotEq,
    ColonColon,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Underscore,
    Pipe,

    // Special
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::StringLit(s) => write!(f, "\"{s}\""),
            Token::CharLit(c) => write!(f, "'{c}'"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Newline => write!(f, "newline"),
            Token::Eof => write!(f, "end of file"),
            Token::Fn => write!(f, "'fn'"),
            Token::Var => write!(f, "'var'"),
            Token::Mut => write!(f, "'mut'"),
            Token::Return => write!(f, "'return'"),
            Token::If => write!(f, "'if'"),
            Token::Else => write!(f, "'else'"),
            Token::While => write!(f, "'while'"),
            Token::For => write!(f, "'for'"),
            Token::In => write!(f, "'in'"),
            Token::Match => write!(f, "'match'"),
            Token::Class => write!(f, "'class'"),
            Token::Methods => write!(f, "'methods'"),
            Token::Enum => write!(f, "'enum'"),
            Token::Trait => write!(f, "'trait'"),
            Token::Use => write!(f, "'use'"),
            Token::Const => write!(f, "'const'"),
            Token::TypeKw => write!(f, "'type'"),
            Token::As => write!(f, "'as'"),
            Token::Plus => write!(f, "'+'"),
            Token::Minus => write!(f, "'-'"),
            Token::Star => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Eq => write!(f, "'='"),
            Token::LParen => write!(f, "'('"),
            Token::RParen => write!(f, "')'"),
            Token::LBrace => write!(f, "'{{'"),
            Token::RBrace => write!(f, "'}}'"),
            Token::LBracket => write!(f, "'['"),
            Token::RBracket => write!(f, "']'"),
            Token::Comma => write!(f, "','"),
            Token::Colon => write!(f, "':'"),
            Token::Arrow => write!(f, "'->'"),
            Token::FatArrow => write!(f, "'=>'"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Spanned>, String> {
        let mut tokens = Vec::new();

        while self.pos < self.source.len() {
            let ch = self.source[self.pos];

            if ch == '#' {
                while self.pos < self.source.len() && self.source[self.pos] != '\n' {
                    self.advance();
                }
                continue;
            }

            if ch == '\n' {
                let line = self.line;
                let col = self.col;
                self.advance();
                if let Some(last) = tokens.last() {
                    let last: &Spanned = last;
                    match &last.token {
                        Token::Newline => {}
                        _ => tokens.push(Spanned {
                            token: Token::Newline,
                            line,
                            col,
                        }),
                    }
                }
                continue;
            }

            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
                continue;
            }

            let line = self.line;
            let col = self.col;

            let token = match ch {
                '+' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::PlusEq
                    } else {
                        Token::Plus
                    }
                }
                '-' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        Token::Arrow
                    } else if self.peek() == Some('=') {
                        self.advance();
                        Token::MinusEq
                    } else {
                        Token::Minus
                    }
                }
                '*' => {
                    self.advance();
                    if self.peek() == Some('*') {
                        self.advance();
                        Token::DoubleStar
                    } else if self.peek() == Some('=') {
                        self.advance();
                        Token::StarEq
                    } else {
                        Token::Star
                    }
                }
                '/' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::SlashEq
                    } else {
                        Token::Slash
                    }
                }
                '%' => {
                    self.advance();
                    Token::Percent
                }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::EqEq
                    } else if self.peek() == Some('>') {
                        self.advance();
                        Token::FatArrow
                    } else {
                        Token::Eq
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::BangEq
                    } else {
                        Token::Bang
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::LtEq
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::GtEq
                    } else {
                        Token::Gt
                    }
                }
                '&' => {
                    self.advance();
                    if self.peek() == Some('&') {
                        self.advance();
                        Token::And
                    } else {
                        return Err(format!(
                            "line {line}:{col}: unexpected '&', did you mean '&&'?"
                        ));
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek() == Some('|') {
                        self.advance();
                        Token::Or
                    } else {
                        Token::Pipe
                    }
                }
                '.' => {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Token::DotDotEq
                        } else {
                            Token::DotDot
                        }
                    } else {
                        Token::Dot
                    }
                }
                ':' => {
                    self.advance();
                    if self.peek() == Some(':') {
                        self.advance();
                        Token::ColonColon
                    } else {
                        Token::Colon
                    }
                }
                '(' => { self.advance(); Token::LParen }
                ')' => { self.advance(); Token::RParen }
                '{' => { self.advance(); Token::LBrace }
                '}' => { self.advance(); Token::RBrace }
                '[' => { self.advance(); Token::LBracket }
                ']' => { self.advance(); Token::RBracket }
                ',' => { self.advance(); Token::Comma }
                '_' if !self.peek_at(1).is_some_and(|c| c.is_alphanumeric() || c == '_') => {
                    self.advance();
                    Token::Underscore
                }
                '"' => self.read_string()?,
                '\'' => self.read_char()?,
                c if c.is_ascii_digit() => self.read_number()?,
                c if c.is_alphabetic() || c == '_' => self.read_ident(),
                _ => {
                    return Err(format!(
                        "line {line}:{col}: unexpected character '{ch}'"
                    ))
                }
            };

            tokens.push(Spanned { token, line, col });
        }

        tokens.push(Spanned {
            token: Token::Eof,
            line: self.line,
            col: self.col,
        });

        Ok(tokens)
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            if self.source[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.advance();
        let mut s = String::new();
        while self.pos < self.source.len() && self.source[self.pos] != '"' {
            if self.source[self.pos] == '\\' {
                self.advance();
                if self.pos >= self.source.len() {
                    return Err(format!(
                        "line {}:{}: unterminated string escape",
                        self.line, self.col
                    ));
                }
                match self.source[self.pos] {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    '{' => s.push('{'),
                    '}' => s.push('}'),
                    '0' => s.push('\0'),
                    c => {
                        s.push('\\');
                        s.push(c);
                    }
                }
            } else {
                s.push(self.source[self.pos]);
            }
            self.advance();
        }
        if self.pos >= self.source.len() {
            return Err(format!(
                "line {}:{}: unterminated string literal",
                self.line, self.col
            ));
        }
        self.advance();
        Ok(Token::StringLit(s))
    }

    fn read_char(&mut self) -> Result<Token, String> {
        self.advance();
        let ch = if self.pos < self.source.len() && self.source[self.pos] == '\\' {
            self.advance();
            match self.source.get(self.pos) {
                Some('n') => '\n',
                Some('t') => '\t',
                Some('r') => '\r',
                Some('\\') => '\\',
                Some('\'') => '\'',
                Some('0') => '\0',
                _ => {
                    return Err(format!(
                        "line {}:{}: invalid escape in char literal",
                        self.line, self.col
                    ))
                }
            }
        } else if self.pos < self.source.len() {
            self.source[self.pos]
        } else {
            return Err(format!(
                "line {}:{}: unterminated char literal",
                self.line, self.col
            ));
        };
        self.advance();
        if self.pos >= self.source.len() || self.source[self.pos] != '\'' {
            return Err(format!(
                "line {}:{}: unterminated char literal",
                self.line, self.col
            ));
        }
        self.advance();
        Ok(Token::CharLit(ch))
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut num = String::new();
        let mut is_float = false;

        while self.pos < self.source.len()
            && (self.source[self.pos].is_ascii_digit() || self.source[self.pos] == '_')
        {
            if self.source[self.pos] != '_' {
                num.push(self.source[self.pos]);
            }
            self.advance();
        }

        if self.pos < self.source.len()
            && self.source[self.pos] == '.'
            && self.peek_at(1).is_some_and(|c| c.is_ascii_digit())
        {
            is_float = true;
            num.push('.');
            self.advance();
            while self.pos < self.source.len()
                && (self.source[self.pos].is_ascii_digit() || self.source[self.pos] == '_')
            {
                if self.source[self.pos] != '_' {
                    num.push(self.source[self.pos]);
                }
                self.advance();
            }
        }

        if is_float {
            num.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| format!("line {}:{}: invalid float literal", self.line, self.col))
        } else {
            num.parse::<i64>()
                .map(Token::Int)
                .map_err(|_| format!("line {}:{}: integer literal too large", self.line, self.col))
        }
    }

    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();
        while self.pos < self.source.len()
            && (self.source[self.pos].is_alphanumeric() || self.source[self.pos] == '_')
        {
            ident.push(self.source[self.pos]);
            self.advance();
        }

        match ident.as_str() {
            "fn" => Token::Fn,
            "var" => Token::Var,
            "mut" => Token::Mut,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "match" => Token::Match,
            "class" => Token::Class,
            "methods" => Token::Methods,
            "enum" => Token::Enum,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "true" => Token::True,
            "false" => Token::False,
            "self" => Token::Self_,
            "trait" => Token::Trait,
            "use" => Token::Use,
            "const" => Token::Const,
            "type" => Token::TypeKw,
            "as" => Token::As,
            "int" => Token::IntType,
            "float" => Token::FloatType,
            "string" => Token::StringType,
            "bool" => Token::BoolType,
            "char" => Token::CharType,
            _ => Token::Ident(ident),
        }
    }
}
