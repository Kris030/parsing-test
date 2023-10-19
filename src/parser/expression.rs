use std::fmt::Debug;

use crate::tokenizer::{Literal, Operator, Token};

#[derive(Clone)]
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
        function_name: &'s str,
        arguments: Vec<Expression<'s>>,
    },

    Var(Token<'s>),
    Lit {
        value: Literal,
    },

    Index {
        expr: Box<Expression<'s>>,
        with: Box<Expression<'s>>,
    },
}
impl Debug for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(t) => write!(f, "{}", t.position.text),
            Self::Lit { value } => write!(f, "{value}"),
            Self::Prefix { op, right } => write!(f, "({op}{right:?})"),
            Self::Infix { left, op, right } => write!(f, "({left:?} {op} {right:?})"),
            Self::Index { expr, with } => write!(f, "({expr:?}[{with:?}])"),
            Self::Postfix { left, op } => write!(f, "({left:?}{op})"),
            Self::Call {
                function_name,
                arguments,
            } => {
                write!(f, "({function_name}(")?;

                let mut args = arguments.iter();
                if let Some(a) = args.next() {
                    write!(f, "{a:?}")?;
                }
                for a in args {
                    write!(f, ", {a:?}")?;
                }

                write!(f, "))")
            }
        }
    }
}
