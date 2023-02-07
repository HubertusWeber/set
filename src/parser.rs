use crate::lexer::Token;
use anyhow::{bail, ensure, Result};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
enum ParseItem {
    Token(Token),
    SyntaxNode(SyntaxNode),
}

#[derive(Debug, Clone)]
pub struct SyntaxNode {
    entry: NodeType,
    children: Vec<SyntaxNode>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    Relation(Relation),
    Connective(Connective),
    Quantifier(Quantifier),
    Operator(Operator),
    Variable(u32),
    Comprehension,
    EmptySet,
}

#[derive(Debug, Clone, Copy)]
pub enum Relation {
    Element,
    Equality,
}

#[derive(Debug, Clone, Copy)]
pub enum Connective {
    Negation,
    Conjunction,
    Disjunction,
    Implication,
    Biconditional,
}

#[derive(Debug, Clone, Copy)]
pub enum Quantifier {
    Universal,
    Existential,
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    PowerSet,
}

#[derive(Debug, Clone, Copy)]
struct Depth {
    val: usize,
    pos: usize,
}

pub fn parse(tokens: Vec<Token>) -> Result<SyntaxNode> {
    let items = create_parse_items(tokens);
    let items = parse_empty_set(items);
    let items = parse_variables(items);
    let items = parse_relations(items)?;
    todo!("{:?}", max_depth(&items))
}

fn create_parse_items(tokens: Vec<Token>) -> Vec<ParseItem> {
    tokens.into_iter().map(|t| ParseItem::Token(t)).collect()
}

fn parse_empty_set(items: Vec<ParseItem>) -> Vec<ParseItem> {
    items
        .into_iter()
        .map(|i| match i {
            ParseItem::Token(Token::Const(c)) => {
                if matches!(c.as_str(), "0" | "∅" | "\\emptyset") {
                    ParseItem::SyntaxNode(SyntaxNode {
                        entry: NodeType::EmptySet,
                        children: vec![],
                    })
                } else {
                    unreachable!()
                }
            }
            i => i,
        })
        .collect()
}

fn parse_variables(items: Vec<ParseItem>) -> Vec<ParseItem> {
    let used_indices = std::cell::RefCell::new(HashSet::<u32>::new());
    let mut index_map = HashMap::<String, u32>::new();
    items
        .into_iter()
        .map(|i| match i {
            ParseItem::Token(Token::Var(v)) => {
                if v.starts_with(&v) && v.len() > 1 {
                    let index = v[1..].parse().unwrap();
                    used_indices.borrow_mut().insert(index);
                    ParseItem::SyntaxNode(SyntaxNode {
                        entry: NodeType::Variable(index),
                        children: vec![],
                    })
                } else {
                    ParseItem::Token(Token::Var(v))
                }
            }
            i => i,
        })
        .collect::<Vec<ParseItem>>()
        .into_iter()
        .map(|i| match i {
            ParseItem::Token(Token::Var(v)) => ParseItem::SyntaxNode(SyntaxNode {
                entry: NodeType::Variable(if index_map.contains_key(&v) {
                    *index_map.get(&v).unwrap()
                } else {
                    let index = (0..)
                        .into_iter()
                        .find_map(|n| {
                            if used_indices.borrow_mut().insert(n) {
                                Some(n)
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    index_map.insert(v, index);
                    index
                }),
                children: vec![],
            }),
            i => i,
        })
        .collect()
}

fn parse_relations(mut items: Vec<ParseItem>) -> Result<Vec<ParseItem>> {
    'outer: loop {
        for i in 0..items.len() {
            if let ParseItem::Token(Token::Rel(rel)) = &items[i] {
                if i == 0 || i + 1 == items.len() {
                    bail!("Found binary relation '{}' at edge of the formula", rel);
                }
                if let (ParseItem::SyntaxNode(l), ParseItem::SyntaxNode(r)) =
                    (&items[i - 1], &items[i + 1])
                {
                    if !(matches!(
                        l.entry,
                        NodeType::EmptySet | NodeType::Comprehension | NodeType::Variable(_)
                    ) && matches!(
                        r.entry,
                        NodeType::EmptySet | NodeType::Comprehension | NodeType::Variable(_)
                    )) {
                        bail!(
                            "Invalid relata {:?} and {:?} for relation '{}'",
                            l.entry,
                            r.entry,
                            rel
                        );
                    }
                    let node = match rel.as_str() {
                        "=" => SyntaxNode {
                            entry: NodeType::Relation(Relation::Equality),
                            children: vec![l.to_owned(), r.to_owned()],
                        },
                        "∈" | "\\epsilon" => SyntaxNode {
                            entry: NodeType::Relation(Relation::Element),
                            children: vec![l.to_owned(), r.to_owned()],
                        },
                        x => unimplemented!("Relation token '{}' not implemented in parser", x),
                    };
                    items.remove(i + 1);
                    items.remove(i - 1);
                    items[i - 1] = ParseItem::SyntaxNode(node);
                    continue 'outer;
                }
            }
        }
        break Ok(items);
    }
}

fn parse_at(items: Vec<ParseItem>, pos: usize) -> Result<Vec<ParseItem>> {
    ensure!(pos < items.len(), "Unexpected end of input");
    match &items[pos] {
        ParseItem::SyntaxNode(..) => Ok(items),
        ParseItem::Token(Token::Quan(..)) => parse_quan_at(items, pos),
        ParseItem::Token(Token::Paren(p)) => match p.as_str() {
            "(" => parse_conn_at(items, pos),
            "{" => parse_relations(parse_comp_at(items, pos)?),
            x => bail!("Unexpected token '{}'", x),
        },
        ParseItem::Token(Token::Conn(c)) => match c.as_str() {
            "¬" | "!" | "\\lnot" => parse_neg_at(items, pos),
            x => bail!("Unexpected token '{}'", x),
        },
        ParseItem::Token(x) => bail!("Unexpected token {:?}", x),
    }
}

fn parse_quan_at(mut items: Vec<ParseItem>, pos: usize) -> Result<Vec<ParseItem>> {
    ensure!(pos + 2 < items.len(), "Unexpected end of input");
    items = parse_at(items, pos + 2)?;
    let formula = items.remove(pos + 2);
    let var = items.remove(pos + 1);
    if let (ParseItem::SyntaxNode(formula), ParseItem::SyntaxNode(var)) = (formula, var) {
        ensure!(matches!(var.entry, NodeType::Variable(..)));
        if let ParseItem::Token(Token::Quan(q)) = &items[pos] {
            let quan = SyntaxNode {
                entry: match q.as_str() {
                    "∀" | "\\forall" => NodeType::Quantifier(Quantifier::Universal),
                    "∃" | "\\exists" => NodeType::Quantifier(Quantifier::Existential),
                    x => unimplemented!("Quantifier token '{}' not implemented in parser", x),
                },
                children: vec![var, formula],
            };
            items[pos] = ParseItem::SyntaxNode(quan);
            Ok(items)
        } else {
            unreachable!("Called parse_quan_at with a position not containing a qunatifier token")
        }
    } else {
        unreachable!("Found unparsed token(s) after quantifier")
    }
}
fn parse_conn_at(mut items: Vec<ParseItem>, pos: usize) -> Result<Vec<ParseItem>> {
    ensure!(pos + 1 < items.len(), "Unexpected end of input");
    items = parse_at(items, pos + 1)?;
    ensure!(pos + 3 < items.len(), "Unexpected end of input");
    items = parse_at(items, pos + 3)?;
    ensure!(pos + 4 < items.len(), "Unexpected end of input");
    ensure!(
        matches!(items.remove(pos + 4), ParseItem::Token(Token::Paren(p)) if p.as_str() == ")"),
        "Mising token ')'"
    );
    ensure!(
        matches!(items.remove(pos), ParseItem::Token(Token::Paren(p)) if p.as_str() == "("),
        "Missing token '('"
    );
    if let (ParseItem::SyntaxNode(left), ParseItem::SyntaxNode(right)) =
        (items.remove(pos), items.remove(pos + 1))
    {
        let children = vec![left, right];
        let entry = NodeType::Connective(match &items[pos] {
            ParseItem::Token(Token::Conn(c)) => match c.as_str() {
                "∧" | "&&" | "\\land" => Connective::Conjunction,
                "∨" | "||" | "\\lor" => Connective::Disjunction,
                "→" | "->" | "\\rightarrow" => Connective::Implication,
                "↔" | "<->" | "\\leftrightarrow" => Connective::Biconditional,
                "¬" | "!" | "\\lnot" => {
                    bail!("Unexpected negation token, expected binary connective")
                }
                x => bail!("Unexpected token '{}', expected binary connective", x),
            },
            x => bail!("Unexpected parse item {:?}, expected connective token", x),
        });
        items[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        Ok(items)
    } else {
        unreachable!("Found Token after calling parse_at")
    }
}
fn parse_neg_at(items: Vec<ParseItem>, pos: usize) -> Result<Vec<ParseItem>> {
    todo!()
}
fn parse_comp_at(items: Vec<ParseItem>, pos: usize) -> Result<Vec<ParseItem>> {
    todo!()
}

fn max_depth(items: &Vec<ParseItem>) -> Result<Depth> {
    let mut parens = Vec::<char>::new();
    let mut max_depth = Depth { val: 0, pos: 0 };
    for (pos, i) in items.iter().enumerate() {
        if let ParseItem::Token(Token::Paren(p)) = i {
            if p == "(" || p == "{" {
                parens.push(p.chars().next().unwrap());
                if parens.len() > max_depth.val {
                    max_depth.val = parens.len();
                    max_depth.pos = pos;
                }
            } else if p == ")" {
                if parens.pop() != Some('(') {
                    bail!("Unexpected token ')'");
                };
            } else if p == "}" {
                if parens.pop() != Some('{') {
                    bail!("Unexpected token '}}'");
                };
            }
        }
    }
    if let Some(p) = parens.pop() {
        bail!("Unclosed parenthesis '{}'", p)
    } else {
        Ok(max_depth)
    }
}
