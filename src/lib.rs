mod display;
mod lexer;
mod parser;
mod transformer;

pub fn run(input: &str) -> String {
    match lexer::tokanize(input.into()) {
        Err(e) => e.to_string(),
        Ok(tokens) => match parser::parse(tokens) {
            Err(e) => e.to_string(),
            Ok(syntax_tree) => syntax_tree.transform().to_string(),
        },
    }
}
