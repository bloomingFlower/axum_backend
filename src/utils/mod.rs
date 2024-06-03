mod error;

pub use self::error::{Error, Result};

use base64::engine::{general_purpose, Engine};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

/// Return the current time in UTC
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Format the time in UTC as a string
// inline(always) is a hint to the compiler to inline the function
#[inline(always)]
pub fn format_time(time: OffsetDateTime) -> String {
    // Rfc3339 is a predefined format(e.g., 2002-10-02T10:00:00-05:00)
    time.format(&Rfc3339).expect("Wrong time format")
}

/// Return the current time in UTC plus the number of duration seconds as a string
pub fn now_utc_plus_sec_str(sec: i64) -> String {
    let now = now_utc();
    let future = now + Duration::seconds(sec);
    format_time(future)
}

/// Parse a string as a UTC time
pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(moment, &Rfc3339).map_err(|_| Error::DateFailParse(moment.to_string()))
}

pub fn b64u_encode(content: impl AsRef<[u8]>) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(content)
}

pub fn b64u_decode(b64u: &str) -> Result<Vec<u8>> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(b64u)
        .map_err(|_| Error::FailToB64uDecode)
}

pub fn b64u_decode_to_string(b64u: &str) -> Result<String> {
    b64u_decode(b64u)
        .ok()
        .and_then(|r| String::from_utf8(r).ok())
        .ok_or(Error::FailToB64uDecode)
}
