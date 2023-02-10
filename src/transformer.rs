use std::collections::HashSet;

use crate::parser::{Connective, NodeType, Operator, Quantifier, Relation, SyntaxNode};

impl SyntaxNode {
    pub fn transform(self) -> Self {
        self.negated_relation().subset().powerset()
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

    fn powerset(mut self) -> Self {
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).powerset();
            self.children.push(child);
        }
        if matches!(self.entry, NodeType::Relation(Relation::Equality))
            && matches!(
                self.children[0].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            self.phi_t().powerset()
        } else if matches!(self.entry, NodeType::Relation(Relation::Equality))
            && matches!(
                self.children[1].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            self.children.swap(0, 1);
            self.phi_t().powerset()
        } else if matches!(self.entry, NodeType::Relation(Relation::Element))
            && matches!(
                self.children[0].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            let var = self.get_free_vars(1).remove(0);
            let right = self.children.remove(1);
            let left = self.children.remove(0);
            let equality = SyntaxNode {
                entry: NodeType::Relation(Relation::Equality),
                children: vec![left, var.clone()],
            }
            .phi_t();
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
            self.powerset()
        } else if matches!(self.entry, NodeType::Relation(Relation::Element))
            && matches!(
                self.children[1].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            let var = self.get_free_vars(1).remove(0);
            let right = self.children.remove(1);
            let left = self.children.remove(0);
            let equality = SyntaxNode {
                entry: NodeType::Relation(Relation::Equality),
                children: vec![right, var.clone()],
            }
            .phi_t();
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
            self.powerset()
        } else if matches!(self.entry, NodeType::Operator(o) if o.is_unary())
            && matches!(
                self.children[0].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            let var = self.get_free_vars(1).remove(0);
            let child = self.children.remove(0);
            let equality = SyntaxNode {
                entry: NodeType::Relation(Relation::Equality),
                children: vec![child, var.clone()],
            }
            .phi_t();
            let operator = SyntaxNode {
                entry: self.entry,
                children: vec![var.clone()],
            };
            let conjunction = SyntaxNode {
                entry: NodeType::Connective(Connective::Conjunction),
                children: vec![equality, operator],
            };
            self.entry = NodeType::Quantifier(Quantifier::Existential);
            self.children.push(var);
            self.children.push(conjunction);
            self
        } else if matches!(self.entry, NodeType::Operator(o) if o.is_binary())
            && matches!(
                self.children[0].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            let var = self.get_free_vars(1).remove(0);
            let right = self.children.remove(1);
            let left = self.children.remove(0);
            let equality = SyntaxNode {
                entry: NodeType::Relation(Relation::Equality),
                children: vec![left, var.clone()],
            }
            .phi_t();
            let operator = SyntaxNode {
                entry: self.entry,
                children: vec![var.clone(), right],
            };
            let conjunction = SyntaxNode {
                entry: NodeType::Connective(Connective::Conjunction),
                children: vec![equality, operator],
            };
            self.entry = NodeType::Quantifier(Quantifier::Existential);
            self.children.push(var);
            self.children.push(conjunction);
            self.powerset()
        } else if matches!(self.entry, NodeType::Operator(o) if o.is_binary())
            && matches!(
                self.children[0].entry,
                NodeType::Operator(Operator::PowerSet)
            )
        {
            let var = self.get_free_vars(1).remove(0);
            let right = self.children.remove(1);
            let left = self.children.remove(0);
            let equality = SyntaxNode {
                entry: NodeType::Relation(Relation::Equality),
                children: vec![right, var.clone()],
            }
            .phi_t();
            let operator = SyntaxNode {
                entry: self.entry,
                children: vec![left, var.clone()],
            };
            let conjunction = SyntaxNode {
                entry: NodeType::Connective(Connective::Conjunction),
                children: vec![equality, operator],
            };
            self.entry = NodeType::Quantifier(Quantifier::Existential);
            self.children.push(var);
            self.children.push(conjunction);
            self.powerset()
        } else {
            self
        }
    }

    fn phi_t(mut self) -> Self {
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
