use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

pub async fn run(ctx: &Context, interaction: &ApplicationCommandInteraction) -> String {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let Ok(()) = manager.leave(guild_id).await else {
        return "botがボイスチャンネルにいないよ".to_string();
    };

    "bye!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("left").description("bot will left your vc")
}
