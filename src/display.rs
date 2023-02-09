use crate::{
    lexer::Token,
    parser::{Connective, NodeType, Operator, Quantifier, Relation, SyntaxNode},
};
use std::fmt;

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\"",
            match self {
                Token::Const(c) => c,
                Token::Var(v) => v,
                Token::Brack(b) => b,
                Token::Rel(r) => r,
                Token::UnOp(o) => o,
                Token::BinOp(o) => o,
                Token::Conn(c) => c,
                Token::Quan(q) => q,
            }
        )
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.entry {
            NodeType::EmptySet => write!(f, "∅"),
            NodeType::Variable(v) => write!(f, "v{}", v),
            NodeType::Comprehension => write!(f, "{{{} | {}}}", self.children[0], self.children[1]),
            NodeType::Relation(r) => match r {
                Relation::Equality => write!(f, "{} = {}", self.children[0], self.children[1]),
                Relation::Element => write!(f, "{} ∈ {}", self.children[0], self.children[1]),
                Relation::Subset => write!(f, "{} ⊆ {}", self.children[0], self.children[1]),
            },
            NodeType::Operator(o) => match o {
                Operator::PowerSet => write!(f, "Pot({})", self.children[0]),
            },
            NodeType::Connective(c) => match c {
                Connective::Negation => write!(f, "¬{}", self.children[0]),
                Connective::Conjunction => {
                    write!(f, "({} ∧ {})", self.children[0], self.children[1])
                }
                Connective::Disjunction => {
                    write!(f, "({} ∨ {})", self.children[0], self.children[1])
                }
                Connective::Implication => {
                    write!(f, "({} → {})", self.children[0], self.children[1])
                }
                Connective::Biconditional => {
                    write!(f, "({} ↔ {})", self.children[0], self.children[1])
                }
            },
            NodeType::Quantifier(q) => match q {
                Quantifier::Universal => write!(f, "∀{} {}", self.children[0], self.children[1]),
                Quantifier::Existential => write!(f, "∃{} {}", self.children[0], self.children[1]),
            },
        }
    }
}
