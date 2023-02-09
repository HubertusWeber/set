use std::collections::HashSet;

use crate::parser::{Connective, NodeType, Relation, SyntaxNode};

impl SyntaxNode {
    pub fn transform(self) -> Self {
        self.negated_relations()
    }

    fn negated_relations(mut self) -> Self {
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).negated_relations();
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
