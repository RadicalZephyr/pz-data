use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace1, space0, space1},
    combinator::map,
    error::ParseError,
    multi::separated_list1,
    number::complete::float,
    sequence::{delimited, pair, preceded, terminated},
    AsChar, IResult, InputLength, InputTake, InputTakeAtPosition, Parser, Slice,
};

pub struct Module<Definitions> {
    pub blocks: Vec<ModuleBlock<Definitions>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleBlock<Definitions> {
    pub name: String,
    pub definitions: Vec<Definitions>,
}

#[derive(Debug, PartialEq)]
pub struct Recipe {
    name: String,
    ingredients: Vec<String>,
    result: String,
    time: f32,
    category: String,
    need_to_be_learned: bool,
}

struct RecipeBody<'a> {
    ingredients: Vec<&'a str>,
    result: &'a str,
    time: f32,
    category: &'a str,
    need_to_be_learned: bool,
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

impl Recipe {
    pub fn new(
        name: impl Into<String>,
        ingredients: Vec<String>,
        result: impl Into<String>,
        time: f32,
        category: impl Into<String>,
        need_to_be_learned: bool,
    ) -> Self {
        Self {
            name: name.into(),
            ingredients,
            result: result.into(),
            time,
            category: category.into(),
            need_to_be_learned,
        }
    }
}

impl<'a> From<(&'a str, RecipeBody<'a>)> for Recipe {
    fn from((name, body): (&'a str, RecipeBody)) -> Self {
        let RecipeBody {
            ingredients,
            result,
            time,
            category,
            need_to_be_learned,
        } = body;
        Recipe {
            name: name.to_string(),
            ingredients: ingredients.into_iter().map(|s| s.to_string()).collect(),
            result: result.to_string(),
            time,
            category: category.to_string(),
            need_to_be_learned,
        }
    }
}

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

fn non_curly_brace<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position1_complete(|item| item.as_char() == '{', nom::error::ErrorKind::Char)
}

fn string_with_spaces_delimited_by_open_brace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (_tail, name_trailing_space_and_brace) = non_curly_brace(<&str>::clone(&input))?;
    let len = name_trailing_space_and_brace.input_len();
    let name_and_trailing_space = name_trailing_space_and_brace.slice(..len - 1);
    let trimmed_name = name_and_trailing_space.trim_end();
    let name_len = trimmed_name.input_len();

    Ok(input.take_split(name_len))
}

pub fn block<'a, 'b, F, O, E>(mut item: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    'b: 'a,
    F: Parser<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        delimited(
            pair(tag("{"), multispace1),
            |input| item.parse(input),
            pair(multispace1, tag("}")),
        )(input)
    }
}

pub fn unnamed_block<'a, 'b, F, O, E>(
    block_tag: &'b str,
    mut item: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    'b: 'a,
    F: Parser<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        let (input, _) = tag(block_tag)(input)?;
        let (input, _) = multispace1(input)?;

        block(|input| item.parse(input))(input)
    }
}

pub fn named_block<'a, 'b, F, O, E>(
    block_tag: &'b str,
    mut item: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, (&'a str, O), E>
where
    'b: 'a,
    F: Parser<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        let (input, _) = tag(block_tag)(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = string_with_spaces_delimited_by_open_brace(input)?;
        let (input, _) = multispace1(input)?;
        let (input, parsed_item) = block(|input| item.parse(input))(input)?;

        Ok((input, (name, parsed_item)))
    }
}

pub fn named_block_repeated<'a, 'b, F, O, E>(
    block_tag: &'b str,
    mut item: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, (&'a str, Vec<O>), E>
where
    'b: 'a,
    F: Parser<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        named_block(
            block_tag,
            separated_list1(multispace1, |input| item.parse(input)),
        )(input)
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

fn recipe_ingredient<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    terminated(identifier1, tag(","))(input)
}

fn recipe_body<'a, E>(input: &'a str) -> IResult<&'a str, RecipeBody, E>
where
    E: ParseError<&'a str>,
{
    let (input, ingredients) = separated_list1(multispace1, recipe_ingredient)(input)?;
    let (input, _) = multispace1(input)?;

    let (input, result) = field_value("Result", ":", alphanumeric1)(input)?;
    let (input, _) = multispace1(input)?;

    let (input, time) = field_value("Time", ":", float)(input)?;
    let (input, _) = multispace1(input)?;

    let (input, category) = field_value("Category", ":", alphanumeric1)(input)?;
    let (input, _) = multispace1(input)?;

    let (input, need_to_be_learned) = field_value("NeedToBeLearn", ":", bool_value)(input)?;

    Ok((
        input,
        RecipeBody {
            ingredients,
            result,
            time,
            category,
            need_to_be_learned,
        },
    ))
}

pub fn recipe<'a, E>(input: &'a str) -> IResult<&'a str, Recipe, E>
where
    E: ParseError<&'a str>,
{
    Parser::into(named_block("recipe", recipe_body)).parse(input)
}

#[cfg(test)]
mod tests {
    use nom::{
        character::complete::{digit1, multispace0},
        combinator::map_res,
        sequence::pair,
    };

    use super::*;

    type Result<T> = IResult<&'static str, T, nom::error::Error<&'static str>>;

    #[test]
    fn parse_container_block() {
        let test_text = "container Foo { foo }";
        let expected = ("Foo", "foo");

        let block_res: Result<(&str, &str)> = named_block("container", tag("foo"))(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }

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

    #[test]
    fn parse_complex_item_repeated_block() {
        let test_text = "block_type BlockName {
 block_item 1
 block_item 2
 block_item 3
}";
        let expected = ("BlockName", vec![1, 2, 3]);

        let block_res: Result<(&str, Vec<u8>)> = named_block_repeated(
            "block_type",
            preceded(
                pair(tag("block_item"), space1),
                map_res(digit1, |s: &str| s.parse::<u8>()),
            ),
        )(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }

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

    #[test]
    fn parse_name_with_spaces() {
        let test_text = "item Name With Spaces { Nil }";
        let expected = ("Name With Spaces", "Nil");

        let block_res: Result<(&str, &str)> = named_block("item", tag("Nil"))(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_name_with_spaces_and_brace_on_following_line() {
        let test_text = "item Name With Spaces
{ Nil }";
        let expected = ("Name With Spaces", "Nil");

        let block_res: Result<(&str, &str)> = named_block("item", tag("Nil"))(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_unnamed_block() {
        let test_text = "imports
{
  Base
}";
        let expected = "Base";

        let block_res: Result<&str> = unnamed_block("imports", tag("Base"))(test_text);
        let (_, actual) = block_res.expect("failed to parse block");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_recipe() {
        let module_text = "
recipe Make Mildew Cure
{
  GardeningSprayEmpty,
  Base.Milk,

  Result:GardeningSprayMilk,
  Time:40.0,
  Category:Farming,
  NeedToBeLearn:true,
}
";
        let expected = Recipe::new(
            "Make Mildew Cure",
            vec!["GardeningSprayEmpty".to_string(), "Base.Milk".to_string()],
            "GardeningSprayMilk",
            40.0,
            "Farming",
            true,
        );

        let module_res: Result<Recipe> = preceded(multispace1, recipe)(module_text);
        let (_, actual) = module_res.expect("failed to parse module");

        assert_eq!(expected, actual);
    }
}
