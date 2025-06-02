#[macro_use]
extern crate version;
use std::time::Duration;

use teloxide::{
    dispatching::UpdateHandler,
    filter_command,
    prelude::*,
    types::{BotCommandScope, MediaKind, MessageKind, ParseMode},
    utils::{command::BotCommands, html as tgfmt},
};
use tokio::time::sleep;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    // TELOXIDE_TOKEN env variable is required
    let bot = Bot::from_env();
    // make bot available commands appear in helper menu
    bot.set_my_commands(PrivateCommand::bot_commands())
        .scope(BotCommandScope::AllPrivateChats)
        .await?;
    log::info!("Starting the bot...");

    Dispatcher::builder(bot, schema())
        .enable_ctrlc_handler()
        .distribution_function(|upd: &Update| {
            if let Some(chat) = upd.chat() {
                if chat.is_supergroup() {
                    None
                } else {
                    Some(chat.id)
                }
            } else {
                None
            }
        })
        .default_handler(|upd| async move {
            log::debug!("Unhandled update: {:?}", upd);
        })
        .build()
        .dispatch()
        .await;
    Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;
    let version_command = |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, format!("Bot version: v{}", version!()))
            .await?;
        Ok(())
    };

    let private_command_handler = filter_command::<PrivateCommand, _>()
        .branch(case![PrivateCommand::Start].endpoint(help_command))
        .branch(case![PrivateCommand::Help].endpoint(help_command))
        .branch(case![PrivateCommand::Version].endpoint(version_command));
    // filter member join/leave messages and delete them
    let member_update_msg_handler = dptree::filter(|msg: Message| match msg.kind {
        MessageKind::NewChatMembers(_) | MessageKind::LeftChatMember(_) => true,
        _ => false,
    })
    .endpoint(member_update_msg_endpoint);
    // greets newcomers
    let new_member_handler = dptree::filter(|cmu: ChatMemberUpdated| {
        cmu.chat.is_supergroup()
            && cmu.old_chat_member.is_left()
            && cmu.new_chat_member.is_present()
    })
    .endpoint(new_member_endpoint);

    dptree::entry()
        .branch(
            Update::filter_message()
                .branch(member_update_msg_handler)
                // filter only messages that are sent directly to bot
                .chain(dptree::filter(|msg: Message| msg.chat.is_private()))
                .branch(private_command_handler)
                // log unhandled messages that are sent directly to bot
                .endpoint(log_message),
        )
        .branch(Update::filter_chat_member().branch(new_member_handler))
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
enum PrivateCommand {
    #[command(hide)]
    Start,
    /// Show useful info
    #[command(aliases = ["h", "?"], hide_aliases)]
    Help,
    #[command(alias = "ver", hide)]
    Version,
}

async fn log_message(msg: Message) -> Result<(), Error> {
    let media = if let MessageKind::Common(mcommon) = msg.kind {
        mcommon.media_kind
    } else {
        return Ok(());
    };
    let with_header = |header: &str, content: &str| {
        format!(
            "~~> {}:{}{}",
            header,
            if content.lines().count() <= 1 {
                " "
            } else {
                "\n"
            },
            content
        )
    };
    let contents = match media {
        MediaKind::Text(m_text) => with_header("Text", &m_text.text),
        _ => String::new(),
    };
    if contents.is_empty() {
        return Ok(());
    }
    log::info!(
        "Message from {}:\n{}",
        if let Some(user) = msg.from {
            format!(
                "user {}",
                if let Some(uname) = user.username {
                    format!("@{}", uname)
                } else {
                    format!("id#{}", user.id)
                }
            )
        } else {
            String::from("undefined")
        },
        contents
    );
    Ok(())
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

async fn new_member_endpoint(bot: Bot, cmu: ChatMemberUpdated) -> Result<(), Error> {
    let user = cmu.new_chat_member.user;
    // greet user with message
    let greeter = (&bot)
        .parse_mode(ParseMode::Html)
        .send_message(
            cmu.chat.id,
            format!(
                "{}, {}!",
                tgfmt::italic("🎶 ~ Welcome aboard"),
                tgfmt::bold(tgfmt::user_mention(user.id, user.first_name.as_str()).as_str()),
            ),
        )
        .disable_notification(true)
        .await?;
    // wait for 15 minutes and then clear the greeing message
    sleep(Duration::from_secs(15 * 60)).await;
    bot.delete_message(greeter.chat.id, greeter.id).await?;
    Ok(())
}
