use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, space1},
    error::ParseError,
    multi::many0,
    sequence::delimited,
    IResult, Parser,
};

enum Error {
    Dummy,
}

pub struct Module<Definitions> {
    pub blocks: Vec<ModuleBlock<Definitions>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleBlock<Definitions> {
    pub name: String,
    pub definitions: Vec<Definitions>,
}

pub fn module<'a, F, O, E>(
    mut definition: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, ModuleBlock<O>, E>
where
    F: Parser<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        let (input, _) = tag("module")(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = alphanumeric1(input)?;
        let (input, _) = space1(input)?;
        let (input, definitions) = delimited(
            tag("{"),
            delimited(space1, many0(|input| definition.parse(input)), space1),
            tag("}"),
        )(input)?;

        Ok((
            input,
            ModuleBlock {
                name: name.to_string(),
                definitions,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result<T> = IResult<&'static str, T, nom::error::Error<&'static str>>;

    #[test]
    fn parse_module() {
        let module_text = "module Base { foo }";
        let expected = ModuleBlock {
            name: String::from("Base"),
            definitions: vec!["foo"],
        };

        let module_res: Result<ModuleBlock<&'static str>> = module(tag("foo"))(module_text);
        let (_, actual) = module_res.expect("failed to parse module");

        assert_eq!(expected, actual);
    }
}
