use chrono::TimeZone;

use crate::time_converter::model::TimeComponents;

pub struct TimeExtractorContext<Tz: TimeZone> {
    local_tz: Tz,
    msg_time: TimeComponents,
}

#[allow(dead_code)]
impl<Tz: TimeZone> TimeExtractorContext<Tz> {
    pub fn new(local_tz: Tz, msg_time: TimeComponents) -> Self {
        TimeExtractorContext { local_tz, msg_time }
    }

    pub fn local_tz(&self) -> &Tz {
        &self.local_tz
    }

    pub fn message_time(&self) -> TimeComponents {
        self.msg_time
    }
}
