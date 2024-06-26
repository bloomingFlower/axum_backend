use time::{Duration, OffsetDateTime};

pub use time::format_description::well_known::Rfc3339;

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
    OffsetDateTime::parse(moment, &Rfc3339).map_err(|_| Error::FailToDateParse(moment.to_string()))
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FailToDateParse(String),
}

impl core::fmt::Display for Error {
    fn fmt(
        &self, 
        fmt: &mut core::fmt::Formatter
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
