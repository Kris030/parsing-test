use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct Delimeter {
    pub(crate) ty: DelimeterType,
    pub(crate) side: DelimeterSide,
}

impl Delimeter {
    pub fn ty(&self) -> &DelimeterType {
        &self.ty
    }

    pub fn side(&self) -> &DelimeterSide {
        &self.side
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DelimeterType {
    Parentheses,
    Square,
    Curly,
}

#[derive(Debug, Clone, Copy)]
pub enum DelimeterSide {
    Left,
    Right,
}
#[derive(Debug, Clone, Copy)]
pub enum Punctuation {
    Semicolon,
    Comma,
    Colon,
    FatArrow,
    Dot,
    DoubleDot,
    TripleDot,
    HashSymbol,
    AtSign,
    QuestionMark,
    Dollar,
    DoubleDollar,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(NumberLiteral),
    String(String),
    Char(char),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{n}"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Char(c) => write!(f, "'{c}'"),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum NumberLiteral {
    Integer(u64),
    Real(f64),
}

impl From<u64> for NumberLiteral {
    fn from(value: u64) -> Self {
        Self::Integer(value)
    }
}
impl From<f64> for NumberLiteral {
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}

impl From<NumberLiteral> for u64 {
    fn from(value: NumberLiteral) -> u64 {
        match value {
            NumberLiteral::Integer(i) => i,
            NumberLiteral::Real(f) => f as u64,
        }
    }
}
impl From<NumberLiteral> for f64 {
    fn from(value: NumberLiteral) -> f64 {
        match value {
            NumberLiteral::Integer(i) => i as f64,
            NumberLiteral::Real(f) => f,
        }
    }
}

impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberLiteral::Integer(i) => write!(f, "{i}"),
            NumberLiteral::Real(r) => write!(f, "{r}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Comment {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,

    // unary
    DoublePlus,
    DoubleMinus,
    Tilda,
    Bang,
    // unary

    // binary
    Star,
    Slash,
    BangEquals,
    RightShift,
    LesserThan,
    DoubleStar,
    DoubleAnd,
    DoubleOr,
    DoubleEquals,
    GreaterThan,
    Caret,
    Percent,
    SingleAnd,
    SingleOr,
    LeftShift,
    // binary

    // assignment
    Equals,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    TildaEquals,
    DoubleStarEquals,
    DoubleAndEquals,
    DoubleOrEquals,
    LesserThanEquals,
    GreaterThanEquals,
    CaretEquals,
    PercentEquals,
    SingleAndEquals,
    SingleOrEquals,
    LeftShiftEquals,
    RightShiftEquals,
    // assignment
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::DoublePlus => write!(f, "++"),
            Self::DoubleMinus => write!(f, "--"),
            Self::Tilda => write!(f, "~"),
            Self::Bang => write!(f, "!"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::BangEquals => write!(f, "!="),
            Self::RightShift => write!(f, ">>"),
            Self::LesserThan => write!(f, "<"),
            Self::DoubleStar => write!(f, "**"),
            Self::DoubleAnd => write!(f, "&&"),
            Self::DoubleOr => write!(f, "||"),
            Self::DoubleEquals => write!(f, "=="),
            Self::GreaterThan => write!(f, ">"),
            Self::Caret => write!(f, "^"),
            Self::Percent => write!(f, "%"),
            Self::SingleAnd => write!(f, "&"),
            Self::SingleOr => write!(f, "|"),
            Self::LeftShift => write!(f, "<<"),
            Self::Equals => write!(f, "="),
            Self::PlusEquals => write!(f, "+="),
            Self::MinusEquals => write!(f, "-="),
            Self::StarEquals => write!(f, "*="),
            Self::SlashEquals => write!(f, "/="),
            Self::TildaEquals => write!(f, "~="),
            Self::DoubleStarEquals => write!(f, "**="),
            Self::DoubleAndEquals => write!(f, "&&="),
            Self::DoubleOrEquals => write!(f, "||="),
            Self::LesserThanEquals => write!(f, "<="),
            Self::GreaterThanEquals => write!(f, ">="),
            Self::CaretEquals => write!(f, "^="),
            Self::PercentEquals => write!(f, "%="),
            Self::SingleAndEquals => write!(f, "&="),
            Self::SingleOrEquals => write!(f, "|="),
            Self::LeftShiftEquals => write!(f, "<<="),
            Self::RightShiftEquals => write!(f, ">>="),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    Underscore,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Identifier,
    Whitespace,

    Punctuation(Punctuation),
    Delimeter(Delimeter),
    Literal(Literal),
    Comment(Comment),
    Operator(Operator),
    Keyword(Keyword),
}

#[derive(Debug, Clone)]
pub struct Token<'s> {
    pub(crate) position: TokenPosition<'s>,
    pub(crate) ty: TokenType,
}

impl<'s> Token<'s> {
    pub fn position(&self) -> &TokenPosition<'s> {
        &self.position
    }

    pub fn ty(&self) -> &TokenType {
        &self.ty
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TokenPosition<'s> {
    pub(crate) absolute_position: usize,

    pub(crate) line: usize,
    pub(crate) column: usize,

    pub(crate) text: &'s str,
}

impl<'s> TokenPosition<'s> {
    pub fn absolute_position(&self) -> usize {
        self.absolute_position
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

pub(crate) fn get_keyword(s: &str) -> Option<Keyword> {
    Some(match s {
        "_" => Keyword::Underscore,
        _ => return None,
    })
}
