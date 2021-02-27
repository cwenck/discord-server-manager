use std::collections::HashMap;
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

#[derive(Error, Debug)]
pub enum UserRoleUpdateHandlerError {
    #[error(
        "Failed to fetch a member with ID [{}] from the guild with ID {}. Caused by: {:?}",
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

type UserRoleUpdateHandlerResult<R> = Result<R, UserRoleUpdateHandlerError>;

#[derive(Debug)]
struct UserRoleUpdateHandler {
    user_roles: RwLock<HashMap<UserId, Vec<RoleId>>>,
}

#[async_trait]
impl EventHandler for UserRoleUpdateHandler {
    async fn guild_member_update(&self, _ctx: Context, event: GuildMemberUpdateEvent) {
        self.update_roles(event.user.id, &event.roles).await;
    }
}

#[allow(dead_code)]
impl UserRoleUpdateHandler {
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
        ctx: Context,
        user_id: UserId,
        guild_id: GuildId,
    ) -> UserRoleUpdateHandlerResult<Vec<RoleId>> {
        use UserRoleUpdateHandlerError::FailedToFetchGuildMemberError;

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
