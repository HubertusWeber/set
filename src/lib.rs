mod display;
mod lexer;
mod parser;
mod transformer;

#[derive(Clone, Copy)]
pub struct SetConfig {
    pub variables: bool,
    pub constants: bool,
    pub negated_relations: bool,
    pub subset: bool,
    pub comprehension: bool,
    pub power_set: bool,
    pub big_intersection: bool,
    pub big_union: bool,
    pub intersection: bool,
    pub difference: bool,
    pub union: bool,
    pub pair_set: bool,
}

pub fn run(input: &str, config: SetConfig) -> String {
    match lexer::tokanize(input.into()) {
        Err(e) => e.to_string(),
        Ok(tokens) => match parser::parse(tokens) {
            Err(e) => e.to_string(),
            Ok(syntax_tree) => syntax_tree.transform(config).to_string(),
        },
    }
}
