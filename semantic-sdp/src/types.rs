use std::borrow::Borrow;
use std::str::FromStr;

#[derive(Clone, Eq, PartialEq)]
pub struct CertificateFingerprint(pub Vec<u8>);

impl FromStr for CertificateFingerprint {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fingerprint: Result<Vec<_>, _> = s.split(':').map(|s| u8::from_str_radix(s, 16)).collect();

        Ok(Self(fingerprint?))
    }
}

impl std::fmt::Display for CertificateFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fingerprint = self
            .0
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");

        f.write_str(&fingerprint)
    }
}

impl std::fmt::Debug for CertificateFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ssrc(pub u32);

impl FromStr for Ssrc {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u32::from_str(s)?))
    }
}

impl std::fmt::Display for Ssrc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u32> for Ssrc {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Mid(pub String);

impl From<&str> for Mid {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl FromStr for Mid {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl std::fmt::Display for Mid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// TODO: I'm not sure if this is the right trait to implement
impl Borrow<str> for Mid {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rid(pub String);

impl From<&str> for Rid {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl FromStr for Rid {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl std::fmt::Display for Rid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// TODO: I'm not sure if this is the right trait to implement
impl Borrow<str> for Rid {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PayloadType(pub u8);

impl FromStr for PayloadType {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u8::from_str(s)?))
    }
}

impl std::fmt::Display for PayloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
