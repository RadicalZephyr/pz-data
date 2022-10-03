use nom::{
    bytes::complete::tag,
    character::complete::{multispace1, space1},
    error::ParseError,
    multi::separated_list1,
    sequence::{delimited, pair},
    AsChar, IResult, InputLength, InputTake, InputTakeAtPosition, Parser, Slice,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    use nom::{
        character::complete::digit1,
        combinator::map_res,
        sequence::{pair, preceded},
    };

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
        let expected = ("Base", vec!["foo", "foo", "foo"]);

        let block_res: Result<(&'static str, Vec<&'static str>)> =
            named_block_repeated("module", tag("foo"))(test_text);
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
}
