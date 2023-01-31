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
