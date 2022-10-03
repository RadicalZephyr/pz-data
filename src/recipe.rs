use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace1},
    error::ParseError,
    multi::separated_list1,
    number::complete::float,
    sequence::terminated,
    IResult, Parser,
};

use crate::{bool_value, field_value, identifier1, named_block};

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
    use nom::sequence::preceded;

    use super::*;

    type Result<T> = nom::IResult<&'static str, T, nom::error::Error<&'static str>>;

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
