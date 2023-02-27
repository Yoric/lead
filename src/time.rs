use anyhow::Context;
use chrono::{DateTime, Utc};

pub fn parse_utc(s: &str) -> Result<DateTime<Utc>, anyhow::Error> {
    dateparser::parse(s).context("Invalid date. Expected format: YYYY-MM-DD [HH:MM:SS]")
}
