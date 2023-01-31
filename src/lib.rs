#![allow(dead_code)]

use anyhow::Result;

mod lexer;
mod parser;

pub fn run() -> Result<()> {
    let tokens = lexer::tokanize("\\forallx(v0 -> 0)".into())?;
    let syntax_tree = parser::parse(tokens);
    println!("{:?}", syntax_tree);
    Ok(())
}
