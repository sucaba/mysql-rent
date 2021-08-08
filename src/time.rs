use chrono::{
    NaiveDate, NaiveDateTime, NaiveTime,
};

/// Fixed `NaiveDateTime` for testing purposes
pub fn system_datetime() -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd(2021, 1, 2),
        NaiveTime::from_hms(10, 11, 12),
    )
}
