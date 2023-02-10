use std::collections::HashSet;

use crate::parser::{Connective, NodeType, Operator, Quantifier, Relation, SyntaxNode};

impl SyntaxNode {
    pub fn transform(self) -> Self {
        self.negated_relation().subset().operators()
    }

    fn negated_relation(mut self) -> Self {
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).negated_relation();
            self.children.push(child);
        }
        if let NodeType::Relation(r) = self.entry {
            let entry = match r {
                Relation::NotEqual => NodeType::Relation(Relation::Equality),
                Relation::NotElement => NodeType::Relation(Relation::Element),
                Relation::NotSubset => NodeType::Relation(Relation::Subset),
                _ => return self,
            };
            let children = self.children;
            let child = SyntaxNode { entry, children };
            self.entry = NodeType::Connective(Connective::Negation);
            self.children = vec![child];
        }
        self
    }

    fn subset(mut self) -> Self {
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).subset();
            self.children.push(child);
        }
        match self.entry {
            NodeType::Relation(r) if matches!(r, Relation::Subset) => {
                let var = self.get_free_vars(1).remove(0);
                let antecedent = SyntaxNode {
                    entry: NodeType::Relation(Relation::Element),
                    children: vec![var.clone(), self.children.remove(0)],
                };
                let consequent = SyntaxNode {
                    entry: NodeType::Relation(Relation::Element),
                    children: vec![var.clone(), self.children.remove(0)],
                };
                let implication = SyntaxNode {
                    entry: NodeType::Connective(Connective::Implication),
                    children: vec![antecedent, consequent],
                };
                self.entry = NodeType::Quantifier(Quantifier::Universal);
                self.children.push(var);
                self.children.push(implication);
            }
            _ => (),
        }
        self
    }

    fn operators(mut self) -> Self {
        match self.entry {
            NodeType::Relation(r) if matches!(r, Relation::Equality) => {
                if matches!(
                    self.children[0].entry,
                    NodeType::Operator(Operator::PowerSet)
                ) {
                    self = self.phi_powerset();
                }
                if matches!(
                    self.children[1].entry,
                    NodeType::Operator(Operator::PowerSet)
                ) {
                    self.children.swap(0, 1);
                    self = self.phi_powerset();
                }
                if matches!(self.children[0].entry, NodeType::Operator(o) if !matches!(o, Operator::PowerSet) )
                    || matches!(self.children[1].entry, NodeType::Operator(o) if !matches!(o, Operator::PowerSet) )
                {
                    self = self.ext();
                }
            }
            NodeType::Relation(r) if matches!(r, Relation::Element) => {
                match self.children[1].entry {
                    NodeType::Operator(Operator::PowerSet) => {
                        let var = self.get_free_vars(1).remove(0);
                        let right = self.children.remove(1);
                        let left = self.children.remove(0);
                        let equality = SyntaxNode {
                            entry: NodeType::Relation(Relation::Equality),
                            children: vec![right, var.clone()],
                        }
                        .phi_powerset();
                        let element = SyntaxNode {
                            entry: NodeType::Relation(Relation::Element),
                            children: vec![left, var.clone()],
                        };
                        let conjunction = SyntaxNode {
                            entry: NodeType::Connective(Connective::Conjunction),
                            children: vec![equality, element],
                        };
                        self.entry = NodeType::Quantifier(Quantifier::Existential);
                        self.children.push(var);
                        self.children.push(conjunction);
                    }
                    NodeType::Operator(Operator::BigIntersection) => {
                        self = self.phi_big_intersection();
                    }
                    NodeType::Operator(Operator::BigUnion) => {
                        self = self.phi_big_union();
                    }
                    NodeType::Operator(Operator::Intersection) => {
                        self = self.phi_intersection();
                    }
                    NodeType::Operator(Operator::Difference) => {
                        self = self.phi_difference();
                    }
                    NodeType::Operator(Operator::Union) => {
                        self = self.phi_union();
                    }
                    NodeType::Operator(Operator::PairSet) => {
                        self = self.phi_pair_set();
                    }
                    _ => (),
                }
                if matches!(self.children[0].entry, NodeType::Operator(..)) {
                    let var = self.get_free_vars(1).remove(0);
                    let right = self.children.remove(1);
                    let left = self.children.remove(0);
                    let equality = SyntaxNode {
                        entry: NodeType::Relation(Relation::Equality),
                        children: vec![left, var.clone()],
                    };
                    let element = SyntaxNode {
                        entry: NodeType::Relation(Relation::Element),
                        children: vec![var.clone(), right],
                    };
                    let conjunction = SyntaxNode {
                        entry: NodeType::Connective(Connective::Conjunction),
                        children: vec![equality, element],
                    };
                    self.entry = NodeType::Quantifier(Quantifier::Existential);
                    self.children.push(var);
                    self.children.push(conjunction);
                }
            }
            _ => (),
        }
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).operators();
            self.children.push(child);
        }
        self
    }

    fn ext(mut self) -> Self {
        let var = self.get_free_vars(1).remove(0);
        let right = self.children.remove(1);
        let left = self.children.remove(0);
        let element_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![var.clone(), right],
        };
        let element_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![var.clone(), left],
        };
        let biconditional = SyntaxNode {
            entry: NodeType::Connective(Connective::Biconditional),
            children: vec![element_left, element_right],
        };
        self.entry = NodeType::Quantifier(Quantifier::Universal);
        self.children.push(var);
        self.children.push(biconditional);
        self
    }

    fn phi_powerset(mut self) -> Self {
        let var = self.get_free_vars(1).remove(0);
        let right = self.children.remove(1);
        let mut left = self.children.remove(0);
        let element = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![var.clone(), right],
        };
        let subset = SyntaxNode {
            entry: NodeType::Relation(Relation::Subset),
            children: vec![var.clone(), left.children.remove(0)],
        }
        .subset();
        let biconditional = SyntaxNode {
            entry: NodeType::Connective(Connective::Biconditional),
            children: vec![element, subset],
        };
        self.entry = NodeType::Quantifier(Quantifier::Existential);
        self.children.push(var);
        self.children.push(biconditional);
        self
    }

    fn phi_big_intersection(mut self) -> Self {
        let var = self.get_free_vars(1).remove(0);
        let mut right = self.children.remove(1);
        let left = self.children.remove(0);
        let element_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left, var.clone()],
        };
        let element_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![var.clone(), right.children.remove(0)],
        };
        let implication = SyntaxNode {
            entry: NodeType::Connective(Connective::Implication),
            children: vec![element_left, element_right],
        };
        self.entry = NodeType::Quantifier(Quantifier::Universal);
        self.children.push(var);
        self.children.push(implication);
        self
    }

    fn phi_big_union(mut self) -> Self {
        let var = self.get_free_vars(1).remove(0);
        let mut right = self.children.remove(1);
        let left = self.children.remove(0);
        let element_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left, var.clone()],
        };
        let element_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![var.clone(), right.children.remove(0)],
        };
        let implication = SyntaxNode {
            entry: NodeType::Connective(Connective::Implication),
            children: vec![element_left, element_right],
        };
        self.entry = NodeType::Quantifier(Quantifier::Existential);
        self.children.push(var);
        self.children.push(implication);
        self
    }

    fn phi_intersection(mut self) -> Self {
        let mut right = self.children.remove(1);
        let left = self.children.remove(0);
        let element_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left.clone(), right.children.remove(1)],
        };
        let element_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left, right.children.remove(0)],
        };
        self.entry = NodeType::Connective(Connective::Conjunction);
        self.children.push(element_left);
        self.children.push(element_right);
        self
    }

    fn phi_difference(mut self) -> Self {
        let mut right = self.children.remove(1);
        let left = self.children.remove(0);
        let element_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left.clone(), right.children.remove(1)],
        };
        let element_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left, right.children.remove(0)],
        };
        let negation = SyntaxNode {
            entry: NodeType::Connective(Connective::Negation),
            children: vec![element_right],
        };
        self.entry = NodeType::Connective(Connective::Conjunction);
        self.children.push(element_left);
        self.children.push(negation);
        self
    }

    fn phi_union(mut self) -> Self {
        let mut right = self.children.remove(1);
        let left = self.children.remove(0);
        let element_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left.clone(), right.children.remove(1)],
        };
        let element_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![left, right.children.remove(0)],
        };
        self.entry = NodeType::Connective(Connective::Disjunction);
        self.children.push(element_left);
        self.children.push(element_right);
        self
    }

    fn phi_pair_set(mut self) -> Self {
        let mut right = self.children.remove(1);
        let left = self.children.remove(0);
        let equality_right = SyntaxNode {
            entry: NodeType::Relation(Relation::Equality),
            children: vec![left.clone(), right.children.remove(1)],
        };
        let equality_left = SyntaxNode {
            entry: NodeType::Relation(Relation::Equality),
            children: vec![left, right.children.remove(0)],
        };
        self.entry = NodeType::Connective(Connective::Disjunction);
        self.children.push(equality_left);
        self.children.push(equality_right);
        self
    }

    fn get_free_vars(&self, count: usize) -> Vec<SyntaxNode> {
        let mut result = Vec::<SyntaxNode>::new();
        let used_indices = self.collect_used_indices(HashSet::<u32>::new());
        let mut n = 0..;
        while result.len() < count {
            let index = n.next().unwrap();
            if !used_indices.contains(&index) {
                result.push(SyntaxNode {
                    entry: NodeType::Variable(index),
                    children: vec![],
                });
            }
        }
        result
    }

    fn collect_used_indices(&self, mut set: HashSet<u32>) -> HashSet<u32> {
        if let NodeType::Variable(v) = self.entry {
            set.insert(v);
        }
        for child in &self.children {
            set = child.collect_used_indices(set);
        }
        set
    }
}
