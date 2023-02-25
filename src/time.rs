use anyhow::Context;
use chrono::{Utc, DateTime};

pub fn parse_utc(s: &str) -> Result<DateTime<Utc>, anyhow::Error> {
    dateparser::parse(s)
        .context("Invalid date")
}