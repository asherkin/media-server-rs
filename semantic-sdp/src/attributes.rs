use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use nom::bytes::complete::{tag_no_case, take_till1};
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
        BundleOnly::NAME => BundleOnly::parse_boxed(input),
        Candidate::NAME => Candidate::parse_boxed(input),
        EndOfCandidates::NAME => EndOfCandidates::parse_boxed(input),
        ExtensionMap::NAME => ExtensionMap::parse_boxed(input),
        ExtensionMapAllowMixed::NAME => ExtensionMapAllowMixed::parse_boxed(input),
        Fingerprint::NAME => Fingerprint::parse_boxed(input),
        FormatParameters::NAME => FormatParameters::parse_boxed(input),
        Group::NAME => Group::parse_boxed(input),
        IceLite::NAME => IceLite::parse_boxed(input),
        IceOptions::NAME => IceOptions::parse_boxed(input),
        IcePwd::NAME => IcePwd::parse_boxed(input),
        IceUfrag::NAME => IceUfrag::parse_boxed(input),
        Inactive::NAME => Inactive::parse_boxed(input),
        MaxPacketTime::NAME => MaxPacketTime::parse_boxed(input),
        MediaStreamId::NAME => MediaStreamId::parse_boxed(input),
        MediaStreamIdSemantic::NAME => MediaStreamIdSemantic::parse_boxed(input),
        Mid::NAME => Mid::parse_boxed(input),
        PacketTime::NAME => PacketTime::parse_boxed(input),
        ReceiveOnly::NAME => ReceiveOnly::parse_boxed(input),
        Rtcp::NAME => Rtcp::parse_boxed(input),
        RtcpFeedback::NAME => RtcpFeedback::parse_boxed(input),
        RtcpMux::NAME => RtcpMux::parse_boxed(input),
        RtcpReducedSize::NAME => RtcpReducedSize::parse_boxed(input),
        RtpMap::NAME => RtpMap::parse_boxed(input),
        SendOnly::NAME => SendOnly::parse_boxed(input),
        SendReceive::NAME => SendReceive::parse_boxed(input),
        Setup::NAME => Setup::parse_boxed(input),
        SsrcAttribute::NAME => SsrcAttribute::parse_boxed(input),
        SsrcGroup::NAME => SsrcGroup::parse_boxed(input),
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
        #[derive(Debug, Clone, Eq, PartialEq)]
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
        #[derive(Debug, Clone, Eq, PartialEq)]
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
        #[derive(Debug, Clone, Eq, PartialEq)]
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

// RFC 4566
declare_simple_value_sdp_attribute!("ptime", PacketTime, u32);
declare_simple_value_sdp_attribute!("maxptime", MaxPacketTime, u32);

// RFC 4566
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RtpMap {
    pub payload: u8,
    pub name: String,
    pub clock: u32,
    pub channels: Option<u8>,
}

impl_value_sdp_attribute!("rtpmap", RtpMap);

impl ParsableAttribute for RtpMap {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, payload) = map_res(value_field, u8::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, name) = take_till1(|c| c == ' ' || c == '/')(input)?;
        let (input, _) = char('/')(input)?;
        let (input, clock) = map_res(
            take_till1(|c| c == ' ' || c == '\r' || c == '\n' || c == '/'),
            u32::from_str,
        )(input)?;
        let (input, channels) = opt(preceded(char('/'), map_res(value_field, u8::from_str)))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let rtpmap = RtpMap {
            payload,
            name: name.to_owned(),
            clock,
            channels,
        };

        Ok((input, rtpmap))
    }

    fn to_string(&self) -> Option<String> {
        let channels = match &self.channels {
            Some(channels) => format!("/{}", channels),
            None => "".to_owned(),
        };

        Some(format!("{} {}/{}{}", self.payload, self.name, self.clock, channels))
    }
}

// RFC 4566
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatParameters {
    pub payload: u8,
    pub parameters: String,
}

impl_value_sdp_attribute!("fmtp", FormatParameters);

impl ParsableAttribute for FormatParameters {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, payload) = map_res(value_field, u8::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, parameters) = not_line_ending(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let fmtp = FormatParameters {
            payload,
            parameters: parameters.to_owned(),
        };

        Ok((input, fmtp))
    }

    fn to_string(&self) -> Option<String> {
        Some(format!("{} {}", self.payload, self.parameters))
    }
}

// RFC 4566
declare_property_sdp_attribute!("recvonly", ReceiveOnly);
declare_property_sdp_attribute!("sendrecv", SendReceive);
declare_property_sdp_attribute!("sendonly", SendOnly);
declare_property_sdp_attribute!("inactive", Inactive);

// RFC 5245
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Candidate {
    // In practice the foundation is always an integer
    pub foundation: String,
    pub component: u16,
    pub transport: IceTransportType,
    pub priority: u32,
    pub address: String,
    pub port: u16,
    pub kind: IceCandidateType,
    pub rel_addr: Option<String>,
    pub rel_port: Option<u16>,
    pub unknown: HashMap<String, String>,

    // RFC 6544
    pub tcp_type: Option<IceTcpType>,
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

// RFC 5245
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IceOptions(pub HashSet<IceOption>);

impl_value_sdp_attribute!("ice-options", IceOptions);

impl ParsableAttribute for IceOptions {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, options) = separated_list1(field_separator, map_res(value_field, IceOption::from_str))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let options = options.into_iter().collect();

        Ok((input, IceOptions(options)))
    }

    fn to_string(&self) -> Option<String> {
        let options: Vec<_> = self.0.iter().map(|o| o.to_string()).collect();

        Some(options.join(" "))
    }
}

// RFC 4145 / RFC 5763
declare_simple_value_sdp_attribute!("setup", Setup, SetupRole);

// RFC 4572 / RFC 5763
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Fingerprint {
    pub hash_function: FingerprintHashFunction,
    pub fingerprint: Vec<u8>,
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

// RFC 3388 / RFC 5888
declare_simple_value_sdp_attribute!("mid", Mid, String);

// RFC 3388 / RFC 5888
#[derive(Debug, Clone, Eq, PartialEq)]
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

// RFC 5576
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SsrcAttribute {
    pub ssrc: u32,
    pub name: String,
    pub value: Option<String>,
}

impl_value_sdp_attribute!("ssrc", SsrcAttribute);

impl ParsableAttribute for SsrcAttribute {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, ssrc) = map_res(value_field, u32::from_str)(input)?;
        let (input, _) = field_separator(input)?;
        let (input, name) = take_till1(|c| c == ':')(input)?;
        let (input, value) = opt(preceded(char(':'), not_line_ending))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let ssrc_attribute = SsrcAttribute {
            ssrc,
            name: name.to_owned(),
            value: value.map(|s| s.to_owned()),
        };

        Ok((input, ssrc_attribute))
    }

    fn to_string(&self) -> Option<String> {
        let value = match &self.value {
            Some(value) => format!(":{}", value),
            None => "".to_owned(),
        };

        Some(format!("{} {}{}", self.ssrc, self.name, value))
    }
}

// RFC 5576
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SsrcGroup {
    pub semantics: SsrcGroupSemantics,
    pub ssrcs: Vec<u32>,
}

impl_value_sdp_attribute!("ssrc-group", SsrcGroup);

impl ParsableAttribute for SsrcGroup {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, semantics) = map_res(value_field, SsrcGroupSemantics::from_str)(input)?;
        let (input, ssrcs) = many1(preceded(field_separator, map_res(value_field, u32::from_str)))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let ssrc_group = SsrcGroup { semantics, ssrcs };

        Ok((input, ssrc_group))
    }

    fn to_string(&self) -> Option<String> {
        let ssrcs: Vec<_> = self.ssrcs.iter().map(|s| s.to_string()).collect();

        Some(format!("{} {}", self.semantics, ssrcs.join(" ")))
    }
}

// RFC 3605
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Rtcp {
    pub port: u16,
    pub network_type: Option<NetworkType>,
    pub address_type: Option<AddressType>,
    pub connection_address: Option<String>,
}

impl_value_sdp_attribute!("rtcp", Rtcp);

impl ParsableAttribute for Rtcp {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, port) = map_res(value_field, u16::from_str)(input)?;
        let (input, has_address) = opt(field_separator)(input)?;
        let (input, network_type, address_type, connection_address) = if has_address.is_some() {
            let (input, network_type) = map_res(value_field, NetworkType::from_str)(input)?;
            let (input, _) = field_separator(input)?;
            let (input, address_type) = map_res(value_field, AddressType::from_str)(input)?;
            let (input, _) = field_separator(input)?;
            let (input, connection_address) = value_field(input)?;
            (input, Some(network_type), Some(address_type), Some(connection_address))
        } else {
            (input, None, None, None)
        };
        let (input, _) = line_ending_or_eof(input)?;

        let rtcp = Rtcp {
            port,
            network_type,
            address_type,
            connection_address: connection_address.map(|s| s.to_owned()),
        };

        Ok((input, rtcp))
    }

    fn to_string(&self) -> Option<String> {
        let address = if let Some(connection_address) = &self.connection_address {
            let network_type = self.network_type.as_ref().unwrap();
            let address_type = self.address_type.as_ref().unwrap();
            format!(" {} {} {}", network_type, address_type, connection_address)
        } else {
            "".to_owned()
        };

        Some(format!("{}{}", self.port, address))
    }
}

// RFC 4585
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RtcpFeedback {
    // None is used for `*`
    pub payload: Option<u8>,
    // TODO: Do we want to define these further?
    //       We probably do want some enums for all the various modes.
    //       An enum with data could represent the id/param split quite nicely.
    pub id: String,
    pub param: Option<String>,
}

impl_value_sdp_attribute!("rtcp-fb", RtcpFeedback);

impl ParsableAttribute for RtcpFeedback {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, payload) = opt(map_res(value_field, u8::from_str))(input)?;
        let (input, _) = opt(field_separator)(input)?;
        let (input, id) = value_field(input)?;
        let (input, param) = opt(preceded(field_separator, value_field))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let rtcp_feedback = RtcpFeedback {
            payload,
            id: id.to_owned(),
            param: param.map(|s| s.to_owned()),
        };

        Ok((input, rtcp_feedback))
    }

    fn to_string(&self) -> Option<String> {
        let payload = match &self.payload {
            Some(payload) => payload.to_string(),
            None => "*".to_owned(),
        };

        let param = match &self.param {
            Some(param) => format!(" {}", param),
            None => "".to_owned(),
        };

        Some(format!("{} {}{}", payload, self.id, param))
    }
}

// RFC 5506
declare_property_sdp_attribute!("rtcp-rsize", RtcpReducedSize);

// RFC 5761
declare_property_sdp_attribute!("rtcp-mux", RtcpMux);

// RFC 8285
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExtensionMap {
    pub id: u16,
    pub direction: Option<ExtensionMapDirection>,
    pub extension: String,
    pub attributes: Vec<String>,
}

impl_value_sdp_attribute!("extmap", ExtensionMap);

impl ParsableAttribute for ExtensionMap {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, id) = map_res(take_till1(|c| c == ' ' || c == '/'), u16::from_str)(input)?;
        let (input, direction) = opt(preceded(
            char('/'),
            map_res(value_field, ExtensionMapDirection::from_str),
        ))(input)?;
        let (input, _) = field_separator(input)?;
        let (input, extension) = value_field(input)?;
        let (input, attributes) = many0(preceded(field_separator, value_field))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let extension_map = ExtensionMap {
            id,
            direction,
            extension: extension.to_owned(),
            attributes: attributes.into_iter().map(|s| s.to_owned()).collect(),
        };

        Ok((input, extension_map))
    }

    fn to_string(&self) -> Option<String> {
        let direction = match &self.direction {
            Some(direction) => format!("/{}", direction),
            None => "".to_owned(),
        };

        let attributes = if !self.attributes.is_empty() {
            format!(" {}", self.attributes.join(" "))
        } else {
            "".to_owned()
        };

        Some(format!("{}{} {}{}", self.id, direction, self.extension, attributes))
    }
}

// RFC 8285
declare_property_sdp_attribute!("extmap-allow-mixed", ExtensionMapAllowMixed);

// draft-ietf-mmusic-sdp-bundle-negotiation
declare_property_sdp_attribute!("bundle-only", BundleOnly);

// draft-ietf-mmusic-trickle-ice-sip
declare_property_sdp_attribute!("end-of-candidates", EndOfCandidates);

// draft-ietf-mmusic-msid
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MediaStreamId {
    pub id: String,
    pub appdata: Option<String>,
}

impl_value_sdp_attribute!("msid", MediaStreamId);

impl ParsableAttribute for MediaStreamId {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, id) = value_field(input)?;
        let (input, appdata) = opt(preceded(field_separator, value_field))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let msid = MediaStreamId {
            id: id.to_owned(),
            appdata: appdata.map(|s| s.to_owned()),
        };

        Ok((input, msid))
    }

    fn to_string(&self) -> Option<String> {
        let appdata = match &self.appdata {
            Some(appdata) => format!(" {}", appdata),
            None => "".to_owned(),
        };

        Some(format!("{}{}", self.id, appdata))
    }
}

// draft-ietf-mmusic-msid (removed in draft 09)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MediaStreamIdSemantic {
    // "WMS" is practically the only value for this.
    pub semantic: String,
    pub msids: Vec<String>,
}

impl_value_sdp_attribute!("msid-semantic", MediaStreamIdSemantic);

impl ParsableAttribute for MediaStreamIdSemantic {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char(':')(input)?;
        let (input, _) = opt(field_separator)(input)?;
        let (input, semantic) = value_field(input)?;
        let (input, msids) = many0(preceded(field_separator, value_field))(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let msid_semantic = MediaStreamIdSemantic {
            semantic: semantic.to_owned(),
            msids: msids.into_iter().map(|s| s.to_owned()).collect(),
        };

        Ok((input, msid_semantic))
    }

    fn to_string(&self) -> Option<String> {
        // The extra space before the semantic seems to be expected by most implementations.
        Some(format!(" {} {}", self.semantic, self.msids.join(" ")))
    }
}
