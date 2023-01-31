use crate::lexer::Token;
use anyhow::{bail, Result};
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

pub fn parse(tokens: Vec<Token>) -> Result<SyntaxNode> {
    let items = create_parse_items(tokens);
    let items = parse_empty_set(items);
    let items = parse_variables(items);
    let items = parse_relations(items)?;
    todo!("{:?}", items)
}

fn create_parse_items(tokens: Vec<Token>) -> Vec<ParseItem> {
    tokens.into_iter().map(|t| ParseItem::Token(t)).collect()
}

fn parse_empty_set(items: Vec<ParseItem>) -> Vec<ParseItem> {
    items
        .into_iter()
        .map(|i| match i {
            ParseItem::Token(Token::Const(c)) => {
                if ["0", "∅", "\\emptyset"].contains(&c.as_str()) {
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
