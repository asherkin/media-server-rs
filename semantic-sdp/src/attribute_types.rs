use std::any::Any;
use std::str::FromStr;

use nom::character::complete::{char, not_line_ending};
use nom::combinator::{map_res, opt};
use nom::error::{ContextError, FromExternalError, ParseError};
use nom::multi::many1;
use nom::sequence::preceded;

use semantic_sdp_derive::SdpEnum;

use crate::{field_separator, line_ending_or_eof, value_field};

// TODO: Look into something like https://github.com/dtolnay/inventory
//       It would be annoying to miss a type here
pub(crate) fn parse_attribute<'a, E>(
    name: &str,
    input: &'a str,
) -> nom::IResult<&'a str, Box<dyn ParsableAttribute>, E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, attribute) = match name {
        IceUfrag::NAME => IceUfrag::parse_boxed(input),
        IceLite::NAME => IceLite::parse_boxed(input),
        Mid::NAME => Mid::parse_boxed(input),
        Group::NAME => Group::parse_boxed(input),
        _ => Option::<String>::parse_boxed(input),
    }?;

    Ok((input, attribute))
}

pub trait BaseAttribute: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T> BaseAttribute for T
where
    T: ParsableAttribute + Sized,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// https://github.com/rust-lang/rust/issues/41517
// pub trait AttributeParseError<I> = ParseError<I> + ContextError<I> + FromExternalError<I, crate::EnumParseError> + FromExternalError<I, std::num::ParseIntError>;

pub trait ParsableAttribute: BaseAttribute + std::fmt::Debug {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        Self: Sized,
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>;
    fn to_string(&self) -> Option<String>;

    fn parse_boxed<'a, E>(input: &'a str) -> nom::IResult<&'a str, Box<dyn ParsableAttribute>, E>
    where
        Self: Sized,
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, attribute) = Self::parse(input)?;

        Ok((input, Box::new(attribute)))
    }
}

impl ParsableAttribute for Option<String> {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, value) = opt(preceded(char(':'), not_line_ending))(input)?;
        let (input, _) = line_ending_or_eof(input)?;
        Ok((input, value.map(|s| s.to_owned())))
    }

    fn to_string(&self) -> Option<String> {
        self.to_owned()
    }
}

pub trait NamedAttribute: ParsableAttribute {
    const NAME: &'static str;
}

macro_rules! impl_value_sdp_attribute {
    ($attribute_name:literal, $type_name:ident) => {
        impl NamedAttribute for $type_name {
            const NAME: &'static str = $attribute_name;
        }
    };
}

macro_rules! impl_property_sdp_attribute {
    ($attribute_name:literal, $type_name:ident) => {
        impl ParsableAttribute for $type_name {
            fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
            where
                E: ParseError<&'a str>
                    + ContextError<&'a str>
                    + FromExternalError<&'a str, crate::EnumParseError>
                    + FromExternalError<&'a str, std::num::ParseIntError>,
            {
                let (input, _) = line_ending_or_eof(input)?;
                Ok((input, $type_name))
            }

            fn to_string(&self) -> Option<String> {
                None
            }
        }

        impl NamedAttribute for $type_name {
            const NAME: &'static str = $attribute_name;
        }
    };
}

#[derive(Debug, Eq, PartialEq)]
pub struct IceUfrag(pub String);

impl_value_sdp_attribute!("ice-ufrag", IceUfrag);

impl ParsableAttribute for IceUfrag {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, value) = not_line_ending(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let ice_ufrag = IceUfrag(value.to_owned());

        Ok((input, ice_ufrag))
    }

    fn to_string(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct IceLite;

impl_property_sdp_attribute!("ice-lite", IceLite);

#[derive(Debug, Eq, PartialEq)]
pub struct Mid(pub String);

impl_value_sdp_attribute!("mid", Mid);

impl ParsableAttribute for Mid {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, value) = not_line_ending(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let mid = Mid(value.to_owned());

        Ok((input, mid))
    }

    fn to_string(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

#[non_exhaustive]
#[derive(Debug, SdpEnum)]
pub enum GroupSemantics {
    // RFC 5888
    #[sdp("LS")]
    LipSynchronization,
    #[sdp("FID")]
    FlowIdentification,

    // draft-ietf-mmusic-sdp-bundle-negotiation
    #[sdp("BUNDLE")]
    Bundle,

    #[sdp(default)]
    Unknown(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Group {
    pub semantics: GroupSemantics,
    pub mids: Vec<String>,
}

impl_value_sdp_attribute!("group", Group);

impl ParsableAttribute for Group {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, semantics) = map_res(value_field, GroupSemantics::from_str)(input)?;
        let (input, mids) = many1(preceded(field_separator, value_field))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let group = Group {
            semantics,
            mids: mids.into_iter().map(|s| s.to_owned()).collect(),
        };

        Ok((input, group))
    }

    fn to_string(&self) -> Option<String> {
        Some(format!("{} {}", self.semantics, self.mids.join(" ")))
    }
}
