use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum Token {
    Brack(String),
    Rel(String),
    Conn(String),
    Quan(String),
    UnOp(String),
    BinOp(String),
    Var(String),
    Const(String),
}

const REL: &'static [&'static str] = &[
    "=",
    "∈",
    "\\in",
    "⊆",
    "\\subseteq",
    "≠",
    "!=",
    "\\neq",
    "∉",
    "\\notin",
    "⊈",
    "\\nsubseteq",
];
const CONN: &'static [&'static str] = &[
    "¬",
    "∧",
    "∨",
    "→",
    "↔",
    "!",
    "&&",
    "||",
    "->",
    "<->",
    "\\lnot",
    "\\land",
    "\\lor",
    "\\rightarrow",
    "\\leftrightarrow",
];
const BRACK: &'static [&'static str] = &["(", ")", "{", "}", "|", ","];
const CONST: &'static [&'static str] = &["0", "∅", "\\emptyset", "ω", "\\omega"];
const QUAN: &'static [&'static str] = &["∀", "∃", "\\forall", "\\exists"];
const UNOP: &'static [&'static str] =
    &["Pot", "Vereinigung", "\\bigcup", "Durchschnitt", "\\bigcap"];
const BINOP: &'static [&'static str] = &["∪", "\\cup", "∩", "\\cap", "\\"];

pub fn tokanize(mut input: String) -> Result<Vec<Token>> {
    let mut result = vec![];
    input = input.split_whitespace().collect();
    'outer: while !input.is_empty() {
        for x in REL {
            if input.starts_with(x) {
                result.push(Token::Rel(input.drain(..x.len()).collect()));
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
        for x in BRACK {
            if input.starts_with(x) {
                result.push(Token::Brack(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in CONST {
            if input.starts_with(x) {
                result.push(Token::Const(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in UNOP {
            if input.starts_with(x) {
                result.push(Token::UnOp(input.drain(..x.len()).collect()));
                continue 'outer;
            }
        }
        for x in BINOP {
            if input.starts_with(x) {
                result.push(Token::BinOp(input.drain(..x.len()).collect()));
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
        if input.chars().next().unwrap().is_ascii() {
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
