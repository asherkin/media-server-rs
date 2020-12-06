use nom::branch::alt;
use nom::bytes::complete::take_till1;
use nom::character::complete::{char, line_ending};
use nom::combinator::eof;
use nom::error::ParseError;

pub use attribute_map::AttributeMap;

mod attribute_map;
pub mod attributes;
pub mod enums;
pub mod sdp;

pub enum EnumParseError {
    VariantNotFound,
}

fn value_field<'a, E>(input: &'a str) -> nom::IResult<&'a str, &str, E>
where
    E: ParseError<&'a str>,
{
    take_till1(|c| c == ' ' || c == '\r' || c == '\n')(input)
}

fn field_separator<'a, E>(input: &'a str) -> nom::IResult<&'a str, char, E>
where
    E: ParseError<&'a str>,
{
    char(' ')(input)
}

// Exists for output type compatibility with line_ending_or_eof for use in alt combinator
fn field_separator_str<'a, E>(input: &'a str) -> nom::IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    char(' ')(input)?;
    Ok((input, &input[..1]))
}

fn line_ending_or_eof<'a, E>(input: &'a str) -> nom::IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    alt((line_ending, eof))(input)
}
