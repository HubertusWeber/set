#![allow(dead_code)]

use anyhow::Result;

mod display;
mod lexer;
mod parser;

pub fn run() -> Result<()> {
    let input = "\\forall x (x = 0 -> {z \\epsilon x | ! z = 0}\\epsilon v1)";
    println!("{}", input);
    let tokens = lexer::tokanize(input.into())?;
    println!("{:?}", tokens);
    let syntax_tree = parser::parse(tokens)?;
    println!("{}", syntax_tree);
    Ok(())
}
