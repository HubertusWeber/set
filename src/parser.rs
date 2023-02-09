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
    pub entry: NodeType,
    pub children: Vec<SyntaxNode>,
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
    Subset,
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
    Union,
    Intersection,
    Difference,
}

#[derive(Debug, Clone, Copy)]
struct Depth {
    val: usize,
    pos: usize,
}

pub fn parse(tokens: Vec<Token>) -> Result<SyntaxNode> {
    tokens
        .into_iter()
        .map(|t| ParseItem::Token(t))
        .collect::<Vec<ParseItem>>()
        .parse()
}

trait Parsable
where
    Self: Sized,
{
    fn parse(self) -> Result<SyntaxNode>;
    fn parse_consts(self) -> Self;
    fn parse_vars(self) -> Self;
    fn parse_at(self, pos: usize) -> Result<Self>;
    fn parse_rel_at(self, pos: usize) -> Result<Self>;
    fn parse_quan_at(self, pos: usize) -> Result<Self>;
    fn parse_conn_at(self, pos: usize) -> Result<Self>;
    fn parse_neg_at(self, pos: usize) -> Result<Self>;
    fn parse_comp_at(self, pos: usize) -> Result<Self>;
    fn parse_unop_at(self, pos: usize) -> Result<Self>;
    fn parse_binop_at(self, pos: usize) -> Result<Self>;
}

impl Parsable for Vec<ParseItem> {
    fn parse(mut self) -> Result<SyntaxNode> {
        self = self.parse_consts().parse_vars().parse_at(0)?;
        ensure!(self.len() == 1, "Unexpected token, expected end of input");
        let ParseItem::SyntaxNode(result) = self.remove(0) else  {unreachable!()};
        Ok(result)
    }

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
            ParseItem::SyntaxNode(n)
                if matches!(
                    n.entry,
                    NodeType::Variable(..)
                        | NodeType::Operator(..)
                        | NodeType::Comprehension
                        | NodeType::EmptySet
                ) && pos + 1 < self.len() =>
            {
                match self[pos + 1] {
                    ParseItem::Token(Token::Rel(..)) => self.parse_rel_at(pos),
                    ParseItem::Token(Token::BinOp(..)) => self.parse_binop_at(pos),
                    _ => Ok(self),
                }
            }
            ParseItem::SyntaxNode(..) => Ok(self),
            ParseItem::Token(Token::Quan(..)) => self.parse_quan_at(pos),
            ParseItem::Token(Token::Brack(p)) => match p.as_str() {
                "(" => self.parse_conn_at(pos),
                "{" => self.parse_comp_at(pos),
                x => unimplemented!("Parser for bracket '{}' not implemented", x),
            },
            ParseItem::Token(Token::Conn(c)) => match c.as_str() {
                "¬" | "!" | "\\lnot" => self.parse_neg_at(pos),
                x => unimplemented!("Parser for connective '{}' not implemented", x),
            },
            ParseItem::Token(Token::UnOp(..)) => self.parse_unop_at(pos),
            ParseItem::Token(x) => bail!("Unexpected token {:?}", x),
        }
    }

    fn parse_rel_at(mut self, pos: usize) -> Result<Self> {
        assert!(
            matches!(&self[pos], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Operator(..) | NodeType::Comprehension | NodeType::EmptySet))
        );
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        assert!(matches!(self[pos + 1], ParseItem::Token(Token::Rel(..))),);
        self = self.parse_at(pos + 2)?;
        ensure!(
            matches!(&self[pos + 2], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Operator(..) | NodeType::Comprehension | NodeType::EmptySet)),
            "Unexpected second relatum, expected variable, operation, comprehension or empty set"
        );
        let ParseItem::SyntaxNode(left) = self.remove(pos) else {unreachable!()};
        let ParseItem::SyntaxNode(right) = self.remove(pos + 1) else {unreachable!()};
        let ParseItem::Token(Token::Rel(rel)) = &self[pos] else {unreachable!()};
        let entry = match rel.as_str() {
            "=" => NodeType::Relation(Relation::Equality),
            "∈" | "\\epsilon" => NodeType::Relation(Relation::Element),
            "⊆" | "\\subseteq" => NodeType::Relation(Relation::Subset),
            x => unimplemented!("Parser for relation '{}' not implemented", x),
        };
        let children = vec![left, right];
        self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        Ok(self)
    }

    fn parse_quan_at(mut self, pos: usize) -> Result<Self> {
        assert!(matches!(self[pos], ParseItem::Token(Token::Quan(..))));
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        self = self.parse_at(pos + 2)?;
        let ParseItem::SyntaxNode(var) = self.remove(pos + 1) else {unreachable!()};
        let ParseItem::SyntaxNode(formula) = self.remove(pos + 1) else {unreachable!()};
        ensure!(
            matches!(var.entry, NodeType::Variable(..)),
            "Unexpected Token, expected variable after quantifier"
        );
        let ParseItem::Token(Token::Quan(q)) = &self[pos] else {unreachable!()};
        let entry = match q.as_str() {
            "∀" | "\\forall" => NodeType::Quantifier(Quantifier::Universal),
            "∃" | "\\exists" => NodeType::Quantifier(Quantifier::Existential),
            x => unimplemented!("Quantifier token '{}' not implemented in parser", x),
        };
        let children = vec![var, formula];
        self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        Ok(self)
    }

    fn parse_conn_at(mut self, pos: usize) -> Result<Self> {
        assert!(matches!(self.remove(pos), ParseItem::Token(Token::Brack(p)) if p == "("));
        ensure!(pos < self.len(), "Unexpected end of input");
        self = self.parse_at(pos)?;
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        self = self.parse_at(pos + 2)?;
        ensure!(pos + 3 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self.remove(pos + 3), ParseItem::Token(Token::Brack(p)) if p == ")"),
            "Missing token ')'"
        );
        let ParseItem::SyntaxNode(left) = self.remove(pos) else {unreachable!()};
        let ParseItem::SyntaxNode(right) = self.remove(pos + 1) else {unreachable!()};
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
    }

    fn parse_neg_at(mut self, pos: usize) -> Result<Self> {
        assert!(
            matches!(&self[pos], ParseItem::Token(Token::Conn(c)) if matches!(c.as_str(), "¬" | "!" | "\\lnot"))
        );
        ensure!(pos + 1 < self.len(), "Unexpected end of input");
        self = self.parse_at(pos + 1)?;
        let ParseItem::SyntaxNode(child) = self.remove(pos + 1) else {unreachable!()};
        let entry = NodeType::Connective(Connective::Negation);
        let children = vec![child];
        self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        Ok(self)
    }

    fn parse_comp_at(mut self, pos: usize) -> Result<Self> {
        assert!(matches!(self.remove(pos), ParseItem::Token(Token::Brack(c)) if c == "{"));
        ensure!(pos < self.len(), "Unexpected end of input");
        self = self.parse_at(pos)?;
        ensure!(
            matches!(&self[pos], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Relation(Relation::Element))),
            "First part of set comprehension must be an element relation"
        );
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(&self[pos + 1],ParseItem::Token(Token::Brack(p)) if p == "|"),
            "Missing token '|' in set comprehension"
        );
        self = self.parse_at(pos + 2)?;
        ensure!(pos + 3 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self.remove(pos + 3), ParseItem::Token(Token::Brack(p)) if p == "}"),
            "Missing token '}}'"
        );
        let ParseItem::SyntaxNode(left) = self.remove(pos) else {unreachable!()};
        let ParseItem::SyntaxNode(right) = self.remove(pos + 1) else {unreachable!()};
        let entry = NodeType::Comprehension;
        let children = vec![left, right];
        self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        self.parse_at(pos)
    }

    fn parse_unop_at(mut self, pos: usize) -> Result<Self> {
        assert!(matches!(self[pos], ParseItem::Token(Token::UnOp(..))));
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self.remove(pos + 1), ParseItem::Token(Token::Brack(p)) if p == "("),
            "Unexpected token, expected '('"
        );
        self = self.parse_at(pos + 1)?;
        ensure!(
            matches!(&self[pos + 1], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Operator(..) | NodeType::Comprehension | NodeType::EmptySet)),
            "Unexpected operand, expected variable, operation, comprehension or empty set"
        );
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        ensure!(
            matches!(self.remove(pos + 2), ParseItem::Token(Token::Brack(p)) if p == ")"),
            "Unexpected token, expected ')'"
        );
        let ParseItem::SyntaxNode(operand) = self.remove(pos + 1) else {unreachable!()};
        let ParseItem::Token(Token::UnOp(op)) = &self[pos] else {unreachable!()};
        let entry = match op.as_str() {
            "Pot" => NodeType::Operator(Operator::PowerSet),
            x => unimplemented!("Operator token '{}' not implemented in parser", x),
        };
        let children = vec![operand];
        self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        self.parse_at(pos)
    }

    fn parse_binop_at(mut self, pos: usize) -> Result<Self> {
        assert!(
            matches!(&self[pos], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Operator(..) | NodeType::Comprehension | NodeType::EmptySet))
        );
        ensure!(pos + 2 < self.len(), "Unexpected end of input");
        assert!(matches!(self[pos + 1], ParseItem::Token(Token::BinOp(..))),);
        match &self[pos + 2] {
            ParseItem::Token(Token::Brack(b)) if b.as_str() == "{" => {
                self = self.parse_comp_at(pos + 2)?
            }
            ParseItem::Token(Token::UnOp(..)) => self = self.parse_unop_at(pos + 2)?,
            _ => (),
        };
        ensure!(
            matches!(&self[pos + 2], ParseItem::SyntaxNode(n) if matches!(n.entry, NodeType::Variable(..) | NodeType::Operator(..) | NodeType::Comprehension | NodeType::EmptySet)),
            "Unexpected second operand, expected variable, operation, comprehension or empty set"
        );
        let ParseItem::SyntaxNode(left) = self.remove(pos) else {unreachable!()};
        let ParseItem::SyntaxNode(right) = self.remove(pos + 1) else {unreachable!()};
        let ParseItem::Token(Token::BinOp(op)) = &self[pos] else {unreachable!()};
        let entry = match op.as_str() {
            "∪" | "\\cup" => NodeType::Operator(Operator::Union),
            "∩" | "\\cap" => NodeType::Operator(Operator::Intersection),
            "\\" => NodeType::Operator(Operator::Difference),
            x => unimplemented!("Parser for binary operator '{}' not implemented", x),
        };
        let children = vec![left, right];
        self[pos] = ParseItem::SyntaxNode(SyntaxNode { entry, children });
        self.parse_at(pos)
    }
}
