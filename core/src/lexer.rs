//! Lexer for Susumu arrow-flow language

use crate::error::{SusumuError, SusumuResult};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals
    Number,
    String,
    Boolean,
    Null,

    // Identifiers and keywords
    Identifier,
    Function,
    Return,
    Error,
    If,
    Else,
    ForEach,
    In,

    // Arrows
    RightArrow, // ->
    LeftArrow,  // <-

    // Conditional keywords
    I,  // i (condition)
    E,  // e (else)
    Ei, // ei (else if)
    // Pattern matching keywords
    Match, // match
    When,  // when (guard)
    True,  // true
    False, // false
    Mut,   // mut

    // Operators
    Plus,       // +
    Minus,      // -
    Multiply,   // *
    Divide,     // /
    Assign,     // =
    Equal,      // ==
    NotEqual,   // !=
    Less,       // <
    Greater,    // >
    LessEq,     // <=
    GreaterEq,  // >=
    Dot,        // .
    Underscore, // _
    At,         // @ (for annotations)

    // Punctuation
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Colon,        // :
    Semicolon,    // ;

    // Special
    Newline,
    EOF,

    // Comments
    Comment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    chars: Vec<char>, // Pre-computed character array for proper indexing
    position: usize,  // Character position (not byte position)
    current_line: usize,
    current_column: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            position: 0,
            current_line: 1,
            current_column: 1,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> SusumuResult<Vec<Token>> {
        while !self.is_at_end() {
            self.scan_token()?;
        }

        self.add_token(TokenType::EOF, "");
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> SusumuResult<()> {
        let c = self.advance();

        match c {
            ' ' | '\r' | '\t' => {
                // Ignore whitespace
            }
            '\n' => {
                self.add_token(TokenType::Newline, "\n");
                self.current_line += 1;
                self.current_column = 1;
            }
            '(' => self.add_token(TokenType::LeftParen, "("),
            ')' => self.add_token(TokenType::RightParen, ")"),
            '{' => self.add_token(TokenType::LeftBrace, "{"),
            '}' => self.add_token(TokenType::RightBrace, "}"),
            '[' => self.add_token(TokenType::LeftBracket, "["),
            ']' => self.add_token(TokenType::RightBracket, "]"),
            ',' => self.add_token(TokenType::Comma, ","),
            ':' => self.add_token(TokenType::Colon, ":"),
            ';' => self.add_token(TokenType::Semicolon, ";"),
            '-' => {
                if self.peek() == '>' {
                    self.advance();
                    self.add_token(TokenType::RightArrow, "->");
                } else {
                    self.add_token(TokenType::Minus, "-");
                }
            }
            '<' => {
                if self.peek() == '-' {
                    self.advance();
                    self.add_token(TokenType::LeftArrow, "<-");
                } else if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::LessEq, "<=");
                } else {
                    self.add_token(TokenType::Less, "<");
                }
            }
            '/' => {
                if self.peek() == '/' {
                    // Line comment
                    self.line_comment()?;
                } else {
                    self.add_token(TokenType::Divide, "/");
                }
            }
            '+' => self.add_token(TokenType::Plus, "+"),
            '*' => self.add_token(TokenType::Multiply, "*"),
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::Equal, "==");
                } else {
                    self.add_token(TokenType::Assign, "=");
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::NotEqual, "!=");
                } else {
                    return Err(SusumuError::lexer_error(
                        self.current_line,
                        self.current_column - 1,
                        "Unexpected character '!'",
                    ));
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::GreaterEq, ">=");
                } else {
                    self.add_token(TokenType::Greater, ">");
                }
            }
            '.' => self.add_token(TokenType::Dot, "."),
            '_' => self.add_token(TokenType::Underscore, "_"),
            '@' => self.add_token(TokenType::At, "@"),
            '"' => self.string_literal()?,
            c if c.is_ascii_digit() => self.number_literal()?,
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier_or_keyword()?,
            _ => {
                return Err(SusumuError::lexer_error(
                    self.current_line,
                    self.current_column - 1,
                    format!("Unexpected character: '{}'", c),
                ));
            }
        }

        Ok(())
    }

    fn line_comment(&mut self) -> SusumuResult<()> {
        let start = if self.position >= 2 {
            self.position - 2
        } else {
            0
        }; // Include the //
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
        let comment_text: String = self.chars[start..self.position].iter().collect();
        self.add_token(TokenType::Comment, &comment_text);
        Ok(())
    }

    fn string_literal(&mut self) -> SusumuResult<()> {
        let start = self.position - 1; // Include opening quote

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(SusumuError::lexer_error(
                self.current_line,
                self.current_column,
                "Unterminated string",
            ));
        }

        // Consume closing quote
        self.advance();

        // Get string content without quotes
        let value: String = self.chars[(start + 1)..(self.position - 1)]
            .iter()
            .collect();
        self.add_token(TokenType::String, &value);

        Ok(())
    }

    fn number_literal(&mut self) -> SusumuResult<()> {
        let start = self.position - 1;

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Handle decimal point
        if self.peek() == '.' && self.peek_ahead(1).is_ascii_digit() {
            self.advance(); // Consume '.'
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let number_str: String = self.chars[start..self.position].iter().collect();
        self.add_token(TokenType::Number, &number_str);

        Ok(())
    }

    fn identifier_or_keyword(&mut self) -> SusumuResult<()> {
        let start = self.position - 1;

        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.chars[start..self.position].iter().collect();
        let token_type = match text.as_str() {
            "function" => TokenType::Function,
            "return" => TokenType::Return,
            "error" => TokenType::Error,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "fe" => TokenType::ForEach, // for-each abbreviation
            "in" => TokenType::In,
            "i" => TokenType::I,
            "e" => TokenType::E,
            "ei" => TokenType::Ei,
            "match" => TokenType::Match,
            "when" => TokenType::When,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "mut" => TokenType::Mut,
            _ => TokenType::Identifier,
        };

        self.add_token(token_type, &text);
        Ok(())
    }

    fn add_token(&mut self, token_type: TokenType, lexeme: &str) {
        self.tokens.push(Token {
            token_type,
            lexeme: lexeme.to_string(),
            line: self.current_line,
            column: self.current_column.saturating_sub(lexeme.chars().count()),
        });
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.position += 1; // Advance by 1 character, not bytes
        self.current_column += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.position]
        }
    }

    fn peek_ahead(&self, distance: usize) -> char {
        let pos = self.position + distance;
        if pos >= self.chars.len() {
            '\0'
        } else {
            self.chars[pos]
        }
    }

    fn current_char(&self) -> char {
        if self.position >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.position]
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.chars.len()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}('{}') at {}:{}",
            self.token_type, self.lexeme, self.line, self.column
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("5 -> add <- 3");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[1].token_type, TokenType::RightArrow);
        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[3].token_type, TokenType::LeftArrow);
        assert_eq!(tokens[4].token_type, TokenType::Number);
    }

    #[test]
    fn test_conditional_tokens() {
        let mut lexer = Lexer::new("i success { result -> return } e { error -> error }");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::I);
        assert_eq!(tokens[1].token_type, TokenType::Success);
        assert_eq!(tokens[2].token_type, TokenType::LeftBrace);
        // ... more assertions
    }

    #[test]
    fn test_string_literals() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].lexeme, "hello world");
    }
}
