// msh/parser.rs - Moore Shell LL(1) Parser
//     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
//
// Lexer and parser for Moore Shell command language

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token {
    Ident(String),
    Predicate(String),
    Subject(String),
    Dot,
    QMark,
    Comma,
    LBracket,
    RBracket,
    String(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    source: String,
    pos: usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self { source: source.to_string(), pos: 0 }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.source.len() {
                tokens.push(Token::Eof);
                break;
            }
            let ch = self.source[self.pos..].chars().next().unwrap();
            let tok = self.lex_char(ch)?;
            tokens.push(tok);
        }
        Ok(tokens)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.source.len() {
            let rest = &self.source[self.pos..];
            if rest.starts_with("//") {
                if let Some(nl) = rest.find('\n') {
                    self.pos += nl + 1;
                } else {
                    self.pos = self.source.len();
                }
            } else if let Some(c) = rest.chars().next() {
                if c.is_whitespace() {
                    self.pos += c.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn lex_char(&mut self, ch: char) -> Result<Token> {
        match ch {
            '.' => { self.pos += 1; Ok(Token::Dot) }
            '?' => { self.pos += 1; Ok(Token::QMark) }
            ',' => { self.pos += 1; Ok(Token::Comma) }
            '[' => { self.pos += 1; Ok(Token::LBracket) }
            ']' => { self.pos += 1; Ok(Token::RBracket) }
            '"' => self.lex_string(),
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_ident(),
            _ => bail!("Unexpected character '{}' at position {}", ch, self.pos),
        }
    }

    fn lex_ident(&mut self) -> Result<Token> {
        let start = self.pos;
        while self.pos < self.source.len() {
            let c = self.source[self.pos..].chars().next().unwrap();
            if c.is_ascii_alphanumeric() || c == '_' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        let ident = self.source[start..self.pos].to_string();
        Ok(Token::Ident(ident))
    }

    fn lex_string(&mut self) -> Result<Token> {
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.source[self.pos..].chars().next().unwrap();
            if ch == '"' {
                let s = self.source[start..self.pos].to_string();
                self.pos += 1;
                return Ok(Token::String(s));
            }
            self.pos += ch.len_utf8();
        }
        bail!("Unterminated string")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Proposition {
    ExistsOn { module: String, tile: String },
    Absent { module: String },
    IsActive { tile: String },
    Contains { container: String, item: String },
    Custom { subject: String, predicate: String, object: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofFailure {
    pub proposition: String,
    pub reasons: Vec<ProofReason>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofReason {
    pub code: u32,
    pub description: String,
}

impl ProofFailure {
    pub fn cannot_satisfy(prop: &str) -> Self {
        Self {
            proposition: prop.to_string(),
            reasons: vec![],
            suggestions: vec![],
        }
    }

    pub fn add_reason(&mut self, code: u32, desc: &str) {
        self.reasons.push(ProofReason { code, description: desc.to_string() });
    }

    pub fn add_suggestion(&mut self, suggestion: &str) {
        self.suggestions.push(suggestion.to_string());
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { pos: 0, tokens }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_proposition(&mut self) -> Result<Proposition> {
        match self.current() {
            Token::Ident(s) => {
                let subject = s.clone();
                self.advance();
                self.parse_predicate(subject)
            }
            t => bail!("Expected proposition, got {:?}", t),
        }
    }

    fn parse_predicate(&mut self, subject: String) -> Result<Proposition> {
        match self.current() {
            Token::QMark => {
                self.advance();
                Ok(Proposition::Custom {
                    subject,
                    predicate: "?".to_string(),
                    object: None,
                })
            }
            Token::Ident(s) => {
                let predicate = s.clone();
                self.advance();
                match predicate.as_str() {
                    "exists_on" => self.parse_exists_on(subject),
                    "absent" => {
                        if matches!(self.current(), Token::Dot) {
                            self.advance();
                        }
                        Ok(Proposition::Absent { module: subject })
                    }
                    "is_active" => {
                        if matches!(self.current(), Token::Dot) {
                            self.advance();
                        }
                        Ok(Proposition::IsActive { tile: subject })
                    }
                    "contains" => self.parse_contains(subject),
                    _ => {
                        if matches!(self.current(), Token::Dot) {
                            self.advance();
                        }
                        Ok(Proposition::Custom {
                            subject,
                            predicate,
                            object: None,
                        })
                    }
                }
            }
            Token::Dot => {
                self.advance();
                Ok(Proposition::Custom {
                    subject,
                    predicate: "?".to_string(),
                    object: None,
                })
            }
            t => bail!("Expected predicate, got {:?}", t),
        }
    }

    fn parse_exists_on(&mut self, module: String) -> Result<Proposition> {
        match self.current() {
            Token::Ident(tile) => {
                let tile_name = tile.clone();
                self.advance();
                if matches!(self.current(), Token::Dot) {
                    self.advance();
                }
                Ok(Proposition::ExistsOn { module, tile: tile_name })
            }
            t => bail!("Expected tile identifier after 'exists_on', got {:?}", t),
        }
    }

    fn parse_contains(&mut self, container: String) -> Result<Proposition> {
        match self.current() {
            Token::Ident(item) => {
                let item_name = item.clone();
                self.advance();
                if matches!(self.current(), Token::Dot) {
                    self.advance();
                }
                Ok(Proposition::Contains { container, item: item_name })
            }
            t => bail!("Expected item identifier after 'contains', got {:?}", t),
        }
    }

    fn parse_discovery(&mut self) -> Result<Proposition> {
        match self.current() {
            Token::Ident(s) => {
                let subject = s.clone();
                self.advance();
                if matches!(self.current(), Token::Dot) {
                    self.advance();
                }
                Ok(Proposition::Custom {
                    subject,
                    predicate: "?".to_string(),
                    object: None,
                })
            }
            _ => bail!("Expected identifier for discovery"),
        }
    }
}

pub fn parse(source: &str) -> Result<Proposition> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_proposition()
}

pub fn parse_line(source: &str) -> Result<Proposition> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        bail!("Empty input")
    }
    parse(trimmed)
}

pub fn format_proposition(prop: &Proposition) -> String {
    match prop {
        Proposition::ExistsOn { module, tile } => format!("{} exists_on {}.", module, tile),
        Proposition::Absent { module } => format!("{} absent.", module),
        Proposition::IsActive { tile } => format!("{} is_active.", tile),
        Proposition::Contains { container, item } => format!("{} contains {}.", container, item),
        Proposition::Custom { subject, predicate, object } => {
            if let Some(obj) = object {
                format!("{} {} {}.", subject, predicate, obj)
            } else {
                format!("{} {}.", subject, predicate)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exists_on() {
        let prop = parse("Imp_Core exists_on Tile_0.").unwrap();
        assert_eq!(prop, Proposition::ExistsOn { module: "Imp_Core".to_string(), tile: "Tile_0".to_string() });
    }

    #[test]
    fn test_parse_absent() {
        let prop = parse("Imp_Core absent.").unwrap();
        assert_eq!(prop, Proposition::Absent { module: "Imp_Core".to_string() });
    }

    #[test]
    fn test_parse_is_active() {
        let prop = parse("Tile_0 is_active.").unwrap();
        assert_eq!(prop, Proposition::IsActive { tile: "Tile_0".to_string() });
    }

    #[test]
    fn test_parse_contains() {
        let prop = parse("Storage contains Imp_Core.").unwrap();
        assert_eq!(prop, Proposition::Contains { container: "Storage".to_string(), item: "Imp_Core".to_string() });
    }

    #[test]
    fn test_parse_discovery() {
        let prop = parse("Tile_0 ?").unwrap();
        assert_eq!(prop, Proposition::Custom { subject: "Tile_0".to_string(), predicate: "?".to_string(), object: None });
    }

    #[test]
    fn test_format_proposition() {
        let prop = Proposition::ExistsOn { module: "GPU".to_string(), tile: "Tile_0".to_string() };
        assert_eq!(format_proposition(&prop), "GPU exists_on Tile_0.");
    }
}