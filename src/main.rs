use tokenizer::TokenPosition;

use crate::tokenizer::{Token, TokenType as Ty};

pub mod tokenizer;

fn main() -> anyhow::Result<()> {
    let file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test/01.txt".to_string());
    let text = std::fs::read_to_string(&file)?;
    let source = Source::new(&file, &text);

    let mut diagnostics = vec![];
    let tokenizer = tokenizer::Tokenizer::new(source, &mut diagnostics);

    for t in tokenizer {
        let comment_or_whitespace = matches!(
            t,
            Ok(Token {
                ty: Ty::Whitespace,
                ..
            })
        );

        if !comment_or_whitespace {
            println!("{t:?}");
        }
    }

    println!("diagnostics: {:?}", diagnostics);

    Ok(())
}

pub struct Source<'n, 's> {
    pub(crate) name: &'n str,
    pub(crate) text: &'s str,
    pub(crate) bin: &'s [u8],
}

impl<'n, 's> Source<'n, 's> {
    pub fn new(name: &'n str, text: &'s str) -> Self {
        Self {
            name,
            text,
            bin: text.as_bytes(),
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn text(&self) -> &str {
        self.text
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic<'s> {
    ty: DiagnosticType,
    position: TokenPosition<'s>,
}
impl<'s> Diagnostic<'s> {
    pub fn new(ty: DiagnosticType, position: TokenPosition<'s>) -> Self {
        Self { ty, position }
    }

    pub fn ty(&self) -> &DiagnosticType {
        &self.ty
    }

    pub fn position(&self) -> &TokenPosition<'s> {
        &self.position
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub enum DiagnosticType {
    UnclosedMultilineComment,
}

impl DiagnosticType {
    pub fn level(&self) -> DiagnosticLevel {
        match self {
            DiagnosticType::UnclosedMultilineComment => DiagnosticLevel::Info,
        }
    }
}
