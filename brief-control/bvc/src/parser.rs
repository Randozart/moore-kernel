// bvc/parser.rs - BVC Language Parser
//     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
//
// Tokenizer and parser for Brief Control orchestration language

use crate::{ControlBlock, ControlStmt, EbvData, PartitionDef};
use anyhow::{bail, Result};

pub fn parse_bvc(source: &str) -> Result<BvcProgram> {
    let mut tokens = Tokenizer::new(source).tokenize()?;
    let mut parser = Parser::new(&mut tokens);
    parser.parse()
}

#[derive(Debug, Clone, PartialEq)]
pub struct BvcProgram {
    pub using_decls: Vec<String>,
    pub control_blocks: Vec<ControlBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Using,
    Control,
    Target,
    Partition,
    Route,
    Mount,
    Unmount,
    Fence,
    Timeout,
    Enable,
    Disable,
    Ident(String),
    String(String),
    Dot,
    Colon,
    ColonColon,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Arrow,
    Equals,
    Semicolon,
    Integer(u64),
    Eof,
}

struct Tokenizer {
    source: String,
    pos: usize,
}

impl Tokenizer {
    fn new(source: &str) -> Self {
        Self { source: source.to_string(), pos: 0 }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.pos >= self.source.len() {
                tokens.push(Token::Eof);
                break;
            }
            let ch = self.source[self.pos..].chars().next().unwrap();
            let tok = self.lex_token(ch)?;
            tokens.push(tok);
        }
        Ok(tokens)
    }

    fn skip_ws_and_comments(&mut self) {
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

    fn lex_token(&mut self, ch: char) -> Result<Token> {
        match ch {
            '{' => { self.pos += 1; Ok(Token::LBrace) }
            '}' => { self.pos += 1; Ok(Token::RBrace) }
            '(' => { self.pos += 1; Ok(Token::LParen) }
            ')' => { self.pos += 1; Ok(Token::RParen) }
            ',' => { self.pos += 1; Ok(Token::Comma) }
            ';' => { self.pos += 1; Ok(Token::Semicolon) }
            '=' => {
                self.pos += 1;
                if self.peek() == Some('>') {
                    self.pos += 1;
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Equals)
                }
            }
            '.' => {
                self.pos += 1;
                if self.peek() == Some('.') {
                    self.pos += 1;
                    Ok(Token::Dot)
                } else {
                    Ok(Token::Dot)
                }
            }
            ':' => {
                self.pos += 1;
                if self.peek() == Some(':') {
                    self.pos += 1;
                    Ok(Token::ColonColon)
                } else {
                    Ok(Token::Colon)
                }
            }
            '"' => self.lex_string(),
            c if c.is_ascii_alphabetic() || c == '_' => self.lex_ident_or_keyword(),
            c if c.is_ascii_digit() => self.lex_number(),
            _ => bail!("Unexpected character '{}' at position {}", ch, self.pos),
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn lex_ident_or_keyword(&mut self) -> Result<Token> {
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
        let kw = match ident.as_str() {
            "using" => Token::Using,
            "control" => Token::Control,
            "target" => Token::Target,
            "partition" => Token::Partition,
            "route" => Token::Route,
            "mount" => Token::Mount,
            "unmount" => Token::Unmount,
            "fence" => Token::Fence,
            "timeout" => Token::Timeout,
            "enable" => Token::Enable,
            "disable" => Token::Disable,
            _ => Token::Ident(ident),
        };
        Ok(kw)
    }

    fn lex_number(&mut self) -> Result<Token> {
        let start = self.pos;
        while self.pos < self.source.len() {
            let c = self.source[self.pos..].chars().next().unwrap();
            if c.is_ascii_digit() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        let num_str = self.source[start..self.pos].to_string();
        let num = num_str.parse::<u64>().map_err(|_| anyhow::anyhow!("Invalid number: {}", num_str))?;
        Ok(Token::Integer(num))
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
        bail!("Unterminated string at position {}", start)
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: &mut Vec<Token>) -> Self {
        Self { pos: 0, tokens: tokens.clone() }
    }

    fn parse(&mut self) -> Result<BvcProgram> {
        let mut using_decls = Vec::new();
        let mut control_blocks = Vec::new();

        loop {
            match self.current() {
                Token::Using => {
                    let u = self.parse_using()?;
                    using_decls.push(u);
                }
                Token::Control => {
                    let c = self.parse_control()?;
                    control_blocks.push(c);
                }
                Token::Eof => break,
                t => bail!("Unexpected token {:?} at top level", t),
            }
        }

        Ok(BvcProgram { using_decls, control_blocks })
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        let current = self.current().clone();
        if current == expected {
            self.advance();
            Ok(())
        } else {
            bail!("Expected {:?}, got {:?}", expected, current)
        }
    }

    fn parse_using(&mut self) -> Result<String> {
        self.expect(Token::Using)?;
        match self.current() {
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                let mut full_name = name;
                loop {
                    if matches!(self.current(), Token::Dot) {
                        self.advance();
                        match self.current() {
                            Token::Ident(s) => {
                                full_name.push('.');
                                full_name.push_str(s);
                                self.advance();
                            }
                            t => bail!("Expected identifier after '.', got {:?}", t),
                        }
                    } else {
                        break;
                    }
                }
                self.expect(Token::Semicolon)?;
                Ok(full_name)
            }
            t => bail!("Expected identifier after 'using', got {:?}", t),
        }
    }

    fn parse_control(&mut self) -> Result<ControlBlock> {
        self.expect(Token::Control)?;
        let name = match self.current() {
            Token::Ident(s) => s.clone(),
            t => bail!("Expected control name, got {:?}", t),
        };
        self.advance();
        self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();
        loop {
            match self.current() {
                Token::Target => stmts.push(self.parse_target_stmt()?),
                Token::Partition => stmts.push(self.parse_partition_stmt()?),
                Token::Route => stmts.push(self.parse_route_stmt()?),
                Token::Mount => stmts.push(self.parse_mount_stmt()?),
                Token::Unmount => stmts.push(self.parse_unmount_stmt()?),
                Token::Fence => stmts.push(self.parse_fence_stmt()?),
                Token::Timeout => stmts.push(self.parse_timeout_stmt()?),
                Token::RBrace => { self.advance(); break; }
                Token::Eof => bail!("Unexpected EOF in control block"),
                t => bail!("Unexpected token in control block: {:?}", t),
            }
        }
        Ok(ControlBlock { name, stmts })
    }

    fn parse_target_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Target)?;
        let mut tiles = Vec::new();
        loop {
            match self.current() {
                Token::Ident(s) => {
                    tiles.push(s.clone());
                    self.advance();
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                        continue;
                    } else {
                        break;
                    }
                }
                t => bail!("Expected tile identifier, got {:?}", t),
            }
        }
        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Target(tiles))
    }

    fn parse_partition_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Partition)?;
        let using_ref = self.parse_using_ref()?;
        self.expect(Token::Ident("across".to_string()))?;
        let tile_ref = match self.current() {
            Token::Ident(s) => s.clone(),
            t => bail!("Expected tile identifier, got {:?}", t),
        };
        self.advance();
        let slot_id = self.parse_optional_ident_as();
        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Partition(crate::PartitionStmt {
            using_ref,
            tile_ref,
            slot_id,
        }))
    }

    fn parse_route_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Route)?;
        let route_name = match self.current() {
            Token::Ident(s) => s.clone(),
            t => bail!("Expected route name, got {:?}", t),
        };
        self.advance();

        let (from_tile, to_tile, port_ref) = self.parse_route_body()?;

        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Route(crate::RouteStmt {
            route_name,
            from_tile,
            to_tile,
            port_ref,
        }))
    }

    fn parse_route_body(&mut self) -> Result<(Option<String>, Option<String>, String)> {
        match self.current() {
            Token::Ident(s) if s == "from" => {
                self.advance();
                let from = match self.current() {
                    Token::Ident(s) => s.clone(),
                    t => bail!("Expected tile identifier, got {:?}", t),
                };
                self.advance();
                self.expect(Token::Ident("to".to_string()))?;
                let to = match self.current() {
                    Token::Ident(s) => s.clone(),
                    t => bail!("Expected tile identifier, got {:?}", t),
                };
                self.advance();
                self.expect(Token::Ident("over".to_string()))?;
                let port = match self.current() {
                    Token::Ident(s) => s.clone(),
                    t => bail!("Expected port identifier, got {:?}", t),
                };
                self.advance();
                Ok((Some(from), Some(to), port))
            }
            Token::Ident(s) if s == "over" => {
                self.advance();
                let port = match self.current() {
                    Token::Ident(s) => s.clone(),
                    t => bail!("Expected port identifier, got {:?}", t),
                };
                self.advance();
                Ok((None, None, port))
            }
            t => bail!("Expected 'over' or 'from...to' in route statement"),
        }
    }

    fn parse_mount_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Mount)?;
        let using_ref = self.parse_using_ref()?;
        self.expect(Token::Ident("to".to_string()))?;
        let tile_ref = match self.current() {
            Token::Ident(s) => s.clone(),
            t => bail!("Expected tile identifier, got {:?}", t),
        };
        self.advance();
        let slot_id = self.parse_optional_ident_as();
        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Mount(crate::MountStmt { using_ref, tile_ref, slot_id }))
    }

    fn parse_unmount_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Unmount)?;
        let using_ref = self.parse_using_ref()?;
        let tile_ref = self.parse_optional_ident_from();
        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Unmount(crate::UnmountStmt { using_ref, tile_ref }))
    }

    fn parse_fence_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Fence)?;
        let slot_id = match self.current() {
            Token::Ident(s) => s.clone(),
            t => bail!("Expected slot identifier, got {:?}", t),
        };
        self.advance();
        let action = match self.current() {
            Token::Enable => { self.advance(); crate::FenceAction::Enable }
            Token::Disable => { self.advance(); crate::FenceAction::Disable }
            t => bail!("Expected 'enable' or 'disable', got {:?}", t),
        };
        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Fence(crate::FenceStmt { slot_id, action }))
    }

    fn parse_timeout_stmt(&mut self) -> Result<ControlStmt> {
        self.expect(Token::Timeout)?;
        let unit = match self.current() {
            Token::Ident(s) if s == "ms" => { self.advance(); crate::TimeoutUnit::Ms }
            Token::Ident(s) if s == "s" => { self.advance(); crate::TimeoutUnit::Sec }
            Token::Ident(s) if s == "min" => { self.advance(); crate::TimeoutUnit::Min }
            t => bail!("Expected timeout unit (ms/s/min), got {:?}", t),
        };
        self.expect(Token::Equals)?;
        let value = match self.current() {
            Token::Integer(n) => { let v = *n; self.advance(); v }
            t => bail!("Expected integer value, got {:?}", t),
        };
        self.expect(Token::Semicolon)?;
        Ok(ControlStmt::Timeout(crate::TimeoutStmt { value, unit }))
    }

    fn parse_using_ref(&mut self) -> Result<String> {
        match self.current() {
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                let mut full_name = name;
                loop {
                    if matches!(self.current(), Token::Dot) {
                        self.advance();
                        match self.current() {
                            Token::Ident(s) => {
                                full_name.push('.');
                                full_name.push_str(s);
                                self.advance();
                            }
                            t => bail!("Expected identifier after '.', got {:?}", t),
                        }
                    } else {
                        break;
                    }
                }
                Ok(full_name)
            }
            t => bail!("Expected using reference, got {:?}", t),
        }
    }

    fn parse_optional_ident_as(&mut self) -> Option<String> {
        match self.current() {
            Token::Ident(s) if s == "as" => {
                self.advance();
                match self.current() {
                    Token::Ident(s) => {
                        let id = s.clone();
                        self.advance();
                        Some(id)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn parse_optional_ident_from(&mut self) -> Option<String> {
        match self.current() {
            Token::Ident(s) if s == "from" => {
                self.advance();
                match self.current() {
                    Token::Ident(s) => {
                        let t = s.clone();
                        self.advance();
                        Some(t)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let source = r#"
using Imp_Core;
control Fabric {
    target Tile_0;
    partition Imp_Core across Tile_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        assert_eq!(program.using_decls.len(), 1);
        assert_eq!(program.control_blocks.len(), 1);
    }

    #[test]
    fn test_parse_gpu_mount() {
        let source = r#"
using Rendered_GPU;
control Display {
    target Tile_0;
    partition Rendered_GPU across Tile_0 as RP_1;
    fence RP_1 enable;
}
"#;
        let program = parse_bvc(source).unwrap();
        assert_eq!(program.using_decls[0], "Rendered_GPU");
        assert_eq!(program.control_blocks[0].name, "Display");
    }
}