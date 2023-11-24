mod commands;
mod songbird_handler;
mod voicevox;

use reqwest::Url;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, FutureExt};
use songbird::SerenityInit;
use std::cell::RefCell;
use std::env;
use std::sync::Arc;
use tokio::{select, task};
use tokio_util::sync::CancellationToken;

use voicevox::api::Voicevox;

// use serenity::model::id::GuildId;

struct Bot {
    voicevox: Voicevox,
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
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(),
                "join" => commands::join::run(&ctx, &command).await,
                "left" => commands::left::run(&ctx, &command).await,
                "timer" => {
                    let token = CancellationToken::new();
                    let cloned_token = token.clone();

                    let cancel_token = self.cancel_token.lock().await;
                    let mut borrowed = cancel_token.borrow_mut();
                    *borrowed = Some(token);

                    let channel_id = command.channel_id.clone();
                    let http = Arc::clone(&ctx.http);

                    let _ = task::spawn(async move {
                        loop {
                            let http_clone = Arc::clone(&http);
                            select! {
                                _ = cloned_token.cancelled().fuse() => { println!("Cancelled"); break;}
                                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)).fuse() => {
                                    channel_id.say(http_clone, "Wake!!").await.unwrap();
                                }
                            }
                            println!("Time");
                        }
                    });
                    commands::timer::run()
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

            if let Err(why) = command
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
            voicevox: Voicevox::new(
                Url::parse(
                    &env::var("VOICEVOX_URL").expect("Expected a VOICEVOX_URL in the environment"),
                )
                .expect("Expected VOICEVOX_URL couldn't parse"),
            ),
            cancel_token: Arc::new(Mutex::new(RefCell::new(None))),
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
