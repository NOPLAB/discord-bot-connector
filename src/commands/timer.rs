use std::{cell::RefCell, io::Cursor, sync::Arc};

use chrono::{Local, Timelike};
use serenity::{
    builder::CreateApplicationCommand, client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};
use songbird::create_player;
use tokio::{select, sync::Mutex};
use tokio_util::sync::CancellationToken;

use crate::{voicevox::api::Voicevox, wavsource::wav_reader};

pub async fn run(
    ctx: &Context,
    app_cmd_interaction: &ApplicationCommandInteraction,
    voicevox: Arc<Voicevox>,
    cancel_token: Arc<Mutex<RefCell<Option<CancellationToken>>>>,
) -> String {
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let cancel_token = cancel_token.lock().await;
    let mut borrowed = cancel_token.borrow_mut();
    *borrowed = Some(token);

    let channel_id = app_cmd_interaction.channel_id.clone();
    let guild_id = app_cmd_interaction.guild_id.unwrap().clone();
    let http = Arc::clone(&ctx.http);
    let ctx_clone = ctx.clone();

    let _ = tokio::task::spawn(async move {
        loop {
            let local_date = Local::now();
            let duration = (59 - local_date.minute()) * 60 + (60 - local_date.second());

            if duration == 0 {
                continue;
            }

            println!("Duration: {duration}");

            let http_clone = Arc::clone(&http);

            select! {
                _ = cloned_token.cancelled() => { println!("Cancelled"); break; }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(duration as u64)) => {
                    channel_id.say(http_clone, "Wake!!").await.unwrap();
                    let mut wav = Cursor::new(voicevox.tts(&*format!("{}時です！", local_date.hour()), 1).await);
                    let (audio, _handle) = create_player(wav_reader(&mut wav));

                    let manager = songbird::get(&ctx_clone)
                        .await
                        .expect("Songbird is not initialized");

                    let handler = manager.get(guild_id).unwrap();
                    handler.lock().await.play(audio);
                }
            }
            println!("Time");
        }
    });
    "timer set!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("timer").description("A timer command")
}
