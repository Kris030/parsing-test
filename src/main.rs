use tokenizer::TokenPosition;

use crate::tokenizer::Tokenizer;

pub mod parser;
pub mod tokenizer;

fn main() -> anyhow::Result<()> {
    let file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test/01.txt".to_string());
    let text = std::fs::read_to_string(&file)?;
    let source = Source::new(&file, &text);

    // print_tokens(source);

    let mut parser_diagnostics = vec![];
    let mut tokenizer_diagnostics = vec![];
    let tokenizer = Tokenizer::new(source, &mut tokenizer_diagnostics);
    let mut parser = parser::Parser::new(tokenizer, &mut parser_diagnostics);

    match parser.expr() {
        Ok(expr) => println!("\nparsed: {expr}\n"),
        Err(e) => println!("\nparser error: {e}\n"),
    }

    println!("tokenizer diagnostics: {:?}", tokenizer_diagnostics);
    println!("parser diagnostics: {:?}", parser_diagnostics);

    Ok(())
}

pub fn print_tokens(source: Source<'_, '_>) {
    for t in Tokenizer::new(source, &mut vec![]) {
        let comment_or_whitespace = matches!(
            t,
            Ok(tokenizer::Token {
                ty: tokenizer::TokenType::Whitespace,
                ..
            })
        );

        if !comment_or_whitespace {
            println!("{t:?}");
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Source<'n, 's> {
    pub(crate) name: &'n str,
    pub(crate) text: &'s str,
}

impl<'n, 's> Source<'n, 's> {
    pub fn new(name: &'n str, text: &'s str) -> Self {
        Self { name, text }
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
