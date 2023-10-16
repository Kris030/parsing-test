mod token;

use crate::Diagnostic;

use super::Source;
pub use token::{TokenType as Ty, *};

#[derive(thiserror::Error, Debug)]
pub enum TokenizerError {
    #[error("Unfinished string literal")]
    UnfinishedString,

    #[error("Unfinished character literal")]
    UnfinishedChar,

    #[error("Already done")]
    AlreadyDone,

    #[error("Unexpected byte: {0:#02x} ({})", *.0 as char)]
    Unexpected(u8),
}

pub struct Tokenizer<'n, 's, 'd, D> {
    source: Source<'n, 's>,

    done: bool,

    pos: usize,
    lookahead: usize,

    line: usize,
    column: usize,

    diagnostics: &'d mut D,
}

impl<'n, 's, 'd, D: Extend<Diagnostic<'s>>> Tokenizer<'n, 's, 'd, D> {
    pub fn new(source: Source<'n, 's>, diagnostics: &'d mut D) -> Self {
        Self {
            done: source.bin.is_empty(),

            source,

            pos: 0,
            lookahead: 1,

            line: 1,
            column: 1,

            diagnostics,
        }
    }

    pub fn pos(&self) -> TokenPosition<'s> {
        TokenPosition {
            absolute_position: self.pos,
            line: self.line,
            column: self.column,
            text: &self.source.text[self.pos..self.lookahead],
        }
    }

    fn curr(&self) -> u8 {
        self.source.bin[self.pos]
    }
    fn peek(&self) -> Option<u8> {
        self.source.bin.get(self.lookahead).copied()
    }
    fn peek_next(&mut self) -> Option<u8> {
        let v = self.peek();
        self.step();
        v
    }

    fn step(&mut self) {
        if !self.done {
            self.lookahead += 1;

            if self.lookahead >= self.source.bin.len() {
                self.done = true;
            }
        }
    }
    fn step_back(&mut self) {
        if self.lookahead > self.pos {
            self.lookahead -= 1;
        }
    }

    fn eat(&mut self, v: u8) -> bool {
        if self.peek() == Some(v) {
            self.step();
            true
        } else {
            false
        }
    }

    fn consume(&mut self) {
        self.pos = self.lookahead;
        self.done = self.pos >= self.source.bin.len();
        self.lookahead += 1;
    }

    fn number(&mut self, first_char: u8) -> NumberLiteral {
        let mut v = vec![first_char - b'0'];

        loop {
            match self.peek() {
                Some(c @ b'0'..=b'9') => v.push(c - b'0'),
                Some(b'_') => (),

                _ => break,
            }
            self.step();
        }

        let mut int_part = 0;
        let mut pow = 1;
        for c in v.iter().rev() {
            int_part += *c as u64 * pow;
            pow *= 10;
        }

        if !self.eat(b'.') {
            return NumberLiteral::Integer(int_part);
        }

        // float
        v.clear();
        loop {
            match self.peek() {
                Some(c @ b'0'..=b'9') => v.push(c - b'0'),

                Some(b'_') => (),

                _ => break,
            }

            self.step();
        }

        let mut float_part = 0.;
        let mut pow = 0.1;
        for c in v.iter() {
            float_part += *c as f64 * pow;
            pow *= 0.1;
        }

        NumberLiteral::Float(int_part as f64 + float_part)
    }

    fn char_lit(&mut self) -> Result<char, TokenizerError> {
        let c = match self.peek().ok_or(TokenizerError::UnfinishedChar)? {
            b'\\' => {
                self.step();
                self.peek_next().ok_or(TokenizerError::UnfinishedChar)? as char
            }

            c => {
                self.step();

                c as char
            }
        };

        if self.eat(b'\'') {
            Ok(c)
        } else {
            Err(TokenizerError::UnfinishedChar)
        }
    }

    fn string_lit(&mut self) -> Result<String, TokenizerError> {
        let mut s = String::new();
        let mut escaped = false;

        loop {
            match self.peek_next().ok_or(TokenizerError::UnfinishedString)? {
                b'"' if !escaped => break,
                b'\\' if !escaped => escaped = true,

                c if escaped => {
                    if let Some(escaped) = escape(c) {
                        s.push_str(escaped);
                    } else {
                        s.push(c as char);
                    }

                    escaped = false;
                }

                c => s.push(c as char),
            }
        }

        Ok(s)
    }

    fn multiline_comment(&mut self) -> Comment {
        self.step();

        let Some(mut pc) = self.peek_next() else {
            return Comment::MultiLine;
        };
        let mut nest = 1usize;

        let mut closed = false;
        while let Some(c) = self.peek_next() {
            match (pc, c) {
                (b'*', b'/') => {
                    nest -= 1;
                    if nest == 0 {
                        closed = true;
                        break;
                    }
                }

                (b'/', b'*') => nest += 1,

                _ => (),
            }

            pc = c;
        }

        if !closed {
            self.diagnostics.extend([Diagnostic::new(
                crate::DiagnosticType::UnclosedMultilineComment,
                self.pos(),
            )]);
        }

        Comment::MultiLine
    }
    fn singleline_comment(&mut self) -> Comment {
        while self.peek() != Some(b'\n') {
            self.step();
        }
        Comment::SingleLine
    }

    fn get_token_inner(&mut self) -> Result<Token<'s>, TokenizerError> {
        if self.done {
            return Err(TokenizerError::AlreadyDone);
        }

        let ty = match self.curr() {
            b'(' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Left,
                ty: DelimeterType::Parentheses,
            }),
            b')' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Right,
                ty: DelimeterType::Parentheses,
            }),
            b'[' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Left,
                ty: DelimeterType::Square,
            }),
            b']' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Right,
                ty: DelimeterType::Square,
            }),
            b'{' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Left,
                ty: DelimeterType::Curly,
            }),
            b'}' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Right,
                ty: DelimeterType::Curly,
            }),

            b'@' => Ty::Punctuation(Punctuation::AtSign),
            b',' => Ty::Punctuation(Punctuation::Colon),
            b';' => Ty::Punctuation(Punctuation::Semicolon),
            b':' => Ty::Punctuation(Punctuation::Colon),
            b'#' => Ty::Punctuation(Punctuation::HashSymbol),
            b'?' => Ty::Punctuation(Punctuation::QuestionMark),
            b'$' => {
                if self.eat(b'$') {
                    Ty::Punctuation(Punctuation::DoubleDollar)
                } else {
                    Ty::Punctuation(Punctuation::Dollar)
                }
            }

            b'=' => match self.peek_next() {
                Some(b'>') => Ty::Punctuation(Punctuation::FatArrow),
                Some(b'=') => Ty::Operator(Operator::DoubleEquals),

                _ => {
                    self.step_back();
                    Ty::Operator(Operator::Equals)
                }
            },

            b'.' => {
                if self.eat(b'.') {
                    if self.eat(b'.') {
                        Ty::Punctuation(Punctuation::TripleDot)
                    } else {
                        Ty::Punctuation(Punctuation::DoubleDot)
                    }
                } else {
                    Ty::Punctuation(Punctuation::Dot)
                }
            }

            b' ' | b'\t' | b'\n' => {
                while matches!(self.peek(), Some(b' ' | b'\t' | b'\n')) {
                    self.step();
                }

                Ty::Whitespace
            }

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while matches!(
                    self.peek(),
                    Some(b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9')
                ) {
                    self.step();
                }

                get_keyword(self.pos().text)
                    .map(Ty::Keyword)
                    .unwrap_or(Ty::Identifier)
            }

            c @ b'0'..=b'9' => Ty::Literal(Literal::Number(self.number(c))),

            b'\'' => Ty::Literal(Literal::Char(self.char_lit()?)),
            b'\"' => Ty::Literal(Literal::String(self.string_lit()?)),

            b'+' => match self.peek() {
                Some(b'=') => Ty::Operator(Operator::PlusEquals),
                Some(b'+') => Ty::Operator(Operator::DoublePlus),
                _ => Ty::Operator(Operator::Plus),
            },
            b'-' => match self.peek() {
                Some(b'=') => Ty::Operator(Operator::MinusEquals),
                Some(b'-') => Ty::Operator(Operator::DoubleMinus),
                _ => Ty::Operator(Operator::Minus),
            },
            b'*' => match self.peek() {
                Some(b'=') => Ty::Operator(Operator::StarEquals),
                Some(b'*') => {
                    self.step();
                    if self.eat(b'=') {
                        Ty::Operator(Operator::DoubleStarEquals)
                    } else {
                        Ty::Operator(Operator::DoubleStar)
                    }
                }
                _ => Ty::Operator(Operator::Star),
            },
            b'!' => {
                if self.eat(b'=') {
                    Ty::Operator(Operator::BangEquals)
                } else {
                    Ty::Operator(Operator::Bang)
                }
            }
            b'~' => {
                if self.eat(b'=') {
                    Ty::Operator(Operator::TildaEquals)
                } else {
                    Ty::Operator(Operator::Tilda)
                }
            }
            b'^' => {
                if self.eat(b'=') {
                    Ty::Operator(Operator::CaretEquals)
                } else {
                    Ty::Operator(Operator::Caret)
                }
            }
            b'%' => {
                if self.eat(b'=') {
                    Ty::Operator(Operator::PercentEquals)
                } else {
                    Ty::Operator(Operator::Percent)
                }
            }

            b'/' => match self.peek() {
                Some(b'*') => Ty::Comment(self.multiline_comment()),
                Some(b'/') => Ty::Comment(self.singleline_comment()),

                Some(b'=') => Ty::Operator(Operator::SlashEquals),

                _ => Ty::Operator(Operator::Slash),
            },

            b'&' => match self.peek() {
                Some(b'&') => {
                    self.step();
                    if self.eat(b'=') {
                        Ty::Operator(Operator::DoubleAndEquals)
                    } else {
                        Ty::Operator(Operator::DoubleAnd)
                    }
                }
                Some(b'=') => TokenType::Operator(Operator::SingleAndEquals),
                _ => Ty::Operator(Operator::SingleAnd),
            },
            b'|' => match self.peek() {
                Some(b'|') => {
                    self.step();
                    if self.eat(b'=') {
                        Ty::Operator(Operator::DoubleOrEquals)
                    } else {
                        Ty::Operator(Operator::DoubleOr)
                    }
                }
                Some(b'=') => TokenType::Operator(Operator::SingleOrEquals),
                _ => Ty::Operator(Operator::SingleOr),
            },

            b'<' => match self.peek_next() {
                Some(b'=') => Ty::Operator(Operator::LesserThanEquals),

                Some(b'<') => {
                    if self.peek() == Some(b'=') {
                        self.step();
                        Ty::Operator(Operator::LeftShiftEquals)
                    } else {
                        Ty::Operator(Operator::LeftShift)
                    }
                }

                _ => {
                    self.step_back();
                    Ty::Operator(Operator::LesserThan)
                }
            },

            b'>' => match self.peek_next() {
                Some(b'=') => Ty::Operator(Operator::GreaterThanEquals),

                Some(b'>') => {
                    if self.peek() == Some(b'=') {
                        self.step();
                        Ty::Operator(Operator::RightShiftEquals)
                    } else {
                        Ty::Operator(Operator::RightShift)
                    }
                }

                _ => {
                    self.step_back();
                    Ty::Operator(Operator::GreaterThan)
                }
            },

            b => return Err(TokenizerError::Unexpected(b)),
        };

        Ok(Token {
            position: self.pos(),
            ty,
        })
    }

    fn get_token(&mut self) -> Result<Token<'s>, TokenizerError> {
        let t = self.get_token_inner();
        self.consume();

        t
    }
}

impl<'n, 's, 'd, D: Extend<Diagnostic<'s>>> Iterator for Tokenizer<'n, 's, 'd, D> {
    type Item = Result<Token<'s>, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        Some(self.get_token())
    }
}

fn escape(c: u8) -> Option<&'static str> {
    Some(match c {
        b'n' => "\n",
        b't' => "\t",
        b'0' => "\0",

        _ => return None,
    })
}
