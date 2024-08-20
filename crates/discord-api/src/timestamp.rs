use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
pub use time::error::Parse as InnerError;
use time::{format_description::well_known::Rfc3339, serde::rfc3339, Duration, OffsetDateTime};

/// Discord's epoch starts at "2015-01-01T00:00:00+00:00"
const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Timestamp(#[serde(with = "rfc3339")] OffsetDateTime);

impl Timestamp {
    /// Creates a new [`Timestamp`] from the number of milliseconds since 1970.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is invalid.
    pub fn from_millis(millis: i64) -> Result<Self, InvalidTimestamp> {
        let x = OffsetDateTime::from_unix_timestamp_nanos(
            Duration::milliseconds(millis).whole_nanoseconds(),
        )
        .ok();
        x.map(Self).ok_or(InvalidTimestamp)
    }

    pub(crate) fn from_discord_id(id: u64) -> Self {
        // This can't fail because of the bit shifting
        // `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154
        // 07:35:11 GMT+0000"
        Self::from_millis(((id >> 22) + DISCORD_EPOCH) as i64).expect("can't fail")
    }

    /// Create a new `Timestamp` with the current date and time in UTC.
    #[must_use]
    pub fn now() -> Self {
        let x = OffsetDateTime::now_utc();
        Self(x)
    }

    /// Creates a new [`Timestamp`] from a UNIX timestamp (seconds since 1970).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value is invalid.
    pub fn from_unix_timestamp(secs: i64) -> Result<Self, InvalidTimestamp> {
        Self::from_millis(secs * 1000)
    }

    /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
    #[must_use]
    pub fn unix_timestamp(&self) -> i64 {
        let x = self.0.unix_timestamp();
        x
    }

    /// Parse a timestamp from an RFC 3339 date and time string.
    ///
    /// # Examples
    /// ```
    /// # use discord_api::timestamp::Timestamp;
    /// #
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
    /// let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
    ///
    /// assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
    /// assert!(Timestamp::parse("2016-04-30T11:18").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string is not a valid RFC 3339 date and time
    /// string.
    pub fn parse(input: &str) -> Result<Timestamp, ParseError> {
        let x = OffsetDateTime::parse(input, &Rfc3339).map_err(ParseError)?;
        Ok(Self(x))
    }

    #[must_use]
    pub fn to_rfc3339(&self) -> Option<String> {
        let x = self.0.format(&Rfc3339).ok()?;
        Some(x)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_rfc3339().ok_or(std::fmt::Error)?)
    }
}

impl std::ops::Deref for Timestamp {
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<OffsetDateTime> for Timestamp {
    fn from(dt: OffsetDateTime) -> Self {
        Self(dt)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        let x = OffsetDateTime::UNIX_EPOCH;
        Self(x)
    }
}

#[derive(Debug)]
pub struct InvalidTimestamp;

impl std::error::Error for InvalidTimestamp {}

impl fmt::Display for InvalidTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid UNIX timestamp value")
    }
}

/// Signifies the failure to parse the `Timestamp` from an RFC 3339 string.
#[derive(Debug)]
pub struct ParseError(InnerError);

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for Timestamp {
    type Err = ParseError;

    /// Parses an RFC 3339 date and time string such as
    /// `2016-04-30T11:18:25.796Z`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Timestamp::parse(s)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for Timestamp {
    type Error = ParseError;

    /// Parses an RFC 3339 date and time string such as
    /// `2016-04-30T11:18:25.796Z`.
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Timestamp::parse(s)
    }
}

impl From<&Timestamp> for Timestamp {
    fn from(ts: &Timestamp) -> Self {
        *ts
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn from_unix_timestamp() {
        let timestamp = Timestamp::from_unix_timestamp(1462015105).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1462015105);
        assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25Z");
    }
}
