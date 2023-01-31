use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum Token {
    Paren(String),
    Rel(String),
    Conn(String),
    Quan(String),
    Op(String),
    Var(String),
    Const(String),
}

const PAREN: &'static [&'static str] = &["(", ")", "{", "}"];
const CONN: &'static [&'static str] = &[
    "¬",
    "∧",
    "∨",
    "→",
    "↔",
    "~",
    "&",
    "|",
    "->",
    "<->",
    "\\lnot",
    "\\land",
    "\\lor",
    "\\rightarrow",
    "\\leftrightarrow",
];
const QUAN: &'static [&'static str] = &["∀", "∃", "\\forall", "\\exists"];
const REL: &'static [&'static str] = &["=", "∈", "\\epsilon"];
const OP: &'static [&'static str] = &["Pot"];
const CONST: &'static [&'static str] = &["0", "∅", "\\emptyset"];

pub fn tokanize(mut input: String) -> Result<Vec<Token>> {
    let mut result = vec![];
    input = input.split_whitespace().collect();
    'outer: while !input.is_empty() {
        for x in PAREN {
            if input.starts_with(x) {
                result.push(Token::Paren(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in CONN {
            if input.starts_with(x) {
                result.push(Token::Conn(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in QUAN {
            if input.starts_with(x) {
                result.push(Token::Quan(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in REL {
            if input.starts_with(x) {
                result.push(Token::Rel(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in OP {
            if input.starts_with(x) {
                result.push(Token::Op(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in CONST {
            if input.starts_with(x) {
                result.push(Token::Const(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        if input.chars().next().unwrap() == 'v' {
            let mut var_str: String = input.remove(0).into();
            while !input.is_empty() {
                if input.chars().next().unwrap().is_numeric() {
                    var_str.push(input.remove(0));
                } else {
                    break;
                }
            }
            result.push(Token::Var(var_str));
            continue 'outer;
        }
        if input.chars().next().unwrap().is_ascii_lowercase() {
            result.push(Token::Var(input.remove(0).into()));
            continue 'outer;
        }
        bail!(
            "Unexpected character '{}' in input string.",
            input.chars().next().unwrap()
        );
    }
    Ok(result)
}
