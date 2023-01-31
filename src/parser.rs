use crate::lexer::Token;
use anyhow::{bail, Result};

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

    todo!("{:?}", items)
}

fn create_parse_items(tokens: Vec<Token>) -> Vec<ParseItem> {
    tokens.into_iter().map(|x| ParseItem::Token(x)).collect()
}

fn parse_empty_set(items: Vec<ParseItem>) -> Vec<ParseItem> {
    items
        .into_iter()
        .map(|x| match x {
            ParseItem::Token(Token::Const(y)) => {
                if ["0", "∅", "\\emptyset"].contains(&y.as_str()) {
                    ParseItem::SyntaxNode(SyntaxNode {
                        entry: NodeType::EmptySet,
                        children: vec![],
                    })
                } else {
                    unreachable!()
                }
            }
            z => z,
        })
        .collect()
}
