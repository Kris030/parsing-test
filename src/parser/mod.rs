pub mod expression;

use crate::{tokenizer::*, Diagnostic};

use self::expression::{Expression as Expr, Path};
use Operator as Op;
use ParserError as ParsErr;
use TokenType as Ty;

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error(transparent)]
    Tokenizer(#[from] TokenizerError),

    #[error("Unexpected token {found:?} instead of {expected:?}")]
    Unexpected {
        found: Option<Ty>,
        expected: &'static str,
    },

    #[error("Unexpected end of expression")]
    UnexpectedEnd,
}

impl ParserError {
    pub fn unexpected(found: Option<Ty>, expected: &'static str) -> Self {
        Self::Unexpected { found, expected }
    }
}

/// based on: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
pub struct Parser<'s, 'd, T: Iterator<Item = TokenizerItem<'s>>, D: Extend<Diagnostic<'s>>> {
    tokenizer: std::iter::Peekable<T>,
    _diagnostics: &'d mut D,
}

impl<'s, 'd, T: Iterator<Item = TokenizerItem<'s>>, D: Extend<Diagnostic<'s>>>
    Parser<'s, 'd, T, D>
{
    pub fn new(tokenizer: T, diagnostics: &'d mut D) -> Self {
        Self {
            tokenizer: tokenizer.peekable(),
            _diagnostics: diagnostics,
        }
    }

    #[allow(unused)]
    fn next_token(&mut self) -> Result<Option<Token<'s>>, TokenizerError> {
        match self.tokenizer.next() {
            Some(Ok(t)) => Ok(Some(t)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
    #[allow(unused)]
    fn peek_token(&mut self) -> Result<Option<&Token<'s>>, TokenizerError> {
        match self.tokenizer.peek() {
            Some(Ok(t)) => Ok(Some(t)),
            Some(Err(e)) => Err(*e),
            None => Ok(None),
        }
    }

    #[allow(unused)]
    fn next_token_ty(&mut self) -> Result<Option<Ty>, TokenizerError> {
        Ok(self.next_token()?.map(|t| t.ty))
    }
    #[allow(unused)]
    fn peek_token_ty(&mut self) -> Result<Option<&Ty>, TokenizerError> {
        Ok(self.peek_token()?.map(|t| &t.ty))
    }

    #[allow(unused)]
    fn eat(&mut self, ty: Ty) -> Result<bool, ParsErr> {
        let Some(a) = self.peek_token_ty()? else {
            return Ok(false);
        };

        let ret = *a == ty;
        if ret {
            self.next_token()?;
        }

        Ok(ret)
    }

    fn expr_primary(&mut self) -> Result<Expr<'s>, ParsErr> {
        Ok(match self.next_token()? {
            Some(Token {
                ty: Ty::Literal(value),
                ..
            }) => Expr::Lit { value },

            Some(
                t @ Token {
                    ty: Ty::Identifier, ..
                },
            ) => match self.peek_token_ty()? {
                Some(Ty::Delimeter(Delimeter {
                    ty: DelimeterType::Parentheses,
                    side: DelimeterSide::Left,
                })) => {
                    self.next_token()?;
                    self.call_expr(Path::new(t.text(), None))?
                }

                Some(Ty::Punctuation(Punctuation::DoubleColon)) => {
                    self.next_token()?;
                    let mut tail = vec![];

                    while let Some(Token {
                        ty: Ty::Identifier, ..
                    }) = self.peek_token()?
                    {
                        tail.push(self.next_token()?.unwrap());

                        if let Some(Ty::Punctuation(Punctuation::DoubleColon)) =
                            self.peek_token_ty()?
                        {
                            self.next_token()?;
                        } else {
                            break;
                        }
                    }

                    let p = Path::new(t.text(), Some(tail));

                    match self.peek_token_ty()? {
                        Some(Ty::Delimeter(Delimeter {
                            ty: DelimeterType::Parentheses,
                            side: DelimeterSide::Left,
                        })) => {
                            self.next_token()?;
                            self.call_expr(p)?
                        }

                        _ => Expr::Name(p),
                    }
                }

                _ => Expr::Name(Path::new(t.text(), None)),
            },

            Some(Token {
                ty:
                    Ty::Delimeter(Delimeter {
                        ty: DelimeterType::Parentheses,
                        side: DelimeterSide::Left,
                    }),
                ..
            }) => {
                let lhs = self.expr_bp(0)?;

                match self.next_token_ty()? {
                    Some(Ty::Delimeter(Delimeter {
                        ty: DelimeterType::Parentheses,
                        side: DelimeterSide::Right,
                    })) => (),

                    found => {
                        return Err(ParserError::Unexpected {
                            found,
                            expected: "a matching right parenthesis",
                        })
                    }
                }

                lhs
            }

            Some(Token {
                ty: Ty::Operator(op),
                ..
            }) => {
                let Some(((), r_bp)) = prefix_binding_power(op) else {
                    todo!()
                };

                let rhs = self.expr_bp(r_bp)?;
                Expr::Prefix {
                    op,
                    right: Box::new(rhs),
                }
            }

            Some(t) => {
                return Err(ParsErr::unexpected(
                    Some(t.ty),
                    "a literal or an identifier",
                ))
            }

            None => return Err(ParsErr::UnexpectedEnd),
        })
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr<'s>, ParsErr> {
        let mut lhs = self.expr_primary()?;

        while let Some(
            op @ (Ty::Operator(_)
            | Ty::Delimeter(Delimeter {
                ty: DelimeterType::Square,
                ..
            })),
        ) = self.peek_token_ty()?
        {
            let op = op.clone();

            if let Some((l_bp, ())) = postfix_binding_power(&op) {
                if l_bp < min_bp {
                    break;
                }

                self.next_token()?;

                lhs = if matches!(
                    op,
                    Ty::Delimeter(Delimeter {
                        ty: DelimeterType::Square,
                        side: DelimeterSide::Left,
                    })
                ) {
                    let rhs = self.expr_bp(0)?;

                    match self.next_token_ty()? {
                        Some(Ty::Delimeter(Delimeter {
                            ty: DelimeterType::Square,
                            side: DelimeterSide::Right,
                        })) => (),

                        found => {
                            return Err(ParserError::Unexpected {
                                found,
                                expected: "a matching right square bracket",
                            })
                        }
                    }

                    Expr::Index {
                        expr: Box::new(lhs),
                        with: Box::new(rhs),
                    }
                } else {
                    Expr::Postfix {
                        left: Box::new(lhs),
                        op: if let Ty::Operator(op) = op {
                            op
                        } else {
                            unreachable!()
                        },
                    }
                };

                continue;
            };

            if let Some((l_bp, r_bp)) = infix_binding_power(&op) {
                if l_bp < min_bp {
                    break;
                }

                self.next_token()?;

                let rhs = self.expr_bp(r_bp)?;
                lhs = Expr::Infix {
                    left: Box::new(lhs),
                    op: if let Ty::Operator(op) = op {
                        op
                    } else {
                        unreachable!()
                    },

                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub fn expr(&mut self) -> Result<Expr<'s>, ParserError> {
        self.expr_bp(0)
    }

    fn call_expr(&mut self, function: Path<'s>) -> Result<Expr<'s>, ParsErr> {
        let mut args = vec![];

        loop {
            let arg = self.expr()?;
            args.push(arg);

            match self.peek_token_ty()? {
                Some(Ty::Delimeter(Delimeter {
                    ty: DelimeterType::Parentheses,
                    side: DelimeterSide::Right,
                })) => {
                    self.next_token()?;
                    break;
                }

                Some(Ty::Punctuation(Punctuation::Comma)) => {
                    self.next_token()?;
                    continue;
                }

                ty => {
                    return Err(ParserError::Unexpected {
                        found: ty.cloned(),
                        expected:
                            "a comma after the argument, or a parenthesis closing the agument list",
                    })
                }
            }
        }

        Ok(Expr::Call {
            function,
            arguments: args,
        })
    }
}

fn prefix_binding_power(op: Operator) -> Option<((), u8)> {
    match op {
        Op::Plus | Op::Minus => Some(((), 5)),
        Op::DoublePlus | Op::DoubleMinus => Some(((), 5)),
        _ => None,
    }
}

fn infix_binding_power(op: &Ty) -> Option<(u8, u8)> {
    let Ty::Operator(op) = op else { return None };

    Some(match op {
        Op::Plus | Op::Minus => (1, 2),
        Op::Star | Op::Slash => (3, 4),

        Op::DoublePlus => todo!(),
        Op::DoubleMinus => todo!(),
        Op::BangEquals => todo!(),
        Op::RightShift => todo!(),
        Op::LesserThan => todo!(),
        Op::DoubleStar => todo!(),
        Op::DoubleAnd => todo!(),
        Op::DoubleOr => todo!(),
        Op::DoubleEquals => todo!(),
        Op::GreaterThan => todo!(),
        Op::Caret => todo!(),
        Op::Percent => todo!(),
        Op::SingleAnd => todo!(),
        Op::SingleOr => todo!(),
        Op::LeftShift => todo!(),
        Op::Equals => todo!(),
        Op::PlusEquals => todo!(),
        Op::MinusEquals => todo!(),
        Op::StarEquals => todo!(),
        Op::SlashEquals => todo!(),
        Op::TildaEquals => todo!(),
        Op::DoubleStarEquals => todo!(),
        Op::DoubleAndEquals => todo!(),
        Op::DoubleOrEquals => todo!(),
        Op::LesserThanEquals => todo!(),
        Op::GreaterThanEquals => todo!(),
        Op::CaretEquals => todo!(),
        Op::PercentEquals => todo!(),
        Op::SingleAndEquals => todo!(),
        Op::SingleOrEquals => todo!(),
        Op::LeftShiftEquals => todo!(),
        Op::RightShiftEquals => todo!(),

        _ => return None,
    })
}

fn postfix_binding_power(op: &Ty) -> Option<(u8, ())> {
    Some(match op {
        Ty::Operator(Op::Bang)
        | Ty::Delimeter(Delimeter {
            ty: DelimeterType::Square,
            side: DelimeterSide::Left,
        }) => (7, ()),

        _ => return None,
    })
}
