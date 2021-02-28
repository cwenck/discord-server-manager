use core::panic;
use std::{collections::HashSet, str::FromStr};

use chrono::TimeZone;
use fancy_regex::{Captures, Regex};
use itertools::Itertools;
use once_cell::sync::Lazy;

use crate::{
    extractor::Extractor,
    time_converter::model::{TimeComponents, TimeKind},
};

use super::TimeExtractorContext;
#[derive(Debug)]
pub struct DynamicTimeExtractor {
    regex: Regex,
}

const HOURS_CAPTURE_NAME: &str = "hours";
const MINUTES_CAPTURE_NAME: &str = "minutes";
const TIME_KIND_CAPTURE_NAME: &str = "time_kind";

// TODO : Consider moving to a separate set utils file
macro_rules! set {
    ( $( $item:expr ),* ) => {
        {
            let mut set = HashSet::new();
            $(
                set.insert($item);
            )*
            set
        }
    };
}

static ALLOWED_CAPTURE_GROUPS: Lazy<HashSet<&str>> = Lazy::new(|| {
    set![
        HOURS_CAPTURE_NAME,
        MINUTES_CAPTURE_NAME,
        TIME_KIND_CAPTURE_NAME
    ]
});

static REQUIRED_CAPTURE_GROUPS: Lazy<HashSet<&str>> = Lazy::new(|| {
    set![
        HOURS_CAPTURE_NAME,
        MINUTES_CAPTURE_NAME,
        TIME_KIND_CAPTURE_NAME
    ]
});

#[allow(dead_code)]
impl DynamicTimeExtractor {
    pub fn new(regex: &str) -> Self {
        let compiled_regex = Regex::new(regex).expect("Failed to compile regex.");
        Self::validate_regex(&compiled_regex);

        Self {
            regex: compiled_regex,
        }
    }

    fn validate_regex(regex: &Regex) {
        let names: HashSet<&str> = regex
            .capture_names()
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();

        if !Self::contains_all(&names, &REQUIRED_CAPTURE_GROUPS) {
            panic!(
                "Regex must contain all of the following named capture groups {:?} but contains {:?}",
                *REQUIRED_CAPTURE_GROUPS, names
            );
        }

        if !Self::contains_all(&ALLOWED_CAPTURE_GROUPS, &names) {
            panic!(
                "Regex must only contain named captue groups from {:?} but contains {:?}",
                *ALLOWED_CAPTURE_GROUPS, names
            );
        }
    }

    // TODO : Consider moving to a separate set utils file
    fn contains_all<T>(set_a: &HashSet<T>, set_b: &HashSet<T>) -> bool
    where
        T: Eq,
        T: core::hash::Hash,
    {
        set_a.intersection(set_b).count() == set_b.len()
    }
}

impl<Tz: TimeZone> Extractor<TimeExtractorContext<Tz>, TimeComponents> for DynamicTimeExtractor {
    fn extract(&self, text: &str, _ctx: &TimeExtractorContext<Tz>) -> Vec<TimeComponents> {
        self.regex
            .captures_iter(text)
            .filter_map(|result| result.ok())
            .filter_map(|captures| process_captures(&captures))
            .unique()
            .collect()
    }
}

fn extract_capture<T: FromStr>(captures: &Captures, name: &str) -> Option<T> {
    captures
        .name(name)
        .map(|capture| capture.as_str())
        .map(|string| string.parse::<T>())
        .and_then(|result| result.ok())
}

fn process_captures(captures: &Captures) -> Option<TimeComponents> {
    let hour = extract_capture::<u32>(&captures, HOURS_CAPTURE_NAME)?;
    let minute = extract_capture::<u32>(&captures, MINUTES_CAPTURE_NAME).unwrap_or(0);
    let time_kind = extract_capture::<String>(&captures, TIME_KIND_CAPTURE_NAME)
        .map(|it| it.to_uppercase())
        .and_then(|it| match it.as_str() {
            "AM" => Some(TimeKind::AM),
            "PM" => Some(TimeKind::PM),
            _ => None,
        })?;

    TimeComponents::new(hour, minute, time_kind).ok()
}

#[cfg(test)]
mod test {
    use crate::time_converter::model::TimeKind;
    use chrono::Utc;

    use super::*;

    // r"(?i:(?<!\w)(?P<hours>(?:1[012])|(?:0?[123456789]))\s*(?P<time_kind>[ap]m)(?!\w))"

    #[test]
    fn test_missing_capture_groups_1() {
        DynamicTimeExtractor::new(r"");
    }

    #[test]
    fn test() {
        let extractor = DynamicTimeExtractor::new(
            r"(?i:(?<!\w)(?P<hours>(?:1[012])|(?:0?[123456789]))\s*(?::\s*(?P<minutes>(?:[12345]\d)|(?:0\d)))?\s*(?P<time_kind>[ap]m)(?!\w))",
        );

        let text = String::from("3am 4america 5am-6pm 5am 17am 17pm");
        let ctx = TimeExtractorContext::new(Utc, TimeComponents::of(1, 0, TimeKind::AM));
        let actual: Vec<TimeComponents> = extractor.extract(&text, &ctx);

        println!("{:?}", actual)
    }
}
