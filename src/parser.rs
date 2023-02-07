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

trait Parsable
where
    Self: Sized,
{
    fn parse_consts(self) -> Self;
    fn parse_vars(self) -> Self;
    fn parse_at(self, pos: usize) -> Result<Self>;
    fn parse_rel_at(self, pos: usize) -> Result<Self>;
    fn parse_quan_at(self, pos: usize) -> Result<Self>;
    fn parse_conn_at(self, pos: usize) -> Result<Self>;
    fn parse_neg_at(self, pos: usize) -> Result<Self>;
    fn parse_comp_at(self, pos: usize) -> Result<Self>;
}

impl Parsable for Vec<ParseItem> {
    fn parse_consts(self) -> Self {
        self.into_iter()
            .map(|i| match i {
                ParseItem::Token(Token::Const(c)) => match c.as_str() {
                    "0" | "∅" | "\\emptyset" => {
                        let entry = NodeType::EmptySet;
                        let children = vec![];
                        ParseItem::SyntaxNode(SyntaxNode { entry, children })
                    }
                    x => unimplemented!("Parser for constant '{}' not implemented", x),
                },
                i => i,
            })
            .collect()
    }

    fn parse_vars(self) -> Self {
        let used_indices = std::cell::RefCell::new(HashSet::<u32>::new());
        let mut index_map = HashMap::<String, u32>::new();
        self.into_iter()
            .map(|i| match i {
                ParseItem::Token(Token::Var(v)) => {
                    if v.starts_with(&v) && v.len() > 1 {
                        let index = v[1..].parse().unwrap();
                        used_indices.borrow_mut().insert(index);
                        let entry = NodeType::Variable(index);
                        let children = vec![];
                        ParseItem::SyntaxNode(SyntaxNode { entry, children })
                    } else {
                        ParseItem::Token(Token::Var(v))
                    }
                }
                i => i,
            })
            .collect::<Vec<ParseItem>>()
            .into_iter()
            .map(|i| match i {
                ParseItem::Token(Token::Var(v)) => {
                    let entry = NodeType::Variable(if index_map.contains_key(&v) {
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
                    });
                    let children = vec![];
                    ParseItem::SyntaxNode(SyntaxNode { entry, children })
                }
                i => i,
            })
            .collect()
    }

    fn parse_at(self, pos: usize) -> Result<Self> {
        ensure!(pos < self.len(), "Unexpected end of input");
        match &self[pos] {
            ParseItem::SyntaxNode(..) => Ok(self),
            ParseItem::Token(Token::Quan(..)) => self.parse_quan_at(pos),
            ParseItem::Token(Token::Paren(p)) => match p.as_str() {
                "(" => self.parse_conn_at(pos),
                "{" => self.parse_comp_at(pos),
                x => bail!("Unexpected token '{}'", x),
            },
            ParseItem::Token(Token::Conn(c)) => match c.as_str() {
                "¬" | "!" | "\\lnot" => self.parse_neg_at(pos),
                x => bail!("Unexpected token '{}'", x),
            },
            ParseItem::Token(x) => bail!("Unexpected token {:?}", x),
        }
    }

    fn parse_rel_at(mut self, pos: usize) -> Result<Self> {
        assert!(
            matches!(self[pos], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Comprehension | NodeType::EmptySet))
        );
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self[pos + 1], ParseItem::Token(Token::Rel(..))),
            "Unexpected token after set"
        );
        self = self.parse_at(pos + 2)?;
        ensure!(
            matches!(self[pos + 2], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Comprehension | NodeType::EmptySet)),
            "Unexpected second relatum, expected variable, comprehension or empty set"
        );
        if let (
            ParseItem::SyntaxNode(left),
            ParseItem::Token(Token::Rel(rel)),
            ParseItem::SyntaxNode(right),
        ) = (self.remove(pos), self[pos], self.remove(pos + 1))
        {
            let entry = match rel.as_str() {
                "=" => NodeType::Relation(Relation::Equality),
                "∈" | "\\epsilon" => NodeType::Relation(Relation::Element),
                x => unimplemented!("Parser for relation '{}' not implemented", x),
            };
            let children = vec![left, right];
            self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
            Ok(self)
        } else {
            unreachable!()
        }
    }

    fn parse_quan_at(mut self, pos: usize) -> Result<Self> {
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        self = self.parse_at(pos + 2)?;
        let formula = self.remove(pos + 2);
        let var = self.remove(pos + 1);
        if let (ParseItem::SyntaxNode(formula), ParseItem::SyntaxNode(var)) = (formula, var) {
            ensure!(matches!(var.entry, NodeType::Variable(..)));
            if let ParseItem::Token(Token::Quan(q)) = &self[pos] {
                let entry = match q.as_str() {
                    "∀" | "\\forall" => NodeType::Quantifier(Quantifier::Universal),
                    "∃" | "\\exists" => NodeType::Quantifier(Quantifier::Existential),
                    x => unimplemented!("Quantifier token '{}' not implemented in parser", x),
                };
                let children = vec![var, formula];
                self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
                Ok(self)
            } else {
                unreachable!()
            }
        } else {
            unreachable!("Found Token after calling parse_at")
        }
    }

    fn parse_conn_at(mut self, pos: usize) -> Result<Self> {
        assert!(matches!(self.remove(pos), ParseItem::Token(Token::Paren(p)) if p == "("));
        ensure!(pos < self.len(), "Unexpected end of input");
        self = self.parse_at(pos)?;
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        self = self.parse_at(pos + 2)?;
        ensure!(pos + 3 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self.remove(pos + 3), ParseItem::Token(Token::Paren(p)) if p == ")"),
            "Missing token ')'"
        );
        if let (ParseItem::SyntaxNode(left), ParseItem::SyntaxNode(right)) =
            (self.remove(pos), self.remove(pos + 1))
        {
            let entry = NodeType::Connective(match &self[pos] {
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
            let children = vec![left, right];
            self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
            Ok(self)
        } else {
            unreachable!("Found Token after calling parse_at")
        }
    }

    fn parse_neg_at(mut self, pos: usize) -> Result<Self> {
        assert!(
            matches!(&self[pos], ParseItem::Token(Token::Conn(c)) if matches!(c.as_str(), "¬" | "!" | "\\lnot"))
        );
        ensure!(pos + 1 < self.len(), "Unexpected end of input");
        self = self.parse_at(pos + 1)?;
        if let ParseItem::SyntaxNode(child) = self.remove(pos + 1) {
            let entry = NodeType::Connective(Connective::Negation);
            let children = vec![child];
            self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
            Ok(self)
        } else {
            unreachable!("Found Token after calling parse_at")
        }
    }

    fn parse_comp_at(mut self, pos: usize) -> Result<Self> {
        assert!(matches!(self.remove(pos), ParseItem::Token(Token::Conn(c)) if c == "{"));
        ensure!(pos < self.len(), "Unexpected end of input");
        self = self.parse_at(pos)?;
        ensure!(
            matches!(&self[pos], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Relation(Relation::Element))),
            "First part of set comprehension must be an element relation"
        );
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(&self[pos + 1],ParseItem::Token(Token::Paren(p)) if p == "|"),
            "Missing token '|' in set comprehension"
        );
        self = self.parse_at(pos + 2)?;
        ensure!(pos + 3 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self.remove(pos + 3), ParseItem::Token(Token::Paren(p)) if p == "}"),
            "Mising token '}}'"
        );
        if let (ParseItem::SyntaxNode(left), ParseItem::SyntaxNode(right)) =
            (self.remove(pos), self.remove(pos + 1))
        {
            let entry = NodeType::Comprehension;
            let children = vec![left, right];
            self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
            Ok(self)
        } else {
            unreachable!("Found token after calling parse_at");
        }
    }
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
