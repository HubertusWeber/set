use anyhow::bail;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum Token {
    Paren(String),
    Conn(String),
    Quan(String),
    Pred(String),
    Var(String),
}

const PAREN: &'static [&'static str; 2] = &["(", ")"];
const CONN: &'static [&'static str; 15] = &[
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
const QUAN: &'static [&'static str; 4] = &["∀", "∃", "\\forall", "\\exists"];
const PRED: &'static [&'static str; 2] = &["∈", "\\epsilon"];

pub fn tokanize(mut input: String) -> Result<Vec<Token>> {
    let mut result = vec![];
    input = input.split_whitespace().collect();
    'outer: while !input.is_empty() {
        for p in PAREN {
            if input.starts_with(p) {
                result.push(Token::Paren(input.drain(..p.len()).collect()));
                continue 'outer;
            }
        }
        for p in CONN {
            if input.starts_with(p) {
                result.push(Token::Conn(input.drain(..p.len()).collect()));
                continue 'outer;
            }
        }
        for p in QUAN {
            if input.starts_with(p) {
                result.push(Token::Quan(input.drain(..p.len()).collect()));
                continue 'outer;
            }
        }
        for p in PRED {
            if input.starts_with(p) {
                result.push(Token::Pred(input.drain(..p.len()).collect()));
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
