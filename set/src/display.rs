use crate::parser::{Connective, Constant, NodeType, Operator, Quantifier, Relation, SyntaxNode};
use std::fmt;

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.entry {
            NodeType::Constant(c) => match c {
                Constant::EmptySet => write!(f, "∅"),
                Constant::Omega => write!(f, "ω"),
            },
            NodeType::Variable(v) => {
                if (v < u32::MAX - 55 && v > u32::MAX - 91)
                    || (v < u32::MAX - 96 && v > u32::MAX - 123)
                {
                    let ascii = char::from_u32(u32::MAX - v).unwrap();
                    write!(f, "{ascii}")
                } else {
                    let mut var = String::from("v");
                    for c in v.to_string().chars() {
                        match c {
                            '0' => var.push('₀'),
                            '1' => var.push('₁'),
                            '2' => var.push('₂'),
                            '3' => var.push('₃'),
                            '4' => var.push('₄'),
                            '5' => var.push('₅'),
                            '6' => var.push('₆'),
                            '7' => var.push('₇'),
                            '8' => var.push('₈'),
                            '9' => var.push('₉'),
                            _ => (),
                        }
                    }
                    write!(f, "{var}")
                }
            }
            NodeType::Comprehension => write!(
                f,
                "{{{} ∈ {} | {}}}",
                self.children[0], self.children[1], self.children[2]
            ),
            NodeType::Relation(r) => match r {
                Relation::Equality => write!(f, "{} = {}", self.children[0], self.children[1]),
                Relation::Element => write!(f, "{} ∈ {}", self.children[0], self.children[1]),
                Relation::Subset => write!(f, "{} ⊆ {}", self.children[0], self.children[1]),
                Relation::NotEqual => write!(f, "{} ≠ {}", self.children[0], self.children[1]),
                Relation::NotElement => write!(f, "{} ∉ {}", self.children[0], self.children[1]),
                Relation::NotSubset => write!(f, "{} ⊈ {}", self.children[0], self.children[1]),
            },
            NodeType::Operator(o) => match o {
                Operator::Singleton => write!(f, "{{{}}}", self.children[0]),
                Operator::PowerSet => write!(f, "Pot({})", self.children[0]),
                Operator::BigUnion => write!(f, "Vereinigung({})", self.children[0]),
                Operator::BigIntersection => write!(f, "Durchschnitt({})", self.children[0]),
                Operator::Union => write!(f, "{} ∪ {}", self.children[0], self.children[1]),
                Operator::Intersection => write!(f, "{} ∩ {}", self.children[0], self.children[1]),
                Operator::Difference => write!(f, "{} \\ {}", self.children[0], self.children[1]),
                Operator::PairSet => write!(f, "{{{} , {}}}", self.children[0], self.children[1]),
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
