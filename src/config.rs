use std::{collections::HashSet, env};

use chrono_tz::Tz;
use log::debug;
use serenity::model::id::RoleId;

#[derive(Debug, Clone)]
pub struct Config {
    bot_token: String,
    location_roles: HashSet<LocationRole>,
}

impl Config {
    pub fn load() -> Config {
        let bot_token =
            env::var("BOT_TOKEN").expect("Discord bot token environment variable not set");

        debug!("BOT_TOKEN={}", bot_token);

        let location_roles_str =
            env::var("LOCATION_ROLES").expect("Location roles environment variable not set");

        debug!("LOCATION_ROLES={}", bot_token);

        let location_roles = Config::parse_location_roles(&location_roles_str);
        Config {
            bot_token,
            location_roles,
        }
    }

    fn parse_location_roles(text: &str) -> HashSet<LocationRole> {
        text.split(',')
            .filter_map(|location_role_str| {
                let mut components_iter = location_role_str.splitn(2, ':');
                let maybe_timezone = components_iter.next();
                let maybe_role_id = components_iter.next();

                if let (Some(role_id), Some(timezone)) = (maybe_role_id, maybe_timezone) {
                    LocationRole::new(role_id, timezone)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    pub fn location_roles(&self) -> &HashSet<LocationRole> {
        &self.location_roles
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationRole {
    role_id: RoleId,
    timezone: Tz,
}

impl LocationRole {
    fn new(role_id: &str, timezone: &str) -> Option<LocationRole> {
        let parsed_timezone: Tz = timezone.parse().ok()?;
        let role_id_num: u64 = role_id.parse().ok()?;

        Some(LocationRole {
            role_id: RoleId(role_id_num),
            timezone: parsed_timezone,
        })
    }

    pub fn role_id(&self) -> RoleId {
        self.role_id
    }

    pub fn timezone(&self) -> Tz {
        self.timezone
    }
}
