use std::str::FromStr;

use crate::time_converter::{
    extractor::{CurrentTimeExtractor, TimeExtractorContext},
    model::{TimeComponents, TimeKind},
};
use regex::{Captures, Regex};
pub trait Extractor<C, R> {
    fn extract(&self, text: &str, ctx: &C) -> Vec<R>;
}

struct DynamicTimeExtractor {
    regex: Regex,
}

const HOURS_CAPTURE_NAME: &str = "hours";
const MINUTES_CAPTURE_NAME: &str = "minutes";
const TIME_KIND_CAPTURE_NAME: &str = "time_kind";

impl DynamicTimeExtractor {
    pub fn new(regex: &str) -> Self {
        let compiled_regex = Regex::new(regex).expect("Failed to compile regex.");
        Self {
            regex: compiled_regex,
        }
    }

    fn extract_capture<T: FromStr>(captures: &Captures, name: &str) -> Option<T> {
        captures
            .name(name)
            .map(|capture| capture.as_str())
            .map(|string| string.parse::<T>())
            .and_then(|result| result.ok())
    }
}

// impl Extractor<TimeExtractorContext, TimeComponents> for DynamicTimeExtractor {
//     fn extract(&self, text: &str, _: &TimeExtractorContext) -> Option<TimeComponents> {
//         self.regex.captures(text).and_then(|captures| {
//             let hour = Self::extract_capture::<u32>(&captures, HOURS_CAPTURE_NAME)?;
//             let minute = Self::extract_capture::<u32>(&captures, MINUTES_CAPTURE_NAME)?;
//             let time_kind = Self::extract_capture::<String>(&captures, TIME_KIND_CAPTURE_NAME)
//                 .map(|it| it.to_uppercase())
//                 .and_then(|it| match it.as_str() {
//                     "AM" => Some(TimeKind::AM),
//                     "PM" => Some(TimeKind::PM),
//                     _ => None,
//                 })?;

//             TimeComponents::new(hour, minute, time_kind).ok()
//         })
//     }
// }

impl CurrentTimeExtractor {}
