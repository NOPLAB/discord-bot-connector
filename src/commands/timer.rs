use serenity::builder::CreateApplicationCommand;

pub fn run() -> String {
    // let channel_id = interaction.channel_id.clone();
    // let http_clone = Arc::clone(&ctx.http);

    // timer.schedule_with_delay(chrono::Duration::seconds(5), move || {
    //     println!("Wake");
    //     let http_clone = Arc::clone(&http_clone);
    //     task::spawn(async move {
    //         channel_id.say(http_clone, "Wake!!").await.unwrap();
    //     });
    // });

    "timer set!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("timer").description("A timer command")
}
