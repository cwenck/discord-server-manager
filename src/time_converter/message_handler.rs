use std::{clone::Clone, collections::HashMap, fmt::Debug, sync::Arc};

use chrono::{DateTime, Utc};

use itertools::Itertools;
use log::debug;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Message, id::RoleId},
    utils::MessageBuilder,
};

use chrono_tz::Tz;

use crate::{
    config::{Config, LocationRole},
    extractor::Extractor,
    user_roles::UserRoleCache,
};

use super::{
    extractor::{
        CurrentTimeExtractor, DynamicTimeExtractor, FixedTimeExtractor, TimeExtractorContext,
    },
    model::{TimeComponents, TimeKind},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TimeZoneInfo {
    name: String,
    tz: Tz,
}

impl TimeZoneInfo {
    fn new(name: &str, tz: Tz) -> Self {
        Self {
            name: String::from(name),
            tz,
        }
    }
}

type TimeExtractor = Box<dyn Extractor<TimeExtractorContext<Tz>, DateTime<Utc>>>;

#[derive(Debug)]
pub struct MessageHandler {
    config: Arc<Config>,
    user_role_cache: Arc<UserRoleCache>,
    time_extractors: Vec<TimeExtractor>,
    input_timezones: HashMap<RoleId, LocationRole>,
    output_timezones: Vec<TimeZoneInfo>,
    output_time_fmt: String,
}

impl MessageHandler {
    pub fn new(config: Arc<Config>, user_role_cache: Arc<UserRoleCache>) -> Self {
        let time_extractors: Vec<TimeExtractor> = vec![
            Box::new(FixedTimeExtractor::new(
                r"(?i:midnight)",
                TimeComponents::new(0, 0, TimeKind::Military)
                    .expect("Expected valid time components."),
            )),
            Box::new(FixedTimeExtractor::new(
                r"(?i:noon|midday)",
                TimeComponents::new(12, 0, TimeKind::Military)
                    .expect("Expected valid time components."),
            )),
            Box::new(CurrentTimeExtractor::new(
                r"(?i:what\s+time\s+is\s+it\s+now|(?:current\s+time))",
            )),
            Box::new(DynamicTimeExtractor::new(
                r"(?i:(?<!\w)(?P<hours>(?:1[012])|(?:0?[123456789]))\s*(?::\s*(?P<minutes>(?:[12345]\d)|(?:0\d)))?\s*(?P<time_kind>[ap]m)(?!\w))",
            )),
        ];

        let input_timezones = config
            .location_roles()
            .iter()
            .copied()
            .map(|location_role| (location_role.role_id(), location_role))
            .collect();

        let output_timezones = vec![
            TimeZoneInfo::new("UK", chrono_tz::Europe::London),
            TimeZoneInfo::new("US East", chrono_tz::America::New_York),
            TimeZoneInfo::new("US West", chrono_tz::America::Los_Angeles),
        ];

        Self {
            config,
            user_role_cache,
            time_extractors,
            input_timezones,
            output_timezones,
            output_time_fmt: String::from("%_I:%M %p %Z"),
        }
    }

    // TODO : add a proper error type
    fn resolve_local_tz(&self, roles: &[RoleId]) -> Result<Tz, ()> {
        let local_timezones: Vec<_> = roles
            .iter()
            .copied()
            .filter_map(|role_id| self.input_timezones.get(&role_id))
            .map(|location_role| location_role.timezone())
            .collect();

        if local_timezones.len() != 1 {
            Err(())
        } else {
            Ok(local_timezones.first().copied().unwrap())
        }
    }

    fn format_time(&self, time: &DateTime<Utc>, tz_info: &TimeZoneInfo) -> String {
        let zoned_time = time.with_timezone(&tz_info.tz);
        let formatted_time = zoned_time.format(&self.output_time_fmt);
        format!("{:<8}: {}", tz_info.name, formatted_time)
    }

    fn construct_response(&self, times: &[DateTime<Utc>]) -> Option<String> {
        if times.is_empty() {
            return None;
        }

        let mut content = MessageBuilder::new();
        for time in times {
            let block = self
                .output_timezones
                .iter()
                .map(|tz_info| self.format_time(time, tz_info))
                .join("\n");

            content.push_codeblock(block, None);
        }

        Some(content.build())
    }

    async fn reply(&self, ctx: &Context, msg: &Message, content: &str) {
        // TODO : Do something with the errors
        let _ = msg
            .channel_id
            .send_message(ctx, |reply_msg| {
                reply_msg.content(&content);
                reply_msg.reference_message(msg);
                reply_msg
            })
            .await;
    }
}

#[async_trait]
impl EventHandler for MessageHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            // The messsage author was a bot. Skip further processing to prevent
            // the bot from responding to its own messages or those from other bots.
            return;
        }

        debug!("New Message:\n {}: {}", msg.author.name, msg.content);

        let guild_id = match msg.guild_id {
            Some(id) => id,
            // This message wasn't sent from within a server, so skip any further processing
            // since server roles are required to determine a user's local timezone.
            _ => return,
        };

        let roles_results = self
            .user_role_cache
            .roles(&ctx, msg.author.id, guild_id)
            .await;

        let roles = match roles_results {
            Ok(roles) => roles,
            // There was an error, so skip further processing
            _ => {
                // TODO : log this error
                return;
            }
        };

        let tz = match self.resolve_local_tz(&roles) {
            Ok(tz) => tz,
            _ => return, // TODO : log this error
        };

        let msg_time_in_local_tz = msg.timestamp.with_timezone(&tz).naive_local().time();
        let msg_time_components = TimeComponents::from(msg_time_in_local_tz);
        let extractor_ctx = TimeExtractorContext::new(tz, msg_time_components);

        let extracted_times: Vec<DateTime<Utc>> = self
            .time_extractors
            .iter()
            .map(|extractor: &TimeExtractor| extractor.as_ref())
            .flat_map(|extractor| extractor.extract(&msg.content, &extractor_ctx))
            .unique()
            .collect();

        if let Some(response) = self.construct_response(&extracted_times) {
            self.reply(&ctx, &msg, &response).await;
        }
    }
}
