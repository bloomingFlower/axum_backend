mod error;

pub use self::error::{Error, Result};

use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

pub fn now_utc() -> OffsetDateTime {
    let now = OffsetDateTime::now_utc();
    now
}

#[inline(always)]
pub fn format_time(time: OffsetDateTime) -> String {
    time.format(&Rfc3339).unwrap()
}

pub fn now_utc_plus_sec_str(sec: i64) -> String {
    let now = now_utc();
    let future = now + Duration::seconds(sec);
    format_time(future)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(moment, &Rfc3339).map_err(|_| Error::DateFailParse(moment.to_string()))
}

pub fn b64u_encode(data: &str) -> String {
    base64_url::encode(data)
}

pub fn b64u_decode(b64u: &str) -> Result<String> {
    let decoded_string = base64_url::decode(b64u)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(Error::FailToB64uDecode)?;

    Ok(decoded_string)
}
