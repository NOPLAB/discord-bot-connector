use async_trait::async_trait;
use songbird::{Event, EventContext, EventHandler, Songbird};
use std::sync::Arc;

pub struct DriverDisconnectNotifier {
    pub songbird_manager: Arc<Songbird>,
}

#[async_trait]
impl EventHandler for DriverDisconnectNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::DriverDisconnect(ctx) = ctx else {
            return None;
        };

        if ctx.reason.is_some() {
            return None;
        }

        self.songbird_manager.remove(ctx.guild_id).await.unwrap();

        None
    }
}
