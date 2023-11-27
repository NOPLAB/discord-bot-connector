mod commands;
mod songbird_handler;
mod voicevox;
mod wavsource;

use chrono::{Local, Timelike};
use reqwest::Url;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, FutureExt};
use songbird::{create_player, SerenityInit};
use std::cell::RefCell;
use std::env;
use std::io::Cursor;
use std::sync::Arc;
use tokio::{select, task};
use tokio_util::sync::CancellationToken;

use voicevox::api::Voicevox;
use wavsource::wav_reader;

struct Bot {
    voicevox: Arc<Voicevox>,
    cancel_token: Arc<Mutex<RefCell<Option<CancellationToken>>>>,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|cmd| commands::ping::register(cmd))
                .create_application_command(|cmd| commands::join::register(cmd))
                .create_application_command(|cmd| commands::left::register(cmd))
                .create_application_command(|cmd| commands::timer::register(cmd))
                .create_application_command(|cmd| commands::cancel::register(cmd))
        })
        .await
        .unwrap();
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(app_cmd_interaction) = interaction {
            let content = match app_cmd_interaction.data.name.as_str() {
                "ping" => {
                    commands::ping::run(&ctx, &app_cmd_interaction, Arc::clone(&self.voicevox))
                        .await
                }
                "join" => commands::join::run(&ctx, &app_cmd_interaction).await,
                "left" => commands::left::run(&ctx, &app_cmd_interaction).await,
                "timer" => {
                    commands::timer::run(
                        &ctx,
                        &app_cmd_interaction,
                        Arc::clone(&self.voicevox),
                        Arc::clone(&self.cancel_token),
                    )
                    .await
                }
                "cancel" => {
                    let cancel_token = self.cancel_token.lock().await;
                    let mut borrowed = cancel_token.borrow_mut();
                    if let Some(token) = borrowed.take() {
                        token.cancel();
                    }

                    commands::cancel::run()
                }
                _ => "not impl!".to_string(),
            };

            if let Err(why) = app_cmd_interaction
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot {
            voicevox: Arc::new(Voicevox::new(
                Url::parse(
                    &env::var("VOICEVOX_URL").expect("Expected a VOICEVOX_URL in the environment"),
                )
                .expect("Expected VOICEVOX_URL couldn't parse"),
            )),
            cancel_token: Arc::new(Mutex::new(RefCell::new(None))),
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
