mod token;

use crate::{Diagnostic, DiagnosticType};

use super::Source;
pub use token::{TokenType as Ty, *};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum TokenizerError {
    #[error("Unfinished string literal")]
    UnfinishedString,

    #[error("Unfinished character literal")]
    UnfinishedChar,

    #[error("Already done")]
    AlreadyDone,

    #[error("Unexpected character: {}", *.0)]
    Unexpected(char),
}

pub struct Tokenizer<'n, 's, 'd, D> {
    source: Source<'n, 's>,

    // done: bool,
    token_start: usize,
    token_end: usize,

    start_line: usize,
    start_column: usize,

    chrs: std::iter::Peekable<std::str::CharIndices<'s>>,

    newlines: Vec<usize>,

    diagnostics: &'d mut D,

    pub emit_whitespace: bool,
    pub emit_comments: bool,
}

impl<'n, 's, 'd, D: Extend<Diagnostic<'s>>> Tokenizer<'n, 's, 'd, D> {
    pub fn new(source: Source<'n, 's>, diagnostics: &'d mut D) -> Self {
        Self {
            chrs: source.text.char_indices().peekable(),
            source,

            newlines: vec![0],

            diagnostics,

            emit_comments: false,
            emit_whitespace: false,

            token_start: 0,
            token_end: 0,

            start_line: 1,
            start_column: 1,
        }
    }

    pub fn pos(&self) -> TokenPosition<'s> {
        TokenPosition {
            absolute_position: self.token_start,
            line: self.start_line,
            column: self.start_column,
            text: &self.source.text[self.token_start..=self.token_end],
        }
    }

    fn next_char(&mut self) -> Option<char> {
        match self.chrs.next() {
            Some((p, c)) => {
                if c == '\n' {
                    self.newlines.push(p);
                }
                self.token_end = p;

                Some(c)
            }
            None => None,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chrs.peek().map(|(_, c)| *c)
    }

    fn eat(&mut self, v: char) -> bool {
        if self.peek_char() == Some(v) {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn consume(&mut self) {
        self.token_start = self.token_end + 1;
        self.token_end = self.token_start;

        self.start_line = self.newlines.len();
        self.start_column = self.token_end - self.newlines.last().unwrap();
    }

    fn number(&mut self, first_char: char) -> NumberLiteral {
        let mut int_part = first_char as u64 - '0' as u64;
        loop {
            match self.peek_char() {
                Some(c @ '0'..='9') => int_part = int_part * 10 + (c as u64 - '0' as u64),
                Some('_') => (),

                _ => break,
            }

            self.next_char();
        }

        if !self.eat('.') {
            return NumberLiteral::Integer(int_part);
        }

        // float

        let mut float_part = 0.;
        let mut pow = 0.1;
        loop {
            match self.peek_char() {
                Some(c @ '0'..='9') => {
                    float_part += (c as u8 - b'0') as f64 * pow;
                    pow *= 0.1;
                }

                Some('_') => (),

                _ => break,
            }

            self.next_char();
        }

        NumberLiteral::Real(int_part as f64 + float_part)
    }

    fn char_lit(&mut self) -> Result<char, TokenizerError> {
        let c = match self.next_char().ok_or(TokenizerError::UnfinishedChar)? {
            '\\' => self.peek_char().ok_or(TokenizerError::UnfinishedChar)?,
            c => c,
        };

        if self.eat('\'') {
            Ok(c)
        } else {
            Err(TokenizerError::UnfinishedChar)
        }
    }

    fn string_lit(&mut self) -> Result<String, TokenizerError> {
        let mut s = String::new();
        let mut escaped = false;

        loop {
            match self.peek_char().ok_or(TokenizerError::UnfinishedString)? {
                '"' if !escaped => break,
                '\\' if !escaped => escaped = true,

                c if escaped => {
                    if let Some(escaped) = escape(c) {
                        s.push_str(escaped);
                    } else {
                        s.push(c);
                    }

                    escaped = false;
                }

                c => s.push(c),
            }
        }

        Ok(s)
    }

    fn multiline_comment(&mut self) -> bool {
        let Some(mut pc) = self.next_char() else {
            return false;
        };

        let mut nest = 1usize;

        let mut closed = false;
        while let Some(c) = self.next_char() {
            match (pc, c) {
                ('*', '/') => {
                    nest -= 1;

                    if nest == 0 {
                        closed = true;
                        break;
                    }
                }

                ('/', '*') => nest += 1,

                _ => (),
            }

            pc = c;
        }

        closed
    }
    fn singleline_comment(&mut self) {
        while self.peek_char() != Some('\n') {
            self.next_char();
        }
    }

    fn get_token_inner(&mut self) -> Option<TokenizerItem<'s>> {
        let ty = match self.next_char()? {
            '(' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Left,
                ty: DelimeterType::Parentheses,
            }),
            ')' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Right,
                ty: DelimeterType::Parentheses,
            }),

            '[' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Left,
                ty: DelimeterType::Square,
            }),
            ']' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Right,
                ty: DelimeterType::Square,
            }),

            '{' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Left,
                ty: DelimeterType::Curly,
            }),
            '}' => Ty::Delimeter(Delimeter {
                side: DelimeterSide::Right,
                ty: DelimeterType::Curly,
            }),

            '@' => Ty::Punctuation(Punctuation::AtSign),
            ',' => Ty::Punctuation(Punctuation::Comma),
            ';' => Ty::Punctuation(Punctuation::Semicolon),
            ':' => Ty::Punctuation(Punctuation::Colon),
            '#' => Ty::Punctuation(Punctuation::HashSymbol),
            '?' => Ty::Punctuation(Punctuation::QuestionMark),
            '$' => {
                if self.eat('$') {
                    Ty::Punctuation(Punctuation::DoubleDollar)
                } else {
                    Ty::Punctuation(Punctuation::Dollar)
                }
            }

            '=' => match self.peek_char() {
                Some('>') => {
                    self.next_char();
                    Ty::Punctuation(Punctuation::FatArrow)
                }
                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::DoubleEquals)
                }

                _ => Ty::Operator(Operator::Equals),
            },

            '.' => {
                if self.eat('.') {
                    if self.eat('.') {
                        Ty::Punctuation(Punctuation::TripleDot)
                    } else {
                        Ty::Punctuation(Punctuation::DoubleDot)
                    }
                } else {
                    Ty::Punctuation(Punctuation::Dot)
                }
            }

            ' ' | '\t' | '\n' => {
                while matches!(self.peek_char(), Some(' ' | '\t' | '\n')) {
                    self.next_char();
                }

                Ty::Whitespace
            }

            'a'..='z' | 'A'..='Z' | '_' => {
                while matches!(
                    self.peek_char(),
                    Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9')
                ) {
                    self.next_char();
                }

                get_keyword(self.pos().text)
                    .map(Ty::Keyword)
                    .unwrap_or(Ty::Identifier)
            }

            c @ '0'..='9' => Ty::Literal(Literal::Number(self.number(c))),

            '\'' => Ty::Literal(Literal::Char(match self.char_lit() {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            })),

            '\"' => Ty::Literal(Literal::String(match self.string_lit() {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            })),

            '+' => match self.peek_char() {
                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::PlusEquals)
                }

                Some('+') => {
                    self.next_char();
                    Ty::Operator(Operator::DoublePlus)
                }
                _ => Ty::Operator(Operator::Plus),
            },

            '-' => match self.peek_char() {
                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::MinusEquals)
                }

                Some('-') => {
                    self.next_char();
                    Ty::Operator(Operator::DoubleMinus)
                }
                _ => Ty::Operator(Operator::Minus),
            },
            '*' => match self.peek_char() {
                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::StarEquals)
                }

                Some('*') => {
                    self.next_char();
                    if self.eat('=') {
                        Ty::Operator(Operator::DoubleStarEquals)
                    } else {
                        Ty::Operator(Operator::DoubleStar)
                    }
                }

                _ => Ty::Operator(Operator::Star),
            },

            '!' => {
                if self.eat('=') {
                    Ty::Operator(Operator::BangEquals)
                } else {
                    Ty::Operator(Operator::Bang)
                }
            }

            '~' => {
                if self.eat('=') {
                    Ty::Operator(Operator::TildaEquals)
                } else {
                    Ty::Operator(Operator::Tilda)
                }
            }

            '^' => {
                if self.eat('=') {
                    Ty::Operator(Operator::CaretEquals)
                } else {
                    Ty::Operator(Operator::Caret)
                }
            }

            '%' => {
                if self.eat('=') {
                    Ty::Operator(Operator::PercentEquals)
                } else {
                    Ty::Operator(Operator::Percent)
                }
            }

            '/' => match self.peek_char() {
                Some('*') => {
                    let closed = self.multiline_comment();

                    if !closed {
                        self.diagnostics.extend([Diagnostic::new(
                            DiagnosticType::UnclosedMultilineComment,
                            self.pos(),
                        )]);
                    }

                    Ty::Comment(Comment::MultiLine)
                }

                Some('/') => {
                    self.singleline_comment();
                    Ty::Comment(Comment::SingleLine)
                }

                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::SlashEquals)
                }

                _ => Ty::Operator(Operator::Slash),
            },

            '&' => match self.peek_char() {
                Some('&') => {
                    self.next_char();
                    if self.eat('=') {
                        Ty::Operator(Operator::DoubleAndEquals)
                    } else {
                        Ty::Operator(Operator::DoubleAnd)
                    }
                }

                Some('=') => {
                    self.next_char();
                    TokenType::Operator(Operator::SingleAndEquals)
                }

                _ => Ty::Operator(Operator::SingleAnd),
            },

            '|' => match self.peek_char() {
                Some('|') => {
                    self.next_char();
                    if self.eat('=') {
                        Ty::Operator(Operator::DoubleOrEquals)
                    } else {
                        Ty::Operator(Operator::DoubleOr)
                    }
                }

                Some('=') => {
                    self.next_char();
                    TokenType::Operator(Operator::SingleOrEquals)
                }

                _ => Ty::Operator(Operator::SingleOr),
            },

            '<' => match self.peek_char() {
                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::LesserThanEquals)
                }

                Some('<') => {
                    self.next_char();
                    if self.peek_char() == Some('=') {
                        self.next_char();
                        Ty::Operator(Operator::LeftShiftEquals)
                    } else {
                        Ty::Operator(Operator::LeftShift)
                    }
                }

                _ => Ty::Operator(Operator::LesserThan),
            },

            '>' => match self.peek_char() {
                Some('=') => {
                    self.next_char();
                    Ty::Operator(Operator::GreaterThanEquals)
                }

                Some('>') => {
                    self.next_char();
                    if self.peek_char() == Some('=') {
                        self.next_char();
                        Ty::Operator(Operator::RightShiftEquals)
                    } else {
                        Ty::Operator(Operator::RightShift)
                    }
                }

                _ => Ty::Operator(Operator::GreaterThan),
            },

            b => return Some(Err(TokenizerError::Unexpected(b))),
        };

        Some(Ok(Token {
            position: self.pos(),
            ty,
        }))
    }

    fn get_token(&mut self) -> Option<TokenizerItem<'s>> {
        loop {
            let t = self.get_token_inner();
            self.consume();

            match t {
                Some(Ok(Token {
                    ty: Ty::Whitespace, ..
                })) if !self.emit_whitespace => continue,

                Some(Ok(Token {
                    ty: Ty::Comment(_), ..
                })) if !self.emit_comments => continue,

                r => return r,
            };
        }
    }
}

pub type TokenizerItem<'s> = Result<Token<'s>, TokenizerError>;

impl<'n, 's, 'd, D: Extend<Diagnostic<'s>>> Iterator for Tokenizer<'n, 's, 'd, D> {
    type Item = TokenizerItem<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_token()
    }
}

fn escape(c: char) -> Option<&'static str> {
    Some(match c {
        'n' => "\n",
        't' => "\t",
        '0' => "\0",

        _ => return None,
    })
}
