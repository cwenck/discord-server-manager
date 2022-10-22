use std::{cell::RefCell, collections::HashMap};

use regex::Regex;
use serenity::model::id::UserId;

use crate::extractor::Extractor;

use super::UserExtractorContext;

#[derive(Debug)]
pub struct MentionedUserExtractor {
    regex_template: String,
    mention_placeholder: String,
    regex_cache: RwLock<HashMap<UserId, Regex>>,
}

impl MentionedUserExtractor {
    pub fn new(regex_template: &str, mention_placeholder: &str) -> Self {
        Self {
            regex_template: regex_template.to_string(),
            mention_placeholder: mention_placeholder.to_string(),
            regex_cache: HashMap::new(),
        }
    }

    fn user_regex(&mut self, user_id: UserId) -> &Regex {
        let regex_template = self.regex_template.as_str();
        let mention_placeholder = self.mention_placeholder.as_str();

        self.regex_cache.entry(user_id).or_insert_with(|| {
            let mention_text = format!("<!{}>", user_id);
            let regex_text = regex_template.replace(&mention_placeholder, &mention_text);
            Regex::new(&regex_text).expect("Failed to compile regex")
        })
    }
}

impl Extractor<UserExtractorContext, UserId> for MentionedUserExtractor {
    fn extract(&self, text: &str, ctx: &UserExtractorContext) -> Vec<UserId> {
        let regexes: Vec<&Regex> = ctx
            .mentioned_users()
            .iter()
            .map(|user| user.id)
            .map(|user_id| self.user_regex(user_id))
            .collect();

        todo!()
    }
}
