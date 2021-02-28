use log::{debug, info};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    http::CacheHttp,
    model::{
        event::GuildMemberUpdateEvent,
        id::{GuildId, RoleId, UserId},
    },
};

use tokio::sync::RwLock;

type UserRoleCacheResult<R> = Result<R, UserRoleCacheError>;
#[derive(Error, Debug)]
pub enum UserRoleCacheError {
    #[error(
        "Failed to fetch a member with ID [{}] from the guild with ID [{}]. Caused by: {:?}",
        user_id,
        guild_id,
        cause
    )]
    FailedToFetchGuildMemberError {
        user_id: UserId,
        guild_id: GuildId,
        cause: serenity::Error,
    },
}

#[derive(Debug)]
pub struct UserRoleCache {
    user_roles: RwLock<HashMap<UserId, Vec<RoleId>>>,
}

impl UserRoleCache {
    pub fn new() -> UserRoleCache {
        Self {
            user_roles: RwLock::new(HashMap::new()),
        }
    }

    async fn update_roles(&self, user_id: UserId, roles: &[RoleId]) {
        let mut user_roles = self.user_roles.write().await;
        user_roles.insert(user_id, Vec::from(roles));
    }

    async fn cached_roles(&self, user_id: UserId) -> Option<Vec<RoleId>> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(&user_id).cloned()
    }

    pub async fn roles(
        &self,
        ctx: &Context,
        user_id: UserId,
        guild_id: GuildId,
    ) -> UserRoleCacheResult<Vec<RoleId>> {
        use UserRoleCacheError::FailedToFetchGuildMemberError;

        if let Some(cached_roles) = self.cached_roles(user_id).await {
            return Ok(cached_roles);
        }

        let member = ctx
            .http()
            .get_member(guild_id.0, user_id.0)
            .await
            .map_err(|err| FailedToFetchGuildMemberError {
                user_id,
                guild_id,
                cause: err,
            })?;

        let roles = member.roles;
        self.update_roles(user_id, &roles).await;
        Ok(roles)
    }
}

#[derive(Debug)]
pub struct UserRoleUpdateHandler {
    cache: Arc<UserRoleCache>,
}

impl UserRoleUpdateHandler {
    pub fn new(cache: Arc<UserRoleCache>) -> Self {
        debug!("Created user role update handler");
        Self { cache }
    }
}

#[async_trait]
impl EventHandler for UserRoleUpdateHandler {
    async fn guild_member_update(&self, _ctx: Context, event: GuildMemberUpdateEvent) {
        info!(
            "Received Guild Member Update Event: username={}, id={}, roles={:?}",
            event.user.name, event.user.id, event.roles
        );

        self.cache.update_roles(event.user.id, &event.roles).await;
    }
}
