use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::take_till1;
use nom::character::complete::{char, digit1, not_line_ending, one_of};
use nom::combinator::{eof, map, map_res, opt, recognize};
use nom::error::{ContextError, FromExternalError, ParseError};
use nom::multi::{many0, many1};
use nom::sequence::{preceded, separated_pair, terminated, tuple};

use crate::attributes::{parse_attribute, ParsableAttribute};
use crate::enums::*;
use crate::AttributeMap;
use crate::{field_separator, field_separator_str, line_ending_or_eof, value_field};
use std::collections::HashMap;

mod tests;

#[derive(Debug)]
pub struct Session {
    pub origin: Origin,
    pub name: Option<String>,
    pub information: Option<String>,
    pub uri: Option<String>,
    pub email_address: Option<String>,
    pub phone_number: Option<String>,
    pub connection: Option<Connection>,
    pub bandwidths: HashMap<BandwidthType, u64>,
    pub times: Vec<Time>,
    pub encryption_key: Option<String>,
    pub attributes: AttributeMap,
    pub media_descriptions: Vec<MediaDescription>,
}

impl Session {
    fn parse<'a, E>(input: &'a str) -> Result<Self, nom::Err<E>>
    where
        E: ParseError<&'a str>
            + ContextError<&'a str>
            + FromExternalError<&'a str, crate::EnumParseError>
            + FromExternalError<&'a str, std::convert::Infallible>
            + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        let (input, _) = char('v')(input)?;
        let (input, _) = char('=')(input)?;
        let (input, _) = char('0')(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        let (input, origin) = parse_origin_line(input)?;
        let (input, name) = parse_name_line(input)?;
        let (input, information) = opt(parse_generic_line('i'))(input)?;
        let (input, uri) = opt(parse_generic_line('u'))(input)?;
        let (input, email_address) = opt(parse_generic_line('e'))(input)?;
        let (input, phone_number) = opt(parse_generic_line('p'))(input)?;
        let (input, connection) = opt(parse_connection_line)(input)?;
        let (input, bandwidths) = many0(parse_bandwidth_line)(input)?;
        let (input, times) = many1(parse_time_lines)(input)?;
        let (input, encryption_key) = opt(parse_generic_line('k'))(input)?;
        let (input, parsed_attributes) = many0(parse_attribute_line)(input)?;
        let (input, media_descriptions) = many0(parse_media_description_lines)(input)?;
        eof(input)?;

        let mut attributes = AttributeMap::new();
        for (name, attribute) in parsed_attributes {
            attributes.append_boxed(name, attribute);
        }

        let session = Session {
            origin,
            name: name.map(|s| s.to_owned()),
            information: information.map(|s| s.to_owned()),
            uri: uri.map(|s| s.to_owned()),
            email_address: email_address.map(|s| s.to_owned()),
            phone_number: phone_number.map(|s| s.to_owned()),
            connection,
            bandwidths: bandwidths.into_iter().collect(),
            times,
            encryption_key: encryption_key.map(|s| s.to_owned()),
            attributes,
            media_descriptions,
        };

        Ok(session)
    }
}

impl FromStr for Session {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s) {
            Ok(result) => Ok(result),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(nom::error::convert_error(s, e)),
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }
}

impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "v=0\r\n")?;

        write!(f, "{}", self.origin)?;

        write!(f, "s={}\r\n", if let Some(name) = &self.name { name } else { "-" })?;

        if let Some(information) = &self.information {
            write!(f, "i={}\r\n", information)?;
        }

        if let Some(uri) = &self.uri {
            write!(f, "u={}\r\n", uri)?;
        }

        if let Some(email_address) = &self.email_address {
            write!(f, "e={}\r\n", email_address)?;
        }

        if let Some(phone_number) = &self.phone_number {
            write!(f, "p={}\r\n", phone_number)?;
        }

        if let Some(connection) = &self.connection {
            write!(f, "{}", connection)?;
        }

        for (kind, bandwidth) in &self.bandwidths {
            write!(f, "b={}:{}\r\n", kind, bandwidth)?;
        }

        for time in &self.times {
            write!(f, "{}", time)?;
        }

        if let Some(encryption_key) = &self.encryption_key {
            write!(f, "k={}\r\n", encryption_key)?;
        }

        write!(f, "{}", self.attributes)?;

        for media_description in &self.media_descriptions {
            write!(f, "{}", media_description)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Origin {
    pub username: Option<String>,
    pub session_id: u64,
    pub session_version: u64,
    pub network_type: NetworkType,
    pub address_type: AddressType,
    pub unicast_address: String,
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "o={} {} {} {} {} {}\r\n",
            if let Some(username) = &self.username {
                username
            } else {
                "-"
            },
            self.session_id,
            self.session_version,
            self.network_type,
            self.address_type,
            self.unicast_address,
        )
    }
}

// TODO: We don't currently parse the extra fields required for multicast addresses.
//       From an API PoV, the multiple c-line stuff would cause friction for unicast usage.
#[derive(Debug)]
pub struct Connection {
    pub network_type: NetworkType,
    pub address_type: AddressType,
    pub connection_address: String,
    // pub multicast_ttl: Option<u8>,
    // pub multicast_count: Option<u32>,
}

impl std::fmt::Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "c={} {} {}\r\n",
            self.network_type, self.address_type, self.connection_address,
        )
    }
}

#[derive(Debug)]
pub struct Time {
    pub start: u64,
    pub stop: u64,
    pub repeat_times: Vec<RepeatTime>,
    // This is part of the time section in draft-ietf-mmusic-rfc4566bis-37
    // We use a single Vec to represent the multiple entries in the z= line,
    // this isn't multiple z= lines.
    pub time_zone_adjustments: Vec<TimeZoneAdjustment>,
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "t={} {}\r\n", self.start, self.stop)?;

        for repeat_time in &self.repeat_times {
            write!(f, "{}", repeat_time)?;
        }

        let mut iter = self.time_zone_adjustments.iter();
        if let Some(time_zone_adjustment) = iter.next() {
            write!(f, "z={}", time_zone_adjustment)?;
            for time_zone_adjustment in iter {
                write!(f, " {}", time_zone_adjustment)?;
            }
            write!(f, "\r\n")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RepeatTime {
    pub repeat_interval: u64,
    pub active_duration: u64,
    pub offsets: Vec<u64>,
}

impl std::fmt::Display for RepeatTime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "r={} {}", self.repeat_interval, self.active_duration)?;
        for offset in &self.offsets {
            write!(f, " {}", offset)?;
        }
        write!(f, "\r\n")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct TimeZoneAdjustment {
    pub adjustment_time: u64,
    pub offset: i64,
}

impl std::fmt::Display for TimeZoneAdjustment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.adjustment_time, self.offset)
    }
}

#[derive(Debug)]
pub struct MediaDescription {
    pub kind: MediaType,
    pub port: u16,
    pub num_ports: Option<u16>,
    pub protocol: TransportProtocol,
    pub formats: Vec<String>,

    pub title: Option<String>,
    // TODO: A media section can have multiple connection lines with multicast addresses,
    //       We're just not supporting multicast currently.
    pub connection: Option<Connection>,
    pub bandwidths: HashMap<BandwidthType, u64>,
    pub encryption_key: Option<String>,
    pub attributes: AttributeMap,
}

impl std::fmt::Display for MediaDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let num_ports = if let Some(num_ports) = self.num_ports {
            format!("/{}", num_ports)
        } else {
            "".to_owned()
        };

        write!(
            f,
            "m={} {}{} {} {}\r\n",
            self.kind,
            self.port,
            num_ports,
            self.protocol,
            self.formats.join(" ")
        )?;

        if let Some(title) = &self.title {
            write!(f, "i={}\r\n", title)?;
        }

        if let Some(connection) = &self.connection {
            write!(f, "{}", connection)?;
        }

        for (kind, bandwidth) in &self.bandwidths {
            write!(f, "b={}:{}\r\n", kind, bandwidth)?;
        }

        if let Some(encryption_key) = &self.encryption_key {
            write!(f, "k={}\r\n", encryption_key)?;
        }

        write!(f, "{}", self.attributes)?;

        Ok(())
    }
}

fn parse_origin_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, Origin, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::convert::Infallible>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('o')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, username) = map(value_field, |v| if v != "-" { Some(v) } else { None })(input)?;
    let (input, _) = field_separator(input)?;
    let (input, session_id) = map_res(value_field, u64::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, session_version) = map_res(value_field, u64::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, network_type) = map_res(value_field, NetworkType::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, address_type) = map_res(value_field, AddressType::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, unicast_address) = value_field(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    let origin = Origin {
        username: username.map(|s| s.to_owned()),
        session_id,
        session_version,
        network_type,
        address_type,
        unicast_address: unicast_address.to_owned(),
    };

    Ok((input, origin))
}

fn parse_name_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, Option<&'a str>, E>
where
    E: ParseError<&'a str>,
{
    let (input, _) = char('s')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, name) = not_line_ending(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    let name = match name {
        " " | "-" => None,
        _ => Some(name),
    };

    Ok((input, name))
}

fn parse_connection_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, Connection, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::convert::Infallible>,
{
    let (input, _) = char('c')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, network_type) = map_res(value_field, NetworkType::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, address_type) = map_res(value_field, AddressType::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, connection_address) = value_field(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    let connection = Connection {
        network_type,
        address_type,
        connection_address: connection_address.to_owned(),
    };

    Ok((input, connection))
}

fn parse_bandwidth_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, (BandwidthType, u64), E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::convert::Infallible>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('b')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, kind) = map_res(
        take_till1(|c| c == ':' || c == '\r' || c == '\n'),
        BandwidthType::from_str,
    )(input)?;
    let (input, _) = char(':')(input)?;
    let (input, bandwidth) = map_res(value_field, u64::from_str)(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    Ok((input, (kind, bandwidth)))
}

fn parse_time_lines<'a, E>(input: &'a str) -> nom::IResult<&'a str, Time, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('t')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, start) = map_res(value_field, u64::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, stop) = map_res(value_field, u64::from_str)(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    let (input, repeat_times) = many0(parse_repeat_time_line)(input)?;
    let (input, time_zone_adjustments) = opt(parse_time_zone_adjustment_line)(input)?;

    let time = Time {
        start,
        stop,
        repeat_times,
        time_zone_adjustments: time_zone_adjustments.unwrap_or_else(Vec::new),
    };

    Ok((input, time))
}

fn parse_repeat_time_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, RepeatTime, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('r')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, repeat_interval) = sdp_time_field(input)?;
    let (input, _) = field_separator(input)?;
    let (input, active_duration) = sdp_time_field(input)?;
    let (input, offsets) = many1(preceded(field_separator, sdp_time_field))(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    let repeat_time = RepeatTime {
        repeat_interval,
        active_duration,
        offsets,
    };

    Ok((input, repeat_time))
}

fn parse_time_zone_adjustment_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, Vec<TimeZoneAdjustment>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('z')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, adjustments) = many1(terminated(
        map(
            separated_pair(sdp_time_field, field_separator, sdp_time_field),
            |(adjustment_time, offset)| TimeZoneAdjustment {
                adjustment_time,
                offset,
            },
        ),
        alt((field_separator_str, line_ending_or_eof)),
    ))(input)?;

    Ok((input, adjustments))
}

fn parse_attribute_line<'a, E>(input: &'a str) -> nom::IResult<&'a str, (String, Box<dyn ParsableAttribute>), E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::convert::Infallible>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('a')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, name) = map(
        take_till1(|c| c == ':' || c == '\r' || c == '\n'),
        str::to_ascii_lowercase,
    )(input)?;
    let (input, attribute) = parse_attribute(&name, input)?;

    Ok((input, (name, attribute)))
}

fn parse_media_description_lines<'a, E>(input: &'a str) -> nom::IResult<&'a str, MediaDescription, E>
where
    E: ParseError<&'a str>
        + ContextError<&'a str>
        + FromExternalError<&'a str, crate::EnumParseError>
        + FromExternalError<&'a str, std::convert::Infallible>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = char('m')(input)?;
    let (input, _) = char('=')(input)?;
    let (input, kind) = map_res(value_field, MediaType::from_str)(input)?;
    let (input, _) = field_separator(input)?;
    let (input, port) = map_res(take_till1(|c| c == ' ' || c == '/'), u16::from_str)(input)?;
    let (input, num_ports) = opt(map_res(preceded(char('/'), value_field), u16::from_str))(input)?;
    let (input, _) = field_separator(input)?;
    let (input, protocol) = map_res(value_field, TransportProtocol::from_str)(input)?;
    let (input, formats) = many1(preceded(field_separator, value_field))(input)?;
    let (input, _) = line_ending_or_eof(input)?;

    let (input, title) = opt(parse_generic_line('i'))(input)?;
    let (input, connection) = opt(parse_connection_line)(input)?;
    let (input, bandwidths) = many0(parse_bandwidth_line)(input)?;
    let (input, encryption_key) = opt(parse_generic_line('k'))(input)?;
    let (input, parsed_attributes) = many0(parse_attribute_line)(input)?;

    let mut attributes = AttributeMap::new();
    for (name, attribute) in parsed_attributes {
        attributes.append_boxed(name, attribute);
    }

    let media_description = MediaDescription {
        kind,
        port,
        num_ports,
        protocol,
        formats: formats.into_iter().map(|s| s.to_owned()).collect(),
        title: title.map(|s| s.to_owned()),
        connection,
        bandwidths: bandwidths.into_iter().collect(),
        encryption_key: encryption_key.map(|s| s.to_owned()),
        attributes,
    };

    Ok((input, media_description))
}

fn parse_generic_line<'a, E>(tag: char) -> impl Fn(&'a str) -> nom::IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    move |input| {
        let (input, _) = char(tag)(input)?;
        let (input, _) = char('=')(input)?;
        let (input, value) = not_line_ending(input)?;
        let (input, _) = line_ending_or_eof(input)?;

        Ok((input, value))
    }
}

fn sdp_time_field<'a, O, E>(input: &'a str) -> nom::IResult<&'a str, O, E>
where
    O: std::ops::Mul<Output = O> + std::convert::From<u32> + std::str::FromStr<Err = std::num::ParseIntError>,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, number) = map_res(recognize(tuple((opt(char('-')), digit1))), O::from_str)(input)?;
    let (input, unit) = opt(one_of("dhms"))(input)?;

    let multiplier: u32 = match unit {
        Some('d') => 86400,
        Some('h') => 3600,
        Some('m') => 60,
        Some('s') | None => 1,
        _ => unreachable!(),
    };

    Ok((input, number * O::from(multiplier)))
}
