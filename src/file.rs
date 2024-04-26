use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use color_eyre::eyre::Context;
use regex::Regex;

use crate::config::RetentionFilePattern;
use crate::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RetentionFile {
    pub filename: String,
    pub date: DateTime<Utc>,
}

impl RetentionFile {
    pub fn new(filename: String, pattern: &RetentionFilePattern) -> Result<Self> {
        Ok(Self {
            date: pattern.parse(&filename)?,
            filename,
        })
    }
}

impl RetentionFilePattern {
    pub fn parse(&self, filename: &str) -> Result<DateTime<Utc>> {
        let regex = self.build_regex()?;
        let captures = regex.captures(filename).ok_or_else(|| color_eyre::eyre::eyre!("Filename '{filename}' doesn't match file pattern."))
            .context(format!("Applying regex {regex:?} on filename"))?;
        let year = if let Some(year) = captures.name("year") {
            let year = i32::from_str(year.as_str())?;

            Some(year)
        } else {
            None
        };
        let month = if let Some(month) = captures.name("month") {
            let month = u32::from_str(month.as_str())?;

            Some(month)
        } else if let Some(month_abr) = captures.name("month_abbr") {
            match month_abr.as_str().to_lowercase().as_str() {
                "jan" => Some(1),
                "feb" => Some(2),
                "mar" => Some(3),
                "apr" => Some(4),
                "may" => Some(5),
                "jun" => Some(6),
                "jul" => Some(7),
                "aug" => Some(8),
                "sep" => Some(9),
                "oct" => Some(10),
                "nov" => Some(11),
                "dec" => Some(12),
                _ => None,
            }
        } else {
            None
        };
        let day = if let Some(day) = captures.name("day") {
            let day = u32::from_str(day.as_str())?;

            Some(day)
        } else {
            None
        };
        let hour = if let Some(hour) = captures.name("hour") {
            let hour = u32::from_str(hour.as_str())?;

            Some(hour)
        } else {
            None
        };
        let minutes = if let Some(minutes) = captures.name("minutes") {
            let minutes = u32::from_str(minutes.as_str())?;

            Some(minutes)
        } else {
            None
        };
        let seconds = if let Some(seconds) = captures.name("seconds") {
            let seconds = u32::from_str(seconds.as_str())?;

            Some(seconds)
        } else {
            None
        };
        // let offset = if let Some(offset) = captures.name("timezone") {
        //     let timezone =
        // }
        let date = NaiveDate::from_ymd_opt(year.unwrap_or(2022), month.unwrap_or(1), day.unwrap_or(1)).unwrap();
        let time = NaiveTime::from_hms_opt(hour.unwrap_or_default(), minutes.unwrap_or_default(), seconds.unwrap_or_default()).unwrap();
        let datetime = DateTime::from_naive_utc_and_offset(NaiveDateTime::new(date, time), *Utc::now().offset());

        Ok(datetime)
    }

    fn build_regex(&self) -> Result<Regex> {
        let regex_str = self.0
            .replace("{name}", "(?P<name>.+)")
            .replace("{year}", "(?P<year>\\d{4})")
            .replace("{month_abr}", "(?P<month_abbr>[a-zA-Z]{3})")
            .replace("{month_abbr}", "(?P<month_abbr>[a-zA-Z]{3})")
            .replace("{month}", "(?P<month>\\d{1,2})")
            .replace("{day}", "(?P<day>\\d{1,2})")
            .replace("{hour}", "(?P<hour>\\d{1,2})")
            .replace("{minutes}", "(?P<minutes>\\d{1,2})")
            .replace("{seconds}", "(?P<seconds>\\d{1,2})")
            .replace("{TZ}", "(?P<timezone>[+-]\\d{2}:\\d{2})");

        let regex = Regex::new(&regex_str)?;

        Ok(regex)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};
    use test_case::test_case;

    use crate::config::RetentionFilePattern;

    #[test_case("2022", 2022)]
    #[test_case("2020", 2020)]
    fn parse_year(filename: &str, year: i32) {
        let file_pattern = RetentionFilePattern("{year}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(year, date_time.year())
    }

    #[test_case("1", 1)]
    #[test_case("3", 3)]
    #[test_case("12", 12)]
    fn parse_month(filename: &str, month: u32) {
        let file_pattern = RetentionFilePattern("{month}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(month, date_time.month())
    }

    #[test_case("Jan", 1)]
    #[test_case("Mar", 3)]
    #[test_case("JUN", 6)]
    #[test_case("dec", 12)]
    fn parse_month_abbrevation(filename: &str, month: u32) {
        let file_pattern = RetentionFilePattern("{month_abbr}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(month, date_time.month())
    }

    // First release had a typo in the file pattern. This test ensures that the month abbreviation is still parsed correctly.
    #[test_case("Jan", 1)]
    #[test_case("Mar", 3)]
    #[test_case("JUN", 6)]
    #[test_case("dec", 12)]
    fn parse_month_abbrevation_with_typo(filename: &str, month: u32) {
        let file_pattern = RetentionFilePattern("{month_abr}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(month, date_time.month())
    }

    #[test_case("1", 1)]
    #[test_case("2", 2)]
    #[test_case("31", 31)]
    fn parse_day(filename: &str, day: u32) {
        let file_pattern = RetentionFilePattern("{day}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(day, date_time.day())
    }

    #[test_case("{year}-{month}-{day}", "2022-12-19", 2022, 12, 19)]
    #[test_case("{year}-{month}-{day}", "2022-01-19", 2022, 1, 19)]
    #[test_case("{year}.{month}.{day}", "2021.1.4", 2021, 1, 4)]
    fn basic_date(pattern: &str, filename: &str, year: i32, month: u32, day: u32) {
        let file_pattern = RetentionFilePattern(pattern.to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(year, date_time.year());
        assert_eq!(month, date_time.month());
        assert_eq!(day, date_time.day());
    }

    #[test_case("1", 1)]
    #[test_case("2", 2)]
    #[test_case("12", 12)]
    fn parse_hour(filename: &str, hour: u32) {
        let file_pattern = RetentionFilePattern("{hour}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(hour, date_time.hour())
    }

    #[test_case("1", 1)]
    #[test_case("0", 0)]
    #[test_case("59", 59)]
    fn parse_minutes(filename: &str, minutes: u32) {
        let file_pattern = RetentionFilePattern("{minutes}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(minutes, date_time.minute())
    }

    #[test_case("1", 1)]
    #[test_case("0", 0)]
    #[test_case("59", 59)]
    fn parse_seconds(filename: &str, seconds: u32) {
        let file_pattern = RetentionFilePattern("{seconds}".to_string());

        let date_time = file_pattern.parse(filename).unwrap();

        assert_eq!(seconds, date_time.second())
    }
}
