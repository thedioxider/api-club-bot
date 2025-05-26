use teloxide::{
    dispatching::UpdateHandler,
    filter_command,
    prelude::*,
    types::{MessageKind, ParseMode},
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
    use dptree::{case, entry};

    let command_handler = Update::filter_message().chain(
        filter_command::<Command, _>()
            .branch(case![Command::Start].endpoint(help_command))
            .branch(case![Command::Help].endpoint(help_command)),
    );
    let member_update_msg_handler = entry()
        .filter(|msg: Message| match msg.kind {
            MessageKind::NewChatMembers(_) | MessageKind::LeftChatMember(_) => true,
            _ => false,
        })
        .endpoint(member_update_msg_endpoint);

    entry().branch(
        Update::filter_message()
            .branch(command_handler)
            .branch(member_update_msg_handler),
    )
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

async fn member_update_msg_endpoint(bot: Bot, msg: Message) -> Result<(), Error> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    Ok(())
}
