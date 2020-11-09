use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use log::{debug, trace, warn};
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::{
    async_trait,
    client::Context,
    client::EventHandler,
    model::{
        channel::Message,
        id::{RoleId, UserId},
    },
    utils::MessageBuilder,
};

const BOT_ID: UserId = UserId(772211432098889758);
const OUTPUT_TIME_FORMAT: &'static str = "%_I:%M %p %Z";
static OUTPUT_TIMEZONES: Lazy<Vec<(&'static str, Tz)>> = Lazy::new(|| {
    vec![
        ("UK", chrono_tz::Europe::London),
        ("US East", chrono_tz::America::New_York),
        ("US West", chrono_tz::America::Los_Angeles),
    ]
});

static LOCATION_ROLE_TO_TIMEZONE: Lazy<HashMap<RoleId, Tz>> = Lazy::new(|| {
    let mut map: HashMap<RoleId, Tz> = HashMap::new();

    map.insert(
        // ID of the Location-US-East discord role
        RoleId(775033717419671602),
        chrono_tz::America::New_York,
    );

    map.insert(
        // ID of the Location-US-West discord role
        RoleId(775033814715203604),
        chrono_tz::America::Los_Angeles,
    );

    map.insert(
        // ID of the Location-UK discord role
        RoleId(775033844625047552),
        chrono_tz::Europe::London,
    );

    map
});

pub struct Handler;

static WORD_SEPARATOR: Lazy<Regex> = Lazy::new(|| {
    // Regex::new("[ ,.;-=!@#$%^&*()_+-/<>'|{}\\\\\"\\[\\]]+").expect("Failed to compile regex")
    Regex::new("[^a-zA-Z0-9:]+").expect("Failed to compile regex")
});

static TIME_REGEX_TO_PARSE_FORMATS: Lazy<Vec<Parser>> = Lazy::new(|| {
    vec![
        // Parser {
        //         regex: Regex::new("^((?:(?:1[012])|(?:0?[123456789])):(?:(?:[12345]\\d)|(?:0[123456789]))\\s(?:[AP]M))\\s([a-zA-Z]{2,3})").expect("Failed to compile regex"),
        //         format:  "%_I:%M %p".to_string(),
        // },
        Parser {
            regex: Regex::new(
                "^((?:(?:1[012])|(?:0?[123456789])):(?:(?:[12345]\\d)|(?:0\\d))\\s(?:[apAP][mM]))(?:$|[^a-zA-Z0-9])",
            )
            .expect("Failed to compile regex"),
            format: "%_I:%M %p".to_string(),
        },
        Parser {
            regex: Regex::new(
                "^((?:(?:1[012])|(?:0?[123456789])):(?:(?:[12345]\\d)|(?:0\\d))(?:[apAP][mM]))(?:$|[^a-zA-Z0-9])",
            )
            .expect("Failed to compile regex"),
            format: "%_I:%M%p".to_string(),
        },
    ]
});

async fn resolve_local_timezone(context: &Context, msg: &Message) -> Option<Tz> {
    let member = msg
        .member(context)
        .await
        .map_err(|err| {
            warn!(
                "Failed to retrive member info for author with id={} and username={}: {}",
                msg.author.id, msg.author.name, err
            )
        })
        .ok()?;

    let timezones: Vec<Tz> = member
        .roles
        .iter()
        .map(|id| LOCATION_ROLE_TO_TIMEZONE.get(id))
        .filter(Option::is_some)
        .map(Option::unwrap)
        .copied()
        .collect();

    if timezones.len() == 1 {
        timezones.get(0).copied()
    } else {
        warn!(
            "Exactly one location role expected, but found user with id={} and username={} has {} location roles",
            msg.author.id,
            msg.author.name,
            timezones.len()
        );
        None
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.author.id == BOT_ID {
            return;
        }

        debug!("[New Message] {}: {}", msg.author.name, msg.content);

        let mut times = Vec::new();

        // Parse times from the message
        // TODO : extract to method
        let mut line = &msg.content as &str;
        loop {
            trace!("[Partial Line] {}", line);

            for time_format in TIME_REGEX_TO_PARSE_FORMATS.iter() {
                let local_tz = resolve_local_timezone(&context, &msg).await;
                if let Some(time) = time_format.parse(line, local_tz) {
                    times.push(time);
                    break;
                }
            }

            match skip_word(line) {
                Some(next_line) => line = next_line,
                None => break,
            }
        }

        let mut reply_blocks = Vec::new();

        // Convert to destination time zones
        for time in times {
            debug!("[Formatting Output for Time] {}", time);

            let mut reply_lines = Vec::new();
            for (description, tz) in OUTPUT_TIMEZONES.iter() {
                let zoned_time = time.with_timezone(tz);
                let formatted_time = zoned_time.format(OUTPUT_TIME_FORMAT);

                reply_lines.push(format!("{:<8}: {}", description, formatted_time))
            }

            if reply_lines.is_empty() {
                debug!("False positive for time {}; no reponse to send", time);
            } else {
                reply_blocks.push(reply_lines.join("\n"));
            }
        }

        if (reply_blocks.is_empty()) {
            debug!("False positive for all times; no reponse to send");
            return;
        }

        let mut reply_builder = MessageBuilder::new();
        reply_blocks.iter().for_each(|block| {
            reply_builder.push_codeblock(block, None);
        });

        let reply = reply_builder.build();
        debug!("[Reply] {}", reply);

        if let Err(reason) = msg.channel_id.say(context, reply).await {
            warn!("Failed to send message reply: {}", reason);
        }
    }
}

fn skip_word(text: &str) -> Option<&str> {
    WORD_SEPARATOR.splitn(text, 2).skip(1).next()
}

#[derive(Debug, Clone)]
struct TimeComponents {
    time: String,
    timezone: Option<String>,
}

impl TimeComponents {
    fn from_str(text: &str, regex: &Regex) -> Option<TimeComponents> {
        regex.captures(text).and_then(|captures| {
            let time = captures.get(1)?.as_str().to_uppercase();
            let timezone = captures
                .get(2)
                .map(|regex_match| regex_match.as_str().to_string());

            Some(TimeComponents { time, timezone })
        })
    }
}

#[derive(Debug, Clone)]
struct Parser {
    regex: Regex,
    format: String,
}

impl Parser {
    fn parse(&self, text: &str, local_tz: Option<Tz>) -> Option<DateTime<Utc>> {
        let time_components = TimeComponents::from_str(text, &self.regex)?;
        debug!("[Time Components] {:?}", time_components);

        // Only proceed if a local timezone was specified. May change later.
        let timezone = local_tz?;
        debug!("[Local Timezone] {}", timezone);

        let local_time = parse_time(&time_components.time, &self.format)?;
        debug!("[Parsed Time] {}", local_time);

        let date = Utc::now().date().with_timezone(&timezone);
        debug!("[Date in Local Timezone] {}", date);

        let datetime = date.and_time(local_time)?;
        debug!("[DateTime in Local Timezone] {}", datetime);

        let utc_datetime = datetime.with_timezone(&Utc);
        debug!("[DateTime in UTC] {}", utc_datetime);

        Some(utc_datetime)
    }
}

fn parse_time(text: &str, format: &str) -> Option<NaiveTime> {
    match NaiveTime::parse_from_str(text, format) {
        Ok(time) => Some(time),
        Err(reason) => {
            warn!("Failed to parse local time: {}", reason);
            None
        }
    }
}
