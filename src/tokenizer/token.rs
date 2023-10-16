#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum DelimeterType {
    Parentheses,
    Square,
    Curly,
}

#[derive(Debug, Clone)]
pub enum DelimeterSide {
    Left,
    Right,
}
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum NumberLiteral {
    Integer(u64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub enum Comment {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
