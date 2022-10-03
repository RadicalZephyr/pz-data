use nom::{error::ParseError, Parser};

use crate::named_block_repeated;

pub struct Module<Definitions> {
    pub blocks: Vec<ModuleBlock<Definitions>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleBlock<Definitions> {
    pub name: String,
    pub definitions: Vec<Definitions>,
}

impl<Definitions> ModuleBlock<Definitions> {
    pub fn new(name: impl Into<String>, definitions: Vec<Definitions>) -> Self {
        Self {
            name: name.into(),
            definitions,
        }
    }
}

impl<'a, T> From<(&'a str, Vec<T>)> for ModuleBlock<T> {
    fn from((name, items): (&'a str, Vec<T>)) -> Self {
        ModuleBlock::new(name, items)
    }
}

pub fn module<'a, F, I, E>(item: F) -> impl Parser<&'a str, ModuleBlock<I>, E>
where
    F: Parser<&'a str, I, E>,
    E: ParseError<&'a str>,
{
    Parser::into(named_block_repeated("module", item))
}

#[cfg(test)]
mod tests {
    use nom::{bytes::complete::tag, IResult, Parser};

    use crate::named_block_repeated;

    use super::*;

    type Result<T> = IResult<&'static str, T, nom::error::Error<&'static str>>;

    #[test]
    fn parse_repeated_block() {
        let test_text = "module Base { foo foo foo }";
        let expected = ModuleBlock {
            name: String::from("Base"),
            definitions: vec!["foo", "foo", "foo"],
        };

        let block_res: Result<ModuleBlock<&'static str>> =
            Parser::into(named_block_repeated("module", tag("foo"))).parse(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }
}
