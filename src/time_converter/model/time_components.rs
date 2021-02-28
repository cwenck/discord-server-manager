use chrono::{NaiveTime, Timelike};

use std::fmt;
use thiserror::Error;

type TimeResult<T> = Result<T, TimeRepresentationError>;
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TimeRepresentationError {
    #[error("Time component value out of bounds. Actual value was {hour}:{minute} {kind}.")]
    OutOfBounds {
        hour: u32,
        minute: u32,
        kind: TimeKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeKind {
    AM,
    PM,
    Military,
}

impl fmt::Display for TimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TimeKind::*;

        let string_val = match *self {
            AM => "AM",
            PM => "PM",
            Military => "Military",
        };

        f.write_str(string_val)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeComponents {
    hour: u32,
    minute: u32,
    kind: TimeKind,
}

impl TimeComponents {
    pub fn new(hour: u32, minute: u32, kind: TimeKind) -> TimeResult<Self> {
        TimeComponents { hour, minute, kind }.validate()
    }

    #[cfg(test)]
    pub(crate) fn of(hour: u32, minute: u32, kind: TimeKind) -> Self {
        TimeComponents { hour, minute, kind }
    }

    fn validate(self) -> TimeResult<Self> {
        use TimeRepresentationError::OutOfBounds;
        let TimeComponents { hour, minute, kind } = self;

        match (hour, minute, kind) {
            (1..=12, 0..=59, TimeKind::AM)
            | (1..=12, 0..=59, TimeKind::PM)
            | (0..=23, 0..=59, TimeKind::Military) => Ok(self),
            _ => Err(OutOfBounds { hour, minute, kind }),
        }
    }

    pub fn to_12h(self) -> Self {
        let TimeComponents { hour, minute, kind } =
            self.validate().expect("Invalid input time comonents.");

        let (updated_hour, updated_kind) = match (hour, kind) {
            (0, TimeKind::Military) => (12, TimeKind::AM),
            (12, TimeKind::Military) => (12, TimeKind::PM),
            (h, TimeKind::Military) if h > 12 => (hour - 12, TimeKind::PM),
            (_, TimeKind::Military) => (hour, TimeKind::AM),
            (_, TimeKind::AM) | (_, TimeKind::PM) => (hour, kind),
        };

        TimeComponents::new(updated_hour, minute, updated_kind)
            .expect("Invalid output time components.")
    }

    pub fn to_24h(self) -> Self {
        let TimeComponents { hour, minute, kind } =
            self.validate().expect("Invalid input time components.");

        let updated_hour = match (hour, kind) {
            (12, TimeKind::AM) => 0,
            (12, TimeKind::PM) => 12,
            (_, TimeKind::Military) | (_, TimeKind::AM) => hour,
            (_, TimeKind::PM) => hour + 12,
        };

        TimeComponents::new(updated_hour, minute, TimeKind::Military)
            .expect("Invalid output time components.")
    }
}

impl From<TimeComponents> for NaiveTime {
    fn from(value: TimeComponents) -> Self {
        let TimeComponents { hour, minute, .. } = value.to_24h();
        NaiveTime::from_hms_opt(hour, minute, 0)
            .expect("Expected time components to map to a valid time.")
    }
}

impl From<NaiveTime> for TimeComponents {
    fn from(value: NaiveTime) -> Self {
        TimeComponents::new(value.hour(), value.minute(), TimeKind::Military)
            .expect("Invalid output time components.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! time_to_12h_data {
        ($($name:ident($input:expr, $expected:expr),)*) => {
            $(
            #[test]
            fn $name(){
                let actual = $input.to_12h();
                assert_eq!(actual, $expected);
            }
            )*
        };
    }

    time_to_12h_data! {
        time_1am_to_12h(TimeComponents::of(1, 0, TimeKind::AM), TimeComponents::of(1, 0, TimeKind::AM)),
        time_2am_to_12h(TimeComponents::of(2, 0, TimeKind::AM), TimeComponents::of(2, 0, TimeKind::AM)),
        time_3am_to_12h(TimeComponents::of(3, 0, TimeKind::AM), TimeComponents::of(3, 0, TimeKind::AM)),
        time_4am_to_12h(TimeComponents::of(4, 0, TimeKind::AM), TimeComponents::of(4, 0, TimeKind::AM)),
        time_5am_to_12h(TimeComponents::of(5, 0, TimeKind::AM), TimeComponents::of(5, 0, TimeKind::AM)),
        time_6am_to_12h(TimeComponents::of(6, 0, TimeKind::AM), TimeComponents::of(6, 0, TimeKind::AM)),
        time_7am_to_12h(TimeComponents::of(7, 0, TimeKind::AM), TimeComponents::of(7, 0, TimeKind::AM)),
        time_8am_to_12h(TimeComponents::of(8, 0, TimeKind::AM), TimeComponents::of(8, 0, TimeKind::AM)),
        time_9am_to_12h(TimeComponents::of(9, 0, TimeKind::AM), TimeComponents::of(9, 0, TimeKind::AM)),
        time_10am_to_12h(TimeComponents::of(10, 0, TimeKind::AM), TimeComponents::of(10, 0, TimeKind::AM)),
        time_11am_to_12h(TimeComponents::of(11, 0, TimeKind::AM), TimeComponents::of(11, 0, TimeKind::AM)),
        time_12am_to_12h(TimeComponents::of(12, 0, TimeKind::AM), TimeComponents::of(12, 0, TimeKind::AM)),
        time_1pm_to_12h(TimeComponents::of(1, 0, TimeKind::PM), TimeComponents::of(1, 0, TimeKind::PM)),
        time_2pm_to_12h(TimeComponents::of(2, 0, TimeKind::PM), TimeComponents::of(2, 0, TimeKind::PM)),
        time_3pm_to_12h(TimeComponents::of(3, 0, TimeKind::PM), TimeComponents::of(3, 0, TimeKind::PM)),
        time_4pm_to_12h(TimeComponents::of(4, 0, TimeKind::PM), TimeComponents::of(4, 0, TimeKind::PM)),
        time_5pm_to_12h(TimeComponents::of(5, 0, TimeKind::PM), TimeComponents::of(5, 0, TimeKind::PM)),
        time_6pm_to_12h(TimeComponents::of(6, 0, TimeKind::PM), TimeComponents::of(6, 0, TimeKind::PM)),
        time_7pm_to_12h(TimeComponents::of(7, 0, TimeKind::PM), TimeComponents::of(7, 0, TimeKind::PM)),
        time_8pm_to_12h(TimeComponents::of(8, 0, TimeKind::PM), TimeComponents::of(8, 0, TimeKind::PM)),
        time_9pm_to_12h(TimeComponents::of(9, 0, TimeKind::PM), TimeComponents::of(9, 0, TimeKind::PM)),
        time_10pm_to_12h(TimeComponents::of(10, 0, TimeKind::PM), TimeComponents::of(10, 0, TimeKind::PM)),
        time_11pm_to_12h(TimeComponents::of(11, 0, TimeKind::PM), TimeComponents::of(11, 0, TimeKind::PM)),
        time_12pm_to_12h(TimeComponents::of(12, 0, TimeKind::PM), TimeComponents::of(12, 0, TimeKind::PM)),
        time_0_military_to_12h(TimeComponents::of(0, 0, TimeKind::Military), TimeComponents::of(12, 0, TimeKind::AM)),
        time_1_military_to_12h(TimeComponents::of(1, 0, TimeKind::Military), TimeComponents::of(1, 0, TimeKind::AM)),
        time_2_military_to_12h(TimeComponents::of(2, 0, TimeKind::Military), TimeComponents::of(2, 0, TimeKind::AM)),
        time_3_military_to_12h(TimeComponents::of(3, 0, TimeKind::Military), TimeComponents::of(3, 0, TimeKind::AM)),
        time_4_military_to_12h(TimeComponents::of(4, 0, TimeKind::Military), TimeComponents::of(4, 0, TimeKind::AM)),
        time_5_military_to_12h(TimeComponents::of(5, 0, TimeKind::Military), TimeComponents::of(5, 0, TimeKind::AM)),
        time_6_military_to_12h(TimeComponents::of(6, 0, TimeKind::Military), TimeComponents::of(6, 0, TimeKind::AM)),
        time_7_military_to_12h(TimeComponents::of(7, 0, TimeKind::Military), TimeComponents::of(7, 0, TimeKind::AM)),
        time_8_military_to_12h(TimeComponents::of(8, 0, TimeKind::Military), TimeComponents::of(8, 0, TimeKind::AM)),
        time_9_military_to_12h(TimeComponents::of(9, 0, TimeKind::Military), TimeComponents::of(9, 0, TimeKind::AM)),
        time_10_military_to_12h(TimeComponents::of(10, 0, TimeKind::Military), TimeComponents::of(10, 0, TimeKind::AM)),
        time_11_military_to_12h(TimeComponents::of(11, 0, TimeKind::Military), TimeComponents::of(11, 0, TimeKind::AM)),
        time_12_military_to_12h(TimeComponents::of(12, 0, TimeKind::Military), TimeComponents::of(12, 0, TimeKind::PM)),
        time_13_military_to_12h(TimeComponents::of(13, 0, TimeKind::Military), TimeComponents::of(1, 0, TimeKind::PM)),
        time_14_military_to_12h(TimeComponents::of(14, 0, TimeKind::Military), TimeComponents::of(2, 0, TimeKind::PM)),
        time_15_military_to_12h(TimeComponents::of(15, 0, TimeKind::Military), TimeComponents::of(3, 0, TimeKind::PM)),
        time_16_military_to_12h(TimeComponents::of(16, 0, TimeKind::Military), TimeComponents::of(4, 0, TimeKind::PM)),
        time_17_military_to_12h(TimeComponents::of(17, 0, TimeKind::Military), TimeComponents::of(5, 0, TimeKind::PM)),
        time_18_military_to_12h(TimeComponents::of(18, 0, TimeKind::Military), TimeComponents::of(6, 0, TimeKind::PM)),
        time_19_military_to_12h(TimeComponents::of(19, 0, TimeKind::Military), TimeComponents::of(7, 0, TimeKind::PM)),
        time_20_military_to_12h(TimeComponents::of(20, 0, TimeKind::Military), TimeComponents::of(8, 0, TimeKind::PM)),
        time_21_military_to_12h(TimeComponents::of(21, 0, TimeKind::Military), TimeComponents::of(9, 0, TimeKind::PM)),
        time_22_military_to_12h(TimeComponents::of(22, 0, TimeKind::Military), TimeComponents::of(10, 0, TimeKind::PM)),
        time_23_military_to_12h(TimeComponents::of(23, 0, TimeKind::Military), TimeComponents::of(11, 0, TimeKind::PM)),
    }

    macro_rules! time_to_24h_data {
        ($($name:ident($input:expr, $expected:expr),)*) => {
            $(
            #[test]
            fn $name(){
                let actual = $input.to_24h();
                assert_eq!(actual, $expected);
            }
            )*
        };
    }

    time_to_24h_data! {
        time_1am_to_24h(TimeComponents::of(1, 0, TimeKind::AM), TimeComponents::of(1, 0, TimeKind::Military)),
        time_2am_to_24h(TimeComponents::of(2, 0, TimeKind::AM), TimeComponents::of(2, 0, TimeKind::Military)),
        time_3am_to_24h(TimeComponents::of(3, 0, TimeKind::AM), TimeComponents::of(3, 0, TimeKind::Military)),
        time_4am_to_24h(TimeComponents::of(4, 0, TimeKind::AM), TimeComponents::of(4, 0, TimeKind::Military)),
        time_5am_to_24h(TimeComponents::of(5, 0, TimeKind::AM), TimeComponents::of(5, 0, TimeKind::Military)),
        time_6am_to_24h(TimeComponents::of(6, 0, TimeKind::AM), TimeComponents::of(6, 0, TimeKind::Military)),
        time_7am_to_24h(TimeComponents::of(7, 0, TimeKind::AM), TimeComponents::of(7, 0, TimeKind::Military)),
        time_8am_to_24h(TimeComponents::of(8, 0, TimeKind::AM), TimeComponents::of(8, 0, TimeKind::Military)),
        time_9am_to_24h(TimeComponents::of(9, 0, TimeKind::AM), TimeComponents::of(9, 0, TimeKind::Military)),
        time_10am_to_24h(TimeComponents::of(10, 0, TimeKind::AM), TimeComponents::of(10, 0, TimeKind::Military)),
        time_11am_to_24h(TimeComponents::of(11, 0, TimeKind::AM), TimeComponents::of(11, 0, TimeKind::Military)),
        time_12am_to_24h(TimeComponents::of(12, 0, TimeKind::AM), TimeComponents::of(0, 0, TimeKind::Military)),
        time_1pm_to_24h(TimeComponents::of(1, 0, TimeKind::PM), TimeComponents::of(13, 0, TimeKind::Military)),
        time_2pm_to_24h(TimeComponents::of(2, 0, TimeKind::PM), TimeComponents::of(14, 0, TimeKind::Military)),
        time_3pm_to_24h(TimeComponents::of(3, 0, TimeKind::PM), TimeComponents::of(15, 0, TimeKind::Military)),
        time_4pm_to_24h(TimeComponents::of(4, 0, TimeKind::PM), TimeComponents::of(16, 0, TimeKind::Military)),
        time_5pm_to_24h(TimeComponents::of(5, 0, TimeKind::PM), TimeComponents::of(17, 0, TimeKind::Military)),
        time_6pm_to_24h(TimeComponents::of(6, 0, TimeKind::PM), TimeComponents::of(18, 0, TimeKind::Military)),
        time_7pm_to_24h(TimeComponents::of(7, 0, TimeKind::PM), TimeComponents::of(19, 0, TimeKind::Military)),
        time_8pm_to_24h(TimeComponents::of(8, 0, TimeKind::PM), TimeComponents::of(20, 0, TimeKind::Military)),
        time_9pm_to_24h(TimeComponents::of(9, 0, TimeKind::PM), TimeComponents::of(21, 0, TimeKind::Military)),
        time_10pm_to_24h(TimeComponents::of(10, 0, TimeKind::PM), TimeComponents::of(22, 0, TimeKind::Military)),
        time_11pm_to_24h(TimeComponents::of(11, 0, TimeKind::PM), TimeComponents::of(23, 0, TimeKind::Military)),
        time_12pm_to_24h(TimeComponents::of(12, 0, TimeKind::PM), TimeComponents::of(12, 0, TimeKind::Military)),
        time_0_military_to_24h(TimeComponents::of(0, 0, TimeKind::Military), TimeComponents::of(0, 0, TimeKind::Military)),
        time_1_military_to_24h(TimeComponents::of(1, 0, TimeKind::Military), TimeComponents::of(1, 0, TimeKind::Military)),
        time_2_military_to_24h(TimeComponents::of(2, 0, TimeKind::Military), TimeComponents::of(2, 0, TimeKind::Military)),
        time_3_military_to_24h(TimeComponents::of(3, 0, TimeKind::Military), TimeComponents::of(3, 0, TimeKind::Military)),
        time_4_military_to_24h(TimeComponents::of(4, 0, TimeKind::Military), TimeComponents::of(4, 0, TimeKind::Military)),
        time_5_military_to_24h(TimeComponents::of(5, 0, TimeKind::Military), TimeComponents::of(5, 0, TimeKind::Military)),
        time_6_military_to_24h(TimeComponents::of(6, 0, TimeKind::Military), TimeComponents::of(6, 0, TimeKind::Military)),
        time_7_military_to_24h(TimeComponents::of(7, 0, TimeKind::Military), TimeComponents::of(7, 0, TimeKind::Military)),
        time_8_military_to_24h(TimeComponents::of(8, 0, TimeKind::Military), TimeComponents::of(8, 0, TimeKind::Military)),
        time_9_military_to_24h(TimeComponents::of(9, 0, TimeKind::Military), TimeComponents::of(9, 0, TimeKind::Military)),
        time_10_military_to_24h(TimeComponents::of(10, 0, TimeKind::Military), TimeComponents::of(10, 0, TimeKind::Military)),
        time_11_military_to_24h(TimeComponents::of(11, 0, TimeKind::Military), TimeComponents::of(11, 0, TimeKind::Military)),
        time_12_military_to_24h(TimeComponents::of(12, 0, TimeKind::Military), TimeComponents::of(12, 0, TimeKind::Military)),
        time_13_military_to_24h(TimeComponents::of(13, 0, TimeKind::Military), TimeComponents::of(13, 0, TimeKind::Military)),
        time_14_military_to_24h(TimeComponents::of(14, 0, TimeKind::Military), TimeComponents::of(14, 0, TimeKind::Military)),
        time_15_military_to_24h(TimeComponents::of(15, 0, TimeKind::Military), TimeComponents::of(15, 0, TimeKind::Military)),
        time_16_military_to_24h(TimeComponents::of(16, 0, TimeKind::Military), TimeComponents::of(16, 0, TimeKind::Military)),
        time_17_military_to_24h(TimeComponents::of(17, 0, TimeKind::Military), TimeComponents::of(17, 0, TimeKind::Military)),
        time_18_military_to_24h(TimeComponents::of(18, 0, TimeKind::Military), TimeComponents::of(18, 0, TimeKind::Military)),
        time_19_military_to_24h(TimeComponents::of(19, 0, TimeKind::Military), TimeComponents::of(19, 0, TimeKind::Military)),
        time_20_military_to_24h(TimeComponents::of(20, 0, TimeKind::Military), TimeComponents::of(20, 0, TimeKind::Military)),
        time_21_military_to_24h(TimeComponents::of(21, 0, TimeKind::Military), TimeComponents::of(21, 0, TimeKind::Military)),
        time_22_military_to_24h(TimeComponents::of(22, 0, TimeKind::Military), TimeComponents::of(22, 0, TimeKind::Military)),
        time_23_military_to_24h(TimeComponents::of(23, 0, TimeKind::Military), TimeComponents::of(23, 0, TimeKind::Military)),
    }
}
