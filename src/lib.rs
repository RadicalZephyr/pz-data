use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace1, space1},
    error::ParseError,
    multi::separated_list1,
    sequence::{delimited, pair},
    IResult, Parser,
};

pub struct Module<Definitions> {
    pub blocks: Vec<ModuleBlock<Definitions>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleBlock<Definitions> {
    pub name: String,
    pub definitions: Vec<Definitions>,
}

impl<Definitions> ModuleBlock<Definitions> {
    pub fn new(name: String, definitions: Vec<Definitions>) -> Self {
        Self { name, definitions }
    }
}

impl<'a, T> From<(&'a str, Vec<T>)> for ModuleBlock<T> {
    fn from((name, items): (&'a str, Vec<T>)) -> Self {
        ModuleBlock::new(String::from(name), items)
    }
}

pub fn block<'a, 'b, F, O, OI, E>(
    block_tag: &'b str,
    mut item: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    'b: 'a,
    O: From<(&'a str, OI)>,
    F: Parser<&'a str, OI, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        let (input, _) = tag(block_tag)(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = alphanumeric1(input)?;
        let (input, _) = multispace1(input)?;
        let (input, parsed_item) = delimited(
            pair(tag("{"), multispace1),
            |input| item.parse(input),
            pair(multispace1, tag("}")),
        )(input)?;

        Ok((input, O::from((name, parsed_item))))
    }
}

pub fn block_repeated<'a, 'b, F, O, OI, E>(
    block_tag: &'b str,
    mut item: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    'b: 'a,
    O: From<(&'a str, Vec<OI>)>,
    F: Parser<&'a str, OI, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        block(
            block_tag,
            separated_list1(multispace1, |input| item.parse(input)),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use nom::{
        character::complete::{digit1, space0},
        combinator::map_res,
        sequence::{pair, preceded},
    };

    use super::*;

    type Result<T> = IResult<&'static str, T, nom::error::Error<&'static str>>;

    #[test]
    fn parse_container_block() {
        let module_text = "container Foo { foo }";
        let expected = ("Foo", "foo");

        let module_res: Result<(&str, &str)> = block("container", tag("foo"))(module_text);
        let (_, actual) = module_res.expect("failed to parse module");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_repeated_block() {
        let module_text = "module Base { foo foo foo }";
        let expected = ModuleBlock {
            name: String::from("Base"),
            definitions: vec!["foo", "foo", "foo"],
        };

        let module_res: Result<ModuleBlock<&'static str>> =
            block_repeated("module", tag("foo"))(module_text);
        let (_, actual) = module_res.expect("failed to parse module");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_complex_item_repeated_block() {
        let module_text = "items Bar {
 item 1
 item 2
 item 3
}";
        let expected = ("Bar", vec![1, 2, 3]);

        let module_res: Result<(&str, Vec<u8>)> = block_repeated(
            "items",
            preceded(
                pair(tag("item"), space1),
                map_res(digit1, |s: &str| s.parse::<u8>()),
            ),
        )(module_text);
        let (_, actual) = module_res.expect("failed to parse module");

        assert_eq!(expected, actual);
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ItemBody {
        display_category: String,
        r#type: String,
        display_name: String,
        icon: String,
    }

    fn field_value<'a, 'b, F, O, E>(
        field_name: &'b str,
        mut value: F,
    ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
        'b: 'a,
        F: Parser<&'a str, O, E>,
        E: ParseError<&'a str>,
    {
        move |input: &'a str| {
            let (input, _) = preceded(space0, tag(field_name))(input)?;
            let (input, _) = delimited(space0, tag("="), space0)(input)?;
            let (input, parsed_value) = value.parse(input)?;
            let (input, _) = tag(",")(input)?;
            Ok((input, parsed_value))
        }
    }

    fn item_body(input: &'static str) -> Result<ItemBody> {
        let (input, display_category) =
            field_value("DisplayCategory", Parser::into(alphanumeric1))(input)?;
        let (input, _) = multispace1(input)?;
        let (input, r#type) = field_value("Type", Parser::into(alphanumeric1))(input)?;
        let (input, _) = multispace1(input)?;
        let (input, display_name) = field_value("DisplayName", Parser::into(alphanumeric1))(input)?;
        let (input, _) = multispace1(input)?;
        let (input, icon) = field_value("Icon", Parser::into(alphanumeric1))(input)?;
        Ok((
            input,
            ItemBody {
                display_category,
                r#type,
                display_name,
                icon,
            },
        ))
    }

    #[test]
    fn parse_complex_item_heterogenous_block() {
        let module_text = "item RedRadish {
  DisplayCategory = Food,
  Type            = Food,
  DisplayName     = Radish,
  Icon            = Radish,
}";
        let expected = (
            "RedRadish",
            ItemBody {
                display_category: String::from("Food"),
                r#type: String::from("Food"),
                display_name: String::from("Radish"),
                icon: String::from("Radish"),
            },
        );

        let module_res: Result<(&str, ItemBody)> = block("item", item_body)(module_text);
        let (_, actual) = module_res.expect("failed to parse module");

        assert_eq!(expected, actual);
    }
}
