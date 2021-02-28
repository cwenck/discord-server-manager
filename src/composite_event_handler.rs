use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Message, event::GuildMemberUpdateEvent},
};

type Handler = Box<dyn EventHandler>;

pub struct CompositeEventHandler {
    handlers: Vec<Handler>,
}

impl CompositeEventHandler {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn event_handler<H>(mut self, handler: H) -> Self
    where
        H: EventHandler + 'static,
    {
        self.handlers.push(Box::new(handler));
        self
    }
}

#[async_trait]
impl EventHandler for CompositeEventHandler {
    async fn guild_member_update(&self, ctx: Context, event: GuildMemberUpdateEvent) {
        for handler in &self.handlers {
            handler
                .guild_member_update(ctx.clone(), event.clone())
                .await
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        for handler in &self.handlers {
            handler.message(ctx.clone(), msg.clone()).await
        }
    }
}
