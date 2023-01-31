#![allow(dead_code)]

use anyhow::Result;

mod lexer;

pub fn run() -> Result<()> {
    let tokens = lexer::tokanize("\\forallx(v0 -> x)".into())?;
    println!("{:?}", tokens);
    Ok(())
}
