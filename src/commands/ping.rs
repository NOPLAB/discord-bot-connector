use std::{io::Cursor, sync::Arc};

use serenity::{
    builder::CreateApplicationCommand, client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};
use songbird::create_player;

use crate::{voicevox::api::Voicevox, wavsource::wav_reader};

pub async fn run(
    ctx: &Context,
    cmd_interaction: &ApplicationCommandInteraction,
    voicevox: Arc<Voicevox>,
) -> String {
    let ctx_clone = ctx.clone();
    let cmd_interaction_clone = cmd_interaction.clone();

    let _ = tokio::task::spawn(async move {
        let mut wav = Cursor::new(voicevox.tts("ぽんぐ！", 1).await);
        let (audio, _handle) = create_player(wav_reader(&mut wav));

        let manager = songbird::get(&ctx_clone)
            .await
            .expect("Songbird is not initialized");

        let handler = manager
            .get(cmd_interaction_clone.guild_id.unwrap())
            .unwrap();
        handler.lock().await.play(audio);
    });

    "pong!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}
