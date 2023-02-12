use std::collections::{BTreeSet, HashMap};

use crate::{
    parser::{Connective, Constant, NodeType, Operator, Quantifier, Relation, SyntaxNode},
    SetConfig,
};

impl SyntaxNode {
    pub fn transform(self, config: SetConfig) -> Self {
        self.variables(config)
            .negated_relations(config)
            .subset(config)
            .constants(config)
            .operators(config)
            .comprehension(config)
    }

    fn variables(self, config: SetConfig) -> Self {
        if !config.variables {
            return self;
        }
        let used_vars = self.collect_used_indices(BTreeSet::<u32>::new());
        let relevant_vars = used_vars
            .into_iter()
            .filter(|v| {
                (*v < u32::MAX - 55 && *v > u32::MAX - 91)
                    || (*v < u32::MAX - 96 && *v > u32::MAX - 123)
            })
            .collect::<Vec<u32>>();
        let new_vars = self.get_free_indices(relevant_vars.len());
        let mut var_map = HashMap::<u32, u32>::new();
        for (k, v) in relevant_vars.into_iter().zip(new_vars.into_iter()) {
            var_map.insert(k, v);
        }
        self.replace_vars(&var_map)
    }

    fn negated_relations(mut self, config: SetConfig) -> Self {
        if !config.negated_relations {
            return self;
        }
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).negated_relations(config);
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

    fn subset(mut self, config: SetConfig) -> Self {
        if !config.subset {
            return self;
        }
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).subset(config);
            self.children.push(child);
        }
        match self.entry {
            NodeType::Relation(Relation::Subset) => {
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

    fn constants(mut self, config: SetConfig) -> Self {
        if !config.constants {
            return self;
        }
        println!("{:?}", self);
        match self.entry {
            NodeType::Relation(Relation::Equality) => {
                if matches!(
                    self.children[0].entry,
                    NodeType::Constant(Constant::EmptySet)
                ) {
                    self = self.phi_empty_set();
                } else if matches!(
                    self.children[1].entry,
                    NodeType::Constant(Constant::EmptySet)
                ) {
                    self.children.swap(0, 1);
                    self = self.phi_empty_set();
                }
            }
            NodeType::Relation(Relation::Element) => {
                if matches!(
                    self.children[1].entry,
                    NodeType::Constant(Constant::EmptySet)
                ) {
                    self = self.element_to_equality_right();
                } else if matches!(
                    self.children[0].entry,
                    NodeType::Constant(Constant::EmptySet)
                ) {
                    self = self.element_to_equality_left();
                }
            }
            _ => (),
        }
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).constants(config);
            self.children.push(child);
        }
        self
    }

    fn comprehension(mut self, config: SetConfig) -> Self {
        if !config.comprehension {
            return self;
        }
        match self.entry {
            NodeType::Relation(Relation::Equality) => {
                if matches!(self.children[0].entry, NodeType::Comprehension) {
                    self = self.phi_comprehension().subset(config);
                }
                if matches!(self.children[1].entry, NodeType::Comprehension) {
                    self.children.swap(0, 1);
                    self = self.phi_comprehension().subset(config);
                }
            }
            NodeType::Relation(Relation::Element) => {
                if matches!(self.children[1].entry, NodeType::Comprehension) {
                    self = self.element_to_equality_right();
                } else if matches!(self.children[0].entry, NodeType::Comprehension) {
                    self = self.element_to_equality_left();
                }
            }
            _ => (),
        }
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).comprehension(config);
            self.children.push(child);
        }
        self
    }

    fn operators(mut self, config: SetConfig) -> Self {
        match self.entry {
            NodeType::Relation(Relation::Equality) => {
                match self.children[0].entry {
                    NodeType::Operator(o) => match o {
                        Operator::PowerSet if config.power_set => {
                            self = self.phi_power_set().subset(config);
                        }
                        Operator::BigIntersection if config.big_intersection => {
                            self = self.ext();
                        }
                        Operator::BigUnion if config.big_union => {
                            self = self.ext();
                        }
                        Operator::Intersection if config.intersection => {
                            self = self.ext();
                        }
                        Operator::Difference if config.difference => {
                            self = self.ext();
                        }
                        Operator::Union if config.union => {
                            self = self.ext();
                        }
                        Operator::PairSet if config.pair_set => {
                            self = self.ext();
                        }
                        _ => (),
                    },
                    _ => (),
                }
                match self.children[1].entry {
                    NodeType::Operator(o) => match o {
                        Operator::PowerSet if config.power_set => {
                            self.children.swap(0, 1);
                            self = self.phi_power_set().subset(config);
                        }
                        Operator::BigIntersection if config.big_intersection => {
                            self = self.ext();
                        }
                        Operator::BigUnion if config.big_union => {
                            self = self.ext();
                        }
                        Operator::Intersection if config.intersection => {
                            self = self.ext();
                        }
                        Operator::Difference if config.difference => {
                            self = self.ext();
                        }
                        Operator::Union if config.union => {
                            self = self.ext();
                        }
                        Operator::PairSet if config.pair_set => {
                            self = self.ext();
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
            NodeType::Relation(Relation::Element) => {
                match self.children[1].entry {
                    NodeType::Operator(o) => match o {
                        Operator::PowerSet if config.power_set => {
                            self = self.element_to_equality_right()
                        }
                        Operator::BigIntersection if config.big_intersection => {
                            self = self.phi_big_intersection();
                        }
                        Operator::BigUnion if config.big_union => {
                            self = self.phi_big_union();
                        }
                        Operator::Intersection if config.intersection => {
                            self = self.phi_intersection();
                        }
                        Operator::Difference if config.difference => {
                            self = self.phi_difference();
                        }
                        Operator::Union if config.union => {
                            self = self.phi_union();
                        }
                        Operator::PairSet if config.pair_set => {
                            self = self.phi_pair_set();
                        }
                        _ => (),
                    },
                    _ => (),
                }
                match self.children[0].entry {
                    NodeType::Operator(o) => match o {
                        Operator::PowerSet if config.power_set => {
                            self = self.element_to_equality_left()
                        }
                        Operator::BigIntersection if config.big_intersection => {
                            self = self.element_to_equality_left()
                        }
                        Operator::BigUnion if config.big_union => {
                            self = self.element_to_equality_left()
                        }
                        Operator::Intersection if config.intersection => {
                            self = self.element_to_equality_left()
                        }
                        Operator::Difference if config.difference => {
                            self = self.element_to_equality_left()
                        }
                        Operator::Union if config.union => self = self.element_to_equality_left(),
                        Operator::PairSet if config.pair_set => {
                            self = self.element_to_equality_left()
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
            _ => (),
        }
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).operators(config);
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

    fn element_to_equality_left(mut self) -> Self {
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
        self
    }

    fn element_to_equality_right(mut self) -> Self {
        let var = self.get_free_vars(1).remove(0);
        let right = self.children.remove(1);
        let left = self.children.remove(0);
        let equality = SyntaxNode {
            entry: NodeType::Relation(Relation::Equality),
            children: vec![right, var.clone()],
        };
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
        self
    }

    fn phi_empty_set(mut self) -> Self {
        let var = self.get_free_vars(1).remove(0);
        let right = self.children.remove(1);
        let element = SyntaxNode {
            entry: NodeType::Relation(Relation::Element),
            children: vec![var.clone(), right],
        };
        let quantifier = SyntaxNode {
            entry: NodeType::Quantifier(Quantifier::Existential),
            children: vec![var, element],
        };
        self.entry = NodeType::Connective(Connective::Negation);
        self.children = vec![quantifier];
        self
    }

    fn phi_comprehension(mut self) -> Self {
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
        };
        let biconditional = SyntaxNode {
            entry: NodeType::Connective(Connective::Biconditional),
            children: vec![element, subset],
        };
        self.entry = NodeType::Quantifier(Quantifier::Existential);
        self.children.push(var);
        self.children.push(biconditional);
        self
    }

    fn phi_power_set(mut self) -> Self {
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
        };
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

    fn replace_vars(mut self, map: &HashMap<u32, u32>) -> Self {
        for _ in 0..self.children.len() {
            let child = self.children.remove(0).replace_vars(map);
            self.children.push(child);
        }
        if let NodeType::Variable(k) = self.entry {
            if let Some(v) = map.get(&k) {
                self.entry = NodeType::Variable(*v);
            }
        }
        self
    }

    fn get_free_vars(&self, count: usize) -> Vec<SyntaxNode> {
        self.get_free_indices(count)
            .into_iter()
            .map(|v| SyntaxNode {
                entry: NodeType::Variable(v),
                children: vec![],
            })
            .collect()
    }

    fn get_free_indices(&self, count: usize) -> Vec<u32> {
        let mut result = Vec::<u32>::new();
        let used_indices = self.collect_used_indices(BTreeSet::<u32>::new());
        let mut n = 0..;
        while result.len() < count {
            let index = n.next().unwrap();
            if !used_indices.contains(&index) {
                result.push(index);
            }
        }
        result.reverse();
        result
    }

    fn collect_used_indices(&self, mut set: BTreeSet<u32>) -> BTreeSet<u32> {
        if let NodeType::Variable(v) = self.entry {
            set.insert(v);
        }
        for child in &self.children {
            set = child.collect_used_indices(set);
        }
        set
    }
}
