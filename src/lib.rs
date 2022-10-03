use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    error::ParseError,
    sequence::{delimited, preceded},
    AsChar, IResult, InputTakeAtPosition, Parser,
};

mod block;
pub use block::{named_block, named_block_repeated, unnamed_block};

mod module;
pub use module::{Module, ModuleBlock};

mod recipe;
pub use recipe::{recipe, Recipe};

fn field_value<'a, 'b, 'c, F, O, E>(
    field_name: &'b str,
    separator: &'c str,
    mut value: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    'b: 'a,
    'c: 'a,
    F: Parser<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        let (input, _) = preceded(space0, tag(field_name))(input)?;
        let (input, _) = delimited(space0, tag(separator), space0)(input)?;
        let (input, parsed_value) = value.parse(input)?;
        let (input, _) = tag(",")(input)?;
        Ok((input, parsed_value))
    }
}

pub fn module<'a, F, I, E>(item: F) -> impl Parser<&'a str, ModuleBlock<I>, E>
where
    F: Parser<&'a str, I, E>,
    E: ParseError<&'a str>,
{
    Parser::into(named_block_repeated("module", item))
}

fn bool_value<'a, E>(input: &'a str) -> IResult<&'a str, bool, E>
where
    E: ParseError<&'a str>,
{
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

fn identifier1<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    input.split_at_position1_complete(
        |item| {
            if item.is_alphanum() {
                return false;
            }
            item.as_char() != '.'
        },
        nom::error::ErrorKind::RegexpFind,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use nom::character::complete::{alphanumeric1, multispace0, multispace1};

    type Result<T> = IResult<&'static str, T, nom::error::Error<&'static str>>;

    #[derive(Debug, PartialEq, Eq)]
    struct ItemBody {
        display_category: String,
        r#type: String,
        display_name: String,
        icon: String,
    }

    fn item_body(input: &'static str) -> Result<ItemBody> {
        let (input, display_category) =
            field_value("DisplayCategory", "=", Parser::into(alphanumeric1))(input)?;
        let (input, _) = multispace1(input)?;
        let (input, r#type) = field_value("Type", "=", Parser::into(alphanumeric1))(input)?;
        let (input, _) = multispace1(input)?;
        let (input, display_name) =
            field_value("DisplayName", "=", Parser::into(alphanumeric1))(input)?;
        let (input, _) = multispace1(input)?;
        let (input, icon) = field_value("Icon", "=", Parser::into(alphanumeric1))(input)?;
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
        let test_text = "item RedRadish {
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

        let block_res: Result<(&str, ItemBody)> = named_block("item", item_body)(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_complex_item_nested_blocks() {
        let test_text = "
module Base {
  item RedRadish {
    DisplayCategory = Food,
    Type            = Food,
    DisplayName     = Radish,
    Icon            = Radish,
  }
}
";
        let expected = (
            "Base",
            vec![(
                "RedRadish",
                ItemBody {
                    display_category: String::from("Food"),
                    r#type: String::from("Food"),
                    display_name: String::from("Radish"),
                    icon: String::from("Radish"),
                },
            )],
        );

        let block_res: Result<(&str, Vec<(&str, ItemBody)>)> = preceded(
            multispace0,
            named_block_repeated("module", named_block("item", item_body)),
        )(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }
}
