use chrono::{Datelike, Days, Local, NaiveDate};

fn midnight_of(date: NaiveDate) -> i64 {
    date.and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
        .and_local_timezone(Local)
        .single()
        .expect("midnight is unambiguous in the local timezone")
        .timestamp()
}

/// Unix timestamp for midnight today in the local timezone. Shared by
/// anything that needs to filter data down to "today" (the CLI's --today
/// flag, the desktop app's daily view), since getting local midnight
/// right (as opposed to UTC midnight) matters for the result to match
/// what the user actually means by "today."
pub fn today_start_timestamp() -> i64 {
    midnight_of(Local::now().date_naive())
}

/// Unix timestamp for midnight on the most recent Monday, for "this
/// week" views.
pub fn week_start_timestamp() -> i64 {
    let today = Local::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday() as u64;
    let monday = today
        .checked_sub_days(Days::new(days_since_monday))
        .expect("subtracting a few days from today stays in range");
    midnight_of(monday)
}

/// Unix timestamp for midnight on the first of the current month, for
/// "this month" views.
pub fn month_start_timestamp() -> i64 {
    let today = Local::now().date_naive();
    let first_of_month =
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("day 1 is always valid");
    midnight_of(first_of_month)
}

/// A named reporting period, translated to the unix timestamp its data
/// should be filtered from. "All" has no lower bound.
pub enum Period {
    Today,
    Week,
    Month,
    All,
}

impl Period {
    pub fn since(&self) -> Option<i64> {
        match self {
            Period::Today => Some(today_start_timestamp()),
            Period::Week => Some(week_start_timestamp()),
            Period::Month => Some(month_start_timestamp()),
            Period::All => None,
        }
    }

    pub fn parse(s: &str) -> Option<Period> {
        match s {
            "today" => Some(Period::Today),
            "week" => Some(Period::Week),
            "month" => Some(Period::Month),
            "all" => Some(Period::All),
            _ => None,
        }
    }
}
