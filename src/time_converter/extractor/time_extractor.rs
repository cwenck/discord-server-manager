use chrono::{DateTime, NaiveTime, TimeZone, Utc};

use crate::{extractor::Extractor, time_converter::model::TimeComponents};

use super::TimeExtractorContext;

impl<C, E> Extractor<C, NaiveTime> for E
where
    E: Extractor<C, TimeComponents>,
{
    fn extract(&self, text: &str, ctx: &C) -> Vec<NaiveTime> {
        self.extract(text, ctx)
            .into_iter()
            .map(|time_components| NaiveTime::from(time_components))
            .collect()
    }
}

impl<Tz, E> Extractor<TimeExtractorContext<Tz>, DateTime<Utc>> for E
where
    E: Extractor<TimeExtractorContext<Tz>, NaiveTime>,
    Tz: TimeZone,
{
    fn extract(&self, text: &str, ctx: &TimeExtractorContext<Tz>) -> Vec<DateTime<Utc>> {
        let date_in_local_tz = Utc::now().date().with_timezone(ctx.local_tz());
        self.extract(text, ctx)
            .into_iter()
            .filter_map(|local_time| date_in_local_tz.and_time(local_time))
            .map(|local_date_time| local_date_time.with_timezone(&Utc))
            .collect()
    }
}
