// OpenQASM 2.0 recursive descent parser.
// Consumes a token stream from the lexer and produces a Circuit.
// Phase 1 deliverable.

use qforge_ir::{Circuit, Gate, QubitRef, CbitRef};
use crate::{error::ParseError, lexer::{Lexer, Token}};
use std::f64::consts::PI;

pub fn parse(source: &str) -> Result<Circuit, ParseError> {
    let tokens = Lexer::new(source).tokenise()?;
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos:    usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }

    fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::Eof) }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.peek() == expected {
            self.advance(); Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                line: 0,
                message: format!("expected {:?}, got {:?}", expected, self.peek()),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            other => Err(ParseError::UnexpectedToken {
                line: 0, message: format!("expected identifier, got {:?}", other),
            }),
        }
    }

    fn expect_int(&mut self) -> Result<usize, ParseError> {
        match self.advance() {
            Token::Integer(n) if n >= 0 => Ok(n as usize),
            other => Err(ParseError::UnexpectedToken {
                line: 0, message: format!("expected integer, got {:?}", other),
            }),
        }
    }

    fn parse_program(&mut self) -> Result<Circuit, ParseError> {
        // OPENQASM 2.0;
        if self.peek() == &Token::OpenQasm {
            self.advance();
            // consume version number
            match self.advance() {
                Token::Real(_) | Token::Integer(_) => {}
                _ => {}
            }
            self.expect(&Token::Semicolon)?;
        }

        let mut circuit = Circuit::new("circuit");

        loop {
            match self.peek().clone() {
                Token::Eof => break,
                Token::Include => {
                    self.advance();
                    // skip include "qelib1.inc";
                    match self.advance() {
                        Token::StringLit(_) => {}
                        _ => {}
                    }
                    self.expect(&Token::Semicolon)?;
                }
                Token::Qreg => {
                    self.advance();
                    let name = self.expect_ident()?;
                    self.expect(&Token::LBracket)?;
                    let size = self.expect_int()?;
                    self.expect(&Token::RBracket)?;
                    self.expect(&Token::Semicolon)?;
                    circuit.qregs.insert(name, size);
                }
                Token::Creg => {
                    self.advance();
                    let name = self.expect_ident()?;
                    self.expect(&Token::LBracket)?;
                    let size = self.expect_int()?;
                    self.expect(&Token::RBracket)?;
                    self.expect(&Token::Semicolon)?;
                    circuit.cregs.insert(name, size);
                }
                Token::Gate => {
                    // skip custom gate definitions for now
                    self.advance();
                    while !matches!(self.peek(), Token::RBrace | Token::Eof) {
                        self.advance();
                    }
                    if self.peek() == &Token::RBrace { self.advance(); }
                }
                Token::Opaque => {
                    self.advance();
                    while !matches!(self.peek(), Token::Semicolon | Token::Eof) {
                        self.advance();
                    }
                    if self.peek() == &Token::Semicolon { self.advance(); }
                }
                Token::Barrier => {
                    self.advance();
                    let mut qubits = Vec::new();
                    loop {
                        let q = self.parse_qubit_ref(&circuit)?;
                        qubits.push(q);
                        if self.peek() == &Token::Comma { self.advance(); }
                        else { break; }
                    }
                    self.expect(&Token::Semicolon)?;
                    circuit.push(Gate::Barrier(qubits));
                }
                Token::Measure => {
                    self.advance();
                    let q = self.parse_qubit_ref(&circuit)?;
                    self.expect(&Token::Arrow)?;
                    let c = self.parse_cbit_ref(&circuit)?;
                    self.expect(&Token::Semicolon)?;
                    circuit.push(Gate::Measure(q, c));
                }
                Token::Reset => {
                    self.advance();
                    let q = self.parse_qubit_ref(&circuit)?;
                    self.expect(&Token::Semicolon)?;
                    circuit.push(Gate::Reset(q));
                }
                Token::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    let gate = self.parse_gate_call(&name, &circuit)?;
                    self.expect(&Token::Semicolon)?;
                    circuit.push(gate);
                }
                _ => { self.advance(); }
            }
        }

        Ok(circuit)
    }

    fn parse_qubit_ref(&mut self, circuit: &Circuit) -> Result<QubitRef, ParseError> {
        let reg = self.expect_ident()?;
        if !circuit.qregs.contains_key(&reg) {
            return Err(ParseError::UndefinedRegister(reg));
        }
        self.expect(&Token::LBracket)?;
        let idx = self.expect_int()?;
        self.expect(&Token::RBracket)?;
        let size = circuit.qregs[&reg];
        if idx >= size {
            return Err(ParseError::IndexOutOfBounds { register: reg, index: idx, size });
        }
        Ok(QubitRef::new(reg, idx))
    }

    fn parse_cbit_ref(&mut self, circuit: &Circuit) -> Result<CbitRef, ParseError> {
        let reg = self.expect_ident()?;
        if !circuit.cregs.contains_key(&reg) {
            return Err(ParseError::UndefinedRegister(reg));
        }
        self.expect(&Token::LBracket)?;
        let idx = self.expect_int()?;
        self.expect(&Token::RBracket)?;
        Ok(CbitRef::new(reg, idx))
    }

    fn parse_params(&mut self) -> Result<Vec<f64>, ParseError> {
        if self.peek() != &Token::LParen { return Ok(vec![]); }
        self.advance();
        let mut params = Vec::new();
        loop {
            let p = self.parse_expr()?;
            params.push(p);
            if self.peek() == &Token::Comma { self.advance(); }
            else { break; }
        }
        self.expect(&Token::RParen)?;
        Ok(params)
    }

    fn parse_expr(&mut self) -> Result<f64, ParseError> {
        let mut val = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Plus  => { self.advance(); val += self.parse_unary()?; }
                Token::Minus => { self.advance(); val -= self.parse_unary()?; }
                Token::Star  => { self.advance(); val *= self.parse_unary()?; }
                Token::Slash => { self.advance(); val /= self.parse_unary()?; }
                Token::Caret => { self.advance(); val = val.powf(self.parse_unary()?); }
                _ => break,
            }
        }
        Ok(val)
    }

    fn parse_unary(&mut self) -> Result<f64, ParseError> {
        if self.peek() == &Token::Minus {
            self.advance();
            return Ok(-self.parse_primary()?);
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<f64, ParseError> {
        match self.advance() {
            Token::Real(v)    => Ok(v),
            Token::Integer(n) => Ok(n as f64),
            Token::LParen     => {
                let v = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(v)
            }
            Token::Ident(s) => {
                match s.as_str() {
                    "pi" => Ok(PI),
                    "sin" => { self.expect(&Token::LParen)?; let v = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(v.sin()) }
                    "cos" => { self.expect(&Token::LParen)?; let v = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(v.cos()) }
                    "tan" => { self.expect(&Token::LParen)?; let v = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(v.tan()) }
                    "exp" => { self.expect(&Token::LParen)?; let v = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(v.exp()) }
                    "ln"  => { self.expect(&Token::LParen)?; let v = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(v.ln()) }
                    "sqrt"=> { self.expect(&Token::LParen)?; let v = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(v.sqrt()) }
                    _ => Err(ParseError::InvalidParameter(format!("unknown function: {}", s))),
                }
            }
            other => Err(ParseError::UnexpectedToken {
                line: 0, message: format!("expected number, got {:?}", other),
            }),
        }
    }

    fn parse_gate_call(&mut self, name: &str, circuit: &Circuit) -> Result<Gate, ParseError> {
        let params = self.parse_params()?;

        match name {
            "h"   => Ok(Gate::H(self.parse_qubit_ref(circuit)?)),
            "x"   => Ok(Gate::X(self.parse_qubit_ref(circuit)?)),
            "y"   => Ok(Gate::Y(self.parse_qubit_ref(circuit)?)),
            "z"   => Ok(Gate::Z(self.parse_qubit_ref(circuit)?)),
            "s"   => Ok(Gate::S(self.parse_qubit_ref(circuit)?)),
            "sdg" => Ok(Gate::Sdg(self.parse_qubit_ref(circuit)?)),
            "t"   => Ok(Gate::T(self.parse_qubit_ref(circuit)?)),
            "tdg" => Ok(Gate::Tdg(self.parse_qubit_ref(circuit)?)),
            "rx"  => { let q = self.parse_qubit_ref(circuit)?; Ok(Gate::Rx(params.first().copied().unwrap_or(0.0), q)) }
            "ry"  => { let q = self.parse_qubit_ref(circuit)?; Ok(Gate::Ry(params.first().copied().unwrap_or(0.0), q)) }
            "rz"  => { let q = self.parse_qubit_ref(circuit)?; Ok(Gate::Rz(params.first().copied().unwrap_or(0.0), q)) }
            "u1"  => { let q = self.parse_qubit_ref(circuit)?; Ok(Gate::U1(params.first().copied().unwrap_or(0.0), q)) }
            "u2"  => { let q = self.parse_qubit_ref(circuit)?; Ok(Gate::U2(params.first().copied().unwrap_or(0.0), params.get(1).copied().unwrap_or(0.0), q)) }
            "u3"  => { let q = self.parse_qubit_ref(circuit)?; Ok(Gate::U3(params.first().copied().unwrap_or(0.0), params.get(1).copied().unwrap_or(0.0), params.get(2).copied().unwrap_or(0.0), q)) }
            "cx" | "cnot" => {
                let ctrl = self.parse_qubit_ref(circuit)?;
                self.expect(&Token::Comma)?;
                let tgt  = self.parse_qubit_ref(circuit)?;
                Ok(Gate::Cx(ctrl, tgt))
            }
            "cz" => {
                let q0 = self.parse_qubit_ref(circuit)?;
                self.expect(&Token::Comma)?;
                let q1 = self.parse_qubit_ref(circuit)?;
                Ok(Gate::Cz(q0, q1))
            }
            "swap" => {
                let q0 = self.parse_qubit_ref(circuit)?;
                self.expect(&Token::Comma)?;
                let q1 = self.parse_qubit_ref(circuit)?;
                Ok(Gate::Swap(q0, q1))
            }
            "ccx" | "toffoli" => {
                let q0 = self.parse_qubit_ref(circuit)?;
                self.expect(&Token::Comma)?;
                let q1 = self.parse_qubit_ref(circuit)?;
                self.expect(&Token::Comma)?;
                let q2 = self.parse_qubit_ref(circuit)?;
                Ok(Gate::Ccx(q0, q1, q2))
            }
            _ => {
                // Custom gate: collect all qubit args
                let mut qubits = Vec::new();
                while matches!(self.peek(), Token::Ident(_)) {
                    qubits.push(self.parse_qubit_ref(circuit)?);
                    if self.peek() == &Token::Comma { self.advance(); } else { break; }
                }
                Ok(Gate::Custom { name: name.to_string(), params, qubits })
            }
        }
    }
}
