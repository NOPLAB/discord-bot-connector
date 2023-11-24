use serenity::builder::CreateApplicationCommand;

pub fn run() -> String {
    "timer cancel!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("cancel").description("A cancel command")
}
