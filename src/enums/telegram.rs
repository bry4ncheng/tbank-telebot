use teloxide::utils::command::*;
#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Initialise the telegram bot.")]
    Start,
    #[command(description = "Get help from the telegram bot.")]
    Help
}