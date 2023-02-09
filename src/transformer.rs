use std::collections::HashSet;

use crate::parser::{Connective, NodeType, Quantifier, Relation, SyntaxNode};

impl SyntaxNode {
    pub fn transform(self) -> Self {
        self.negated_relation().subset()
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
                let index = self.get_free_vars(1)[0];
                let var = SyntaxNode {
                    entry: NodeType::Variable(index),
                    children: vec![],
                };
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

    fn get_free_vars(&self, count: usize) -> Vec<u32> {
        let mut result = Vec::<u32>::new();
        let used_indices = self.collect_used_indices(HashSet::<u32>::new());
        let mut n = 0..;
        while result.len() < count {
            let index = n.next().unwrap();
            if !used_indices.contains(&index) {
                result.push(index);
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
