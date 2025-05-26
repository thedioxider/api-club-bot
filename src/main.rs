use teloxide::{
    dispatching::UpdateHandler,
    filter_command,
    prelude::*,
    types::ParseMode,
    utils::{command::BotCommands, html as tgfmt},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let bot = Bot::from_env();
    bot.set_my_commands(Command::bot_commands()).await?;
    log::info!("Starting the bot...");

    Dispatcher::builder(bot, schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = Update::filter_message().chain(
        filter_command::<Command, _>()
            .branch(case![Command::Start].endpoint(help_command))
            .branch(case![Command::Help].endpoint(help_command)),
    );

    dptree::entry().branch(command_handler)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
enum Command {
    #[command(hide)]
    Start,
    /// Show useful info
    #[command(aliases = ["h", "?"])]
    Help,
}

async fn help_command(bot: Bot, msg: Message) -> Result<(), Error> {
    bot.parse_mode(ParseMode::Html)
        .send_message(
            msg.chat.id,
            format!(
                "Check out the {}!",
                tgfmt::bold(
                    tgfmt::link("https://github.com/thedioxider/api-club-bot", "[repo]").as_str()
                ),
            ),
        )
        .await?;
    Ok(())
}
