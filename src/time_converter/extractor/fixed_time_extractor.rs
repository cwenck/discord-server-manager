use chrono::TimeZone;
use regex::Regex;

use crate::{extractor::Extractor, time_converter::model::TimeComponents};

use super::time_extractor_context::TimeExtractorContext;

pub struct FixedTimeExtractor {
    regex: Regex,
    fixed_time: TimeComponents,
}

#[allow(dead_code)]
impl FixedTimeExtractor {
    pub fn new(regex: &str, value: TimeComponents) -> Self {
        let compiled_regex = Regex::new(regex).expect("Failed to compile regex.");
        Self {
            regex: compiled_regex,
            fixed_time: value,
        }
    }
}

impl<Tz: TimeZone> Extractor<TimeExtractorContext<Tz>, TimeComponents> for FixedTimeExtractor {
    fn extract(&self, text: &str, _: &TimeExtractorContext<Tz>) -> Vec<TimeComponents> {
        let mut result = Vec::new();

        if self.regex.is_match(text) {
            result.push(self.fixed_time);
        };

        result
    }
}

#[cfg(test)]
mod test {
    use crate::time_converter::model::TimeKind;

    use super::*;

    macro_rules! test_extract_data {
        ($($name:ident{extractor: $input_extractor:expr, text: $input_text:expr, expected: $expected:expr,},)*) => {
            $(
            #[test]
            fn $name(){
                let msg_time = TimeComponents::new(5, 7, TimeKind::AM).expect("Expected the time compoents to be valid.");
                let ctx = TimeExtractorContext::new(chrono::Utc, msg_time);
                let actual: Vec<TimeComponents> = $input_extractor.extract($input_text, &ctx);
                assert_eq!(actual, $expected);
            }
            )*
        };
    }

    test_extract_data! {
        test_extract_no_match_1 {
            extractor: FixedTimeExtractor::new(r"midday", TimeComponents::of(12, 0, TimeKind::PM)),
            text: "It is midnight.",
            expected: vec![],
        },
        test_extract_single_match_1 {
            extractor: FixedTimeExtractor::new(r"noon|midday", TimeComponents::of(12, 0, TimeKind::PM)),
            text: "I'm free at noon.",
            expected: vec![TimeComponents::of(12, 0, TimeKind::PM)],
        },
        test_extract_single_match_2 {
            extractor: FixedTimeExtractor::new(r"noon|midday", TimeComponents::of(12, 0, TimeKind::PM)),
            text: "I'm free at midday.",
            expected: vec![TimeComponents::of(12, 0, TimeKind::PM)],
        },
        test_extract_single_match_3 {
            extractor: FixedTimeExtractor::new(r"midnight", TimeComponents::of(12, 0, TimeKind::AM)),
            text: "It is midnight.",
            expected: vec![TimeComponents::of(12, 0, TimeKind::AM)],
        },
        test_extract_multi_match_1 {
            extractor: FixedTimeExtractor::new(r"noon", TimeComponents::of(12, 0, TimeKind::Military)),
            text: "At noon, it is noon.",
            expected: vec![TimeComponents::of(12, 0, TimeKind::Military)],
        },
    }
}
