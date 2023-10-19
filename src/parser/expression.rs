use std::fmt::{Debug, Display};

use crate::tokenizer::{Literal, Operator, Token};

#[derive(Debug, Clone)]
pub struct Path<'s> {
    pub(crate) head: &'s str,
    pub(crate) tail: Option<Vec<Token<'s>>>,
}

impl<'s> Path<'s> {
    pub fn new(head: &'s str, tail: Option<Vec<Token<'s>>>) -> Self {
        Self { head, tail }
    }
}

impl Display for Path<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.head)?;
        let Some(tail) = &self.tail else {
            return Ok(());
        };

        for n in tail {
            write!(f, "::{}", n.text())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Expression<'s> {
    Prefix {
        op: Operator,
        right: Box<Expression<'s>>,
    },

    Infix {
        left: Box<Expression<'s>>,
        op: Operator,
        right: Box<Expression<'s>>,
    },

    Postfix {
        left: Box<Expression<'s>>,
        op: Operator,
    },

    Call {
        function: Path<'s>,
        arguments: Vec<Expression<'s>>,
    },

    Name(Path<'s>),

    Lit {
        value: Literal,
    },

    Index {
        expr: Box<Expression<'s>>,
        with: Box<Expression<'s>>,
    },
}
impl Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(p) => write!(f, "{p}"),
            Self::Lit { value } => write!(f, "{value}"),
            Self::Prefix { op, right } => write!(f, "({op}{right})"),
            Self::Infix { left, op, right } => write!(f, "({left} {op} {right})"),
            Self::Index { expr, with } => write!(f, "({expr}[{with}])"),
            Self::Postfix { left, op } => write!(f, "({left}{op})"),
            Self::Call {
                function,
                arguments,
            } => {
                write!(f, "({function}(")?;

                let mut args = arguments.iter();
                if let Some(a) = args.next() {
                    write!(f, "{a}")?;
                }

                for a in args {
                    write!(f, ", {a}")?;
                }

                write!(f, "))")
            }
        }
    }
}
