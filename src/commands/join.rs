use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use songbird::CoreEvent;

use crate::songbird_handler::DriverDisconnectNotifier;

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> String {
    let guild = ctx.cache.guild(&interaction.guild_id.unwrap()).unwrap();
    let channel_id = guild
        .voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => return "ボイスチャンネルにいないよ".to_string(),
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(h) = manager.get(guild.id) {
        h.lock().await.join(connect_to).await.unwrap();
    } else {
        let (h, _success) = manager.join(guild.id, connect_to).await;

        h.lock().await.add_global_event(
            CoreEvent::DriverDisconnect.into(),
            DriverDisconnectNotifier {
                songbird_manager: manager,
            },
        );
    }

    "i'm coming!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("join").description("bot will join your vc")
}
