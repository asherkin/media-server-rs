use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, hex_digit1, not_line_ending};
use nom::combinator::{map_res, opt};
use nom::error::{ContextError, FromExternalError, ParseError};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{preceded, separated_pair};

use crate::enums::*;
use crate::{field_separator, line_ending_or_eof, value_field};

pub(crate) fn parse_attribute<'a, E>(name: &str, input: &'a str) -> nom::IResult<&'a str, Box<dyn ParsableAttribute>, E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, attribute) = match name {
        Candidate::NAME => Candidate::parse_boxed(input),
        Fingerprint::NAME => Fingerprint::parse_boxed(input),
        Group::NAME => Group::parse_boxed(input),
        IceLite::NAME => IceLite::parse_boxed(input),
        IcePwd::NAME => IcePwd::parse_boxed(input),
        IceUfrag::NAME => IceUfrag::parse_boxed(input),
        Mid::NAME => Mid::parse_boxed(input),
        Setup::NAME => Setup::parse_boxed(input),
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

        #[cfg(test)]
        paste::paste! {
            #[test]
            fn [<parse_test_ $type_name:lower>]() {
                // This test ensures that the attribute name has been added to
                // the match in parse_attribute

                type E<'a> = (&'a str, nom::error::ErrorKind);
                let result = parse_attribute::<E>($attribute_name, "");

                let attribute = match result {
                    Ok((_, attribute)) => attribute,
                    _ => return,
                };

                if let Some(_) = attribute.as_any().downcast_ref::<Option<String>>() {
                    panic!("Attribute {} has not been included in parse_attribute match", stringify!($type_name));
                }
            }
        }
    };
}

macro_rules! declare_property_sdp_attribute {
    ($attribute_name:literal, $type_name:ident) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $type_name;

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

        impl_value_sdp_attribute!($attribute_name, $type_name);
    };
}

macro_rules! declare_simple_value_sdp_attribute {
    ($attribute_name:literal, $type_name:ident, String) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $type_name(pub String);

        impl ParsableAttribute for $type_name {
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
                Ok((input, Self(value.to_owned())))
            }

            fn to_string(&self) -> Option<String> {
                Some(self.0.to_string())
            }
        }

        impl_value_sdp_attribute!($attribute_name, $type_name);
    };
    ($attribute_name:literal, $type_name:ident, $value_type:ident) => {
        #[derive(Debug, Eq, PartialEq)]
        pub struct $type_name(pub $value_type);

        impl ParsableAttribute for $type_name {
            fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
            where
                E: ParseError<&'a str>
                    + ContextError<&'a str>
                    + FromExternalError<&'a str, crate::EnumParseError>
                    + FromExternalError<&'a str, std::num::ParseIntError>,
            {
                let (input, _) = char(':')(input)?;
                let (input, value) = map_res(not_line_ending, $value_type::from_str)(input)?;
                let (input, _) = line_ending_or_eof(input)?;
                Ok((input, Self(value)))
            }

            fn to_string(&self) -> Option<String> {
                Some(self.0.to_string())
            }
        }

        impl_value_sdp_attribute!($attribute_name, $type_name);
    };
}

// RFC 5245
#[derive(Debug, Eq, PartialEq)]
struct Candidate {
    // In practice the foundation is always an integer
    foundation: String,
    component: u16,
    transport: IceTransportType,
    priority: u32,
    address: String,
    port: u16,
    kind: IceCandidateType,
    rel_addr: Option<String>,
    rel_port: Option<u16>,
    unknown: HashMap<String, String>,

    // RFC 6544
    tcp_type: Option<IceTcpType>,
}

impl_value_sdp_attribute!("candidate", Candidate);

impl ParsableAttribute for Candidate {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, foundation) = value_field(input)?;
        let (input, _) = field_separator(input)?;
        let (input, component) = map_res(value_field, u16::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, transport) = map_res(value_field, IceTransportType::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, priority) = map_res(value_field, u32::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, address) = value_field(input)?;
        let (input, _) = field_separator(input)?;
        let (input, port) = map_res(value_field, u16::from_str)(input)?;
        let (input, kind) = preceded(tag_no_case(" typ "), map_res(value_field, IceCandidateType::from_str))(input)?;
        let (input, rel_addr) = opt(preceded(tag_no_case(" raddr "), value_field))(input)?;
        let (input, rel_port) = opt(preceded(tag_no_case(" rport "), map_res(value_field, u16::from_str)))(input)?;
        let (input, unknown) = many0(preceded(
            field_separator,
            separated_pair(value_field, field_separator, value_field),
        ))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let mut unknown: HashMap<&str, &str> = unknown.into_iter().collect();

        let tcp_type = unknown
            .remove("tcptype")
            .map(|v| match IceTcpType::from_str(v) {
                Ok(v) => Ok(v),
                Err(e) => Err(nom::Err::Error(E::from_external_error(
                    v,
                    nom::error::ErrorKind::MapRes,
                    e,
                ))),
            })
            .transpose()?;

        let unknown: HashMap<String, String> = unknown
            .into_iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v.to_owned()))
            .collect();

        let candidate = Candidate {
            foundation: foundation.to_owned(),
            component,
            transport,
            priority,
            address: address.to_owned(),
            port,
            kind,
            rel_addr: rel_addr.map(|s| s.to_owned()),
            rel_port,
            unknown,
            tcp_type,
        };

        Ok((input, candidate))
    }

    fn to_string(&self) -> Option<String> {
        let mut named = vec![format!("typ {}", self.kind)];

        if let Some(rel_addr) = &self.rel_addr {
            named.push(format!("raddr {}", rel_addr));
        }

        if let Some(rel_port) = &self.rel_port {
            named.push(format!("rport {}", rel_port));
        }

        if let Some(tcp_type) = &self.tcp_type {
            named.push(format!("tcptype {}", tcp_type));
        }

        named.extend(self.unknown.iter().map(|(k, v)| format!("{} {}", k, v)));

        let value = format!(
            "{} {} {} {} {} {} {}",
            self.foundation,
            self.component,
            self.transport,
            self.priority,
            self.address,
            self.port,
            named.join(" "),
        );

        Some(value)
    }
}

// RFC 5245
declare_property_sdp_attribute!("ice-lite", IceLite);

// RFC 5245
declare_simple_value_sdp_attribute!("ice-pwd", IcePwd, String);

// RFC 5245
declare_simple_value_sdp_attribute!("ice-ufrag", IceUfrag, String);

// RFC 4145 / RFC 5763
declare_simple_value_sdp_attribute!("setup", Setup, SetupRole);

// RFC 4572 / RFC 5763
#[derive(Debug, Eq, PartialEq)]
struct Fingerprint {
    hash_function: FingerprintHashFunction,
    fingerprint: Vec<u8>,
}

impl_value_sdp_attribute!("fingerprint", Fingerprint);

impl ParsableAttribute for Fingerprint {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, hash_function) = map_res(value_field, FingerprintHashFunction::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, fingerprint) =
            separated_list1(char(':'), map_res(hex_digit1, |s| u8::from_str_radix(s, 16)))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let fingerprint = Fingerprint {
            hash_function,
            fingerprint,
        };

        Ok((input, fingerprint))
    }

    fn to_string(&self) -> Option<String> {
        let fingerprint = self
            .fingerprint
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");

        Some(format!("{} {}", self.hash_function, fingerprint))
    }
}

declare_simple_value_sdp_attribute!("mid", Mid, String);

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
