use std::collections::HashSet;
use std::hash::Hash;

use chrono::Datelike;

use crate::config::RetentionPolicy;
use crate::file::RetentionFile;

impl RetentionPolicy {
    pub fn retain(&self, files: Vec<RetentionFile>) -> (Vec<RetentionFile>, Vec<RetentionFile>) {
        if self == &RetentionPolicy::default() {
            return (files, Default::default());
        }

        let mut files = files.into_iter();

        let mut keep = Vec::new();
        let mut drop = Vec::new();


        if let Some(last) = self.keep_last {
            for _ in 0..last {
                if let Some(file) = files.next() {
                    keep.push(file)
                }
            }
        }
        if let Some(daily) = self.keep_daily {
            retain_items(&mut files, &mut keep, &mut drop, daily, |file| file.date.date_naive());
        }
        if let Some(weekly) = self.keep_weekly {
            retain_items(&mut files, &mut keep, &mut drop, weekly, |file| file.date.iso_week());
        }
        if let Some(monthly) = self.keep_monthly {
            retain_items(&mut files, &mut keep, &mut drop, monthly, |file| file.date.month());
        }
        if let Some(yearly) = self.keep_yearly {
            retain_items(&mut files, &mut keep, &mut drop, yearly, |file| file.date.year());
        }

        for file in files {
            drop.push(file);
        }

        (keep, drop)
    }
}

fn retain_items<I: Iterator<Item=RetentionFile>, F: Eq + Hash>(files: &mut I, keep: &mut Vec<RetentionFile>, drop: &mut Vec<RetentionFile>, count: usize, get_identifier: impl Fn(&RetentionFile) -> F) {
    let mut categories = HashSet::new();
    for day in keep.iter() {
        categories.insert(get_identifier(day));
    }
    let mut remaining = count;
    while remaining > 0 {
        if let Some(file) = files.next() {
            let day = get_identifier(&file);
            if categories.contains(&day) {
                drop.push(file);
            } else {
                categories.insert(day);
                keep.push(file);
                remaining = remaining.saturating_sub(1);
            }
        }else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::ops::{Sub};

    use chrono::{Datelike, DateTime, Duration, Months, TimeZone, Utc};
    use test_case::test_case;

    use crate::config::RetentionPolicy;
    use crate::file::RetentionFile;

    #[test_case(2)]
    #[test_case(5)]
    fn retain_should_drop_no_files_for_empty_policy(count: usize) {
        let policy = RetentionPolicy::default();
        let file = RetentionFile {
            date: Default::default(),
            filename: Default::default(),
        };
        let files = vec![file; count];

        let (keep, drop) = policy.retain(files);

        assert_eq!(count, keep.len());
        assert_eq!(0, drop.len());
    }

    #[test_case(2, 1)]
    #[test_case(5, 2)]
    fn retain_should_keep_last_files(total: usize, last: usize) {
        let policy = RetentionPolicy {
            keep_last: Some(last),
            ..Default::default()
        };
        let file = RetentionFile {
            date: Default::default(),
            filename: Default::default(),
        };
        let files = vec![file; total];

        let (keep, drop) = policy.retain(files);

        assert_eq!(last, keep.len());
        assert_eq!(total - last, drop.len());
    }

    #[test_case(2, vec ! [now(true), hours(2, false)])]
    #[test_case(2, vec ! [now(true), days(1, true), days(2, false)])]
    #[test_case(5, vec ! [now(true), hours(12, false), days(1, true), hours(36, false), days(2, true)])]
    fn retain_should_keep_daily_files(keep_daily: usize, files: Vec<Expected>) {
        let policy = RetentionPolicy {
            keep_daily: Some(keep_daily),
            ..Default::default()
        };
        let (files, keep_expected, drop_expected) = create_expected(files);

        let (keep, drop) = policy.retain(files);

        assert_eq!(keep_expected, keep);
        assert_eq!(drop_expected, drop);
    }

    #[test_case(2, vec ! [now(true), days(2, false)])]
    #[test_case(2, vec ! [now(true), weeks(1, true), weeks(2, false)])]
    #[test_case(5, vec ! [now(true), days(4, false), weeks(1, true), days(10, false), weeks(2, true)])]
    // Special case where we have the same week number in two years
    #[test_case(5, vec ! [days(- 2, true), years(2, true)])]
    fn retain_should_keep_weekly_files(keep_weekly: usize, files: Vec<Expected>) {
        let policy = RetentionPolicy {
            keep_weekly: Some(keep_weekly),
            ..Default::default()
        };
        let (files, keep_expected, drop_expected) = create_expected(files);

        let (keep, drop) = policy.retain(files);

        assert_eq!(keep_expected, keep);
        assert_eq!(drop_expected, drop);
    }

    #[test_case(2, vec![days(1, true), days(2, false)])]
    #[test_case(2, vec![now(true), days(1, true), months(2, false)])]
    #[test_case(3, vec![now(true), months(1, true), months(2, true), months(3, false)])]
    fn retain_should_keep_monthly_files(keep_monthly: usize, files: Vec<Expected>) {
        let policy = RetentionPolicy {
            keep_monthly: Some(keep_monthly),
            ..Default::default()
        };
        let (files, keep_expected, drop_expected) = create_expected(files);

        let (keep, drop) = policy.retain(files);

        assert_eq!(keep_expected, keep);
        assert_eq!(drop_expected, drop);
    }

    #[test_case(2, vec![now(true), years(1, true), years(2, false)])]
    #[test_case(2, vec![days(1, true), years(1, false), years(2, true)])]
    #[test_case(1, vec![now(true), years(1, false), years(2, false)])]
    fn retain_should_keep_yearly_files(keep_yearly: usize, files: Vec<Expected>) {
        let policy = RetentionPolicy {
            keep_yearly: Some(keep_yearly),
            ..Default::default()
        };
        let (files, keep_expected, drop_expected) = create_expected(files);

        let (keep, drop) = policy.retain(files);

        assert_eq!(keep_expected, keep);
        assert_eq!(drop_expected, drop);
    }

    #[test_case(vec!["2022-01-01", "2021-12-31", "2021-12-30", "2021-11-30", "2021-10-31", "2021-09-30"])]
    fn retain_should_retain_combination(files: Vec<&'static str>) {
        let policy = RetentionPolicy {
            keep_last: Some(1),
            keep_daily: Some(2),
            keep_monthly: Some(3),
            ..Default::default()
        };
        let (files, keep_expected, drop_expected) = create_daily_expected(files);

        let (keep, drop) = policy.retain(files);

        assert_eq!(keep_expected, keep);
        assert_eq!(drop_expected, drop);
    }

    fn day() -> DateTime<Utc> {
        Utc.from_utc_datetime(&DateTime::parse_from_rfc3339("2022-01-01T22:00:00Z").unwrap().naive_utc())
    }

    fn now(matches: bool) -> Expected {
        (day(), matches)
    }

    fn hours(hours: i64, matches: bool) -> Expected {
        (day().sub(Duration::hours(hours)), matches)
    }

    fn days(days: i64, matches: bool) -> Expected {
        (day().sub(Duration::days(days)), matches)
    }

    fn weeks(weeks: i64, matches: bool) -> Expected {
        (day().sub(Duration::weeks(weeks)), matches)
    }

    fn months(months: i32, matches: bool) -> Expected {
        let date = if months > 0 {
            day().checked_sub_months(Months::new(months.unsigned_abs())).unwrap()
        } else {
            day().checked_add_months(Months::new(months.unsigned_abs())).unwrap()
        };
        (date, matches)
    }

    fn years(years: i32, matches: bool) -> Expected {
        let date = day().with_year(2022 - years).unwrap();
        (date, matches)
    }

    type Expected = (DateTime<Utc>, bool);

    fn create_expected(files: Vec<Expected>) -> (Vec<RetentionFile>, Vec<RetentionFile>, Vec<RetentionFile>) {
        let files = files.into_iter()
            .map(|(date, should_keep)| {
                let file = RetentionFile {
                    date,
                    filename: Default::default(),
                };
                (file, should_keep)
            })
            .collect::<Vec<_>>();
        let keep_expected = files.iter().filter(|(_, should_keep)| *should_keep).map(|(file, _)| file.clone()).collect();
        let drop_expected = files.iter().filter(|(_, should_keep)| !should_keep).map(|(file, _)| file.clone()).collect();
        let files = files.into_iter().map(|(file, _)| file).collect();

        (files, keep_expected, drop_expected)
    }

    fn create_daily_expected(dates: Vec<&str>) -> (Vec<RetentionFile>, Vec<RetentionFile>, Vec<RetentionFile>) {
        let dates: Vec<_> = dates.into_iter().map(|date| Utc.from_utc_datetime(&DateTime::parse_from_rfc3339(&format!("{date}T22:00:00Z")).unwrap().naive_utc())).collect();
        let start_date = dates.first().cloned().unwrap();
        let last_date = dates.last().cloned().unwrap();
        let expected: HashSet<_> = dates.into_iter().collect();
        let mut dates = Vec::new();
        dates.push(start_date);
        let mut days = 1;
        while dates.last().unwrap() != &last_date {
            let day = start_date.sub(Duration::days(days));
            dates.push(day);

            days += 1;
        }
        let files = dates.into_iter()
            .map(|date| {
                let keep = expected.contains(&date);

                (date, keep)
            })
            .collect();

        create_expected(files)
    }
}
