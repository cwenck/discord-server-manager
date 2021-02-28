use chrono::TimeZone;
use regex::Regex;

use crate::{extractor::Extractor, time_converter::model::TimeComponents};

use super::TimeExtractorContext;

#[derive(Debug)]
pub struct CurrentTimeExtractor {
    regex: Regex,
}

#[allow(dead_code)]
impl CurrentTimeExtractor {
    pub fn new(regex: &str) -> Self {
        let compiled_regex = Regex::new(regex).expect("Failed to compile regex.");

        Self {
            regex: compiled_regex,
        }
    }
}

impl<Tz: TimeZone> Extractor<TimeExtractorContext<Tz>, TimeComponents> for CurrentTimeExtractor {
    fn extract(&self, text: &str, ctx: &TimeExtractorContext<Tz>) -> Vec<TimeComponents> {
        let mut result = Vec::new();

        if self.regex.is_match(text) {
            result.push(ctx.message_time());
        }

        result
    }
}

#[cfg(test)]
mod test {
    use crate::time_converter::model::TimeKind;

    use super::*;

    macro_rules! test_extract_data {
        ($($name:ident{extractor: $input_extractor:expr, text: $input_text:expr, msg_time: $input_msg_time:expr, expected: $expected:expr,},)*) => {
            $(
            #[test]
            fn $name(){
                let ctx = TimeExtractorContext::new(chrono::Utc, $input_msg_time);
                let actual: Vec<TimeComponents> = $input_extractor.extract($input_text, &ctx);
                assert_eq!(actual, $expected);
            }
            )*
        };
    }

    test_extract_data! {
        test_extract_no_match {
            extractor: CurrentTimeExtractor::new(r"now"),
            text: "Never!",
            msg_time: TimeComponents::of(10, 15, TimeKind::PM),
            expected: vec![],
        },
        test_extract_single_match_1 {
            extractor: CurrentTimeExtractor::new(r"now"),
            text: "The time is now.",
            msg_time: TimeComponents::of(1, 5, TimeKind::AM),
            expected: vec![TimeComponents::of(1, 5, TimeKind::AM)],
        },
        test_extract_single_match_2{
            extractor: CurrentTimeExtractor::new(r"^(?i:what\W*time\W*is\W*it\W*right\W*now[?]*)"),
            text: "whattimeisitrightnow",
            msg_time: TimeComponents::of(3, 55, TimeKind::PM),
            expected: vec![TimeComponents::of(3, 55, TimeKind::PM)],
        },
        test_extract_single_match_3{
            extractor: CurrentTimeExtractor::new(r"^(?i:what\W*time\W*is\W*it\W*right\W*now[?]*)"),
            text: "what time is it right now?",
            msg_time: TimeComponents::of(14, 25, TimeKind::Military),
            expected: vec![TimeComponents::of(14, 25, TimeKind::Military)],
        },
        test_extract_single_match_4{
            extractor: CurrentTimeExtractor::new(r"^(?i:what\W*time\W*is\W*it\W*right\W*now[?]*)"),
            text: "WhAt  TiMeIs     It RiGhTnOw???",
            msg_time: TimeComponents::of(1, 5, TimeKind::AM),
            expected: vec![TimeComponents::of(1, 5, TimeKind::AM)],
        },
        test_extract_no_match_2{
            extractor: CurrentTimeExtractor::new(r"^(?i:what\W*time\W*is\W*it\W*right\W*now[?]*)"),
            text: "Shouldn't match what time is it right now?",
            msg_time: TimeComponents::of(14, 25, TimeKind::Military),
            expected: vec![],
        },
        test_extract_multi_match_1{
            extractor: CurrentTimeExtractor::new(r"[nN]ow"),
            text: "Can we do it now? How about now? Now?",
            msg_time: TimeComponents::of(15, 11, TimeKind::Military),
            expected: vec![TimeComponents::of(15, 11, TimeKind::Military)],
        },
    }
}
