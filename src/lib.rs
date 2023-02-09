#![allow(dead_code)]

use std::{fs::File, io::BufRead, io::BufReader};

use anyhow::Result;

mod display;
mod lexer;
mod parser;

pub fn run() -> Result<()> {
    let input_file = File::open("input.txt")?;
    let input_reader = BufReader::new(input_file);
    for line in input_reader.lines() {
        let tokens = lexer::tokanize(line?)?;
        let syntax_tree = parser::parse(tokens)?;
        println!("{}", syntax_tree);
    }
    Ok(())
}
