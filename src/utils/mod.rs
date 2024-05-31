mod error;

pub use self::error::{Error, Result};

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

/// Encode a string to base64
pub fn b64u_encode(data: &str) -> String {
    base64_url::encode(data)
}

/// Decode a base64 to string
pub fn b64u_decode(b64u: &str) -> Result<String> {
    let decoded_string = base64_url::decode(b64u)
        // Ok -> Some, Err -> None
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(Error::FailToB64uDecode)?;

    Ok(decoded_string)
}
