use crate::{_Dialogue, BOT_DATA_PATH, Error, State};

use chrono::prelude::*;
use regex::Regex;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use teloxide::prelude::*;
use tokio::fs::create_dir_all;

fn sanitize_input(input: &str) -> String {
    let re = Regex::new(r"\s+").expect("your regex is ass. session terminated");
    let sanitized = re.replace_all(input.trim(), " ").into_owned();
    format!("\"{}\"", sanitized)
}

pub async fn request_command(bot: Bot, msg: Message, dialogue: _Dialogue) -> Result<(), Error> {
    let state = dialogue.get_or_default().await?;
    match state {
        State::Start => {
            bot.send_message(msg.chat.id, "🎙 ~ Send an author of a song to be played")
                .await?;
            dialogue.update(State::RequestArtist).await?;
        }
        State::RequestArtist => match msg.text() {
            Some(text) => {
                dialogue
                    .update(State::RequestSong {
                        artist: sanitize_input(text),
                    })
                    .await?;
                bot.send_message(msg.chat.id, "🎧 ~ Send a name of the song")
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send it plain text, please")
                    .await?;
            }
        },
        State::RequestSong { artist } => match msg.text() {
            Some(text) => {
                dialogue
                    .update(State::RequestLink {
                        artist: artist,
                        song: sanitize_input(text),
                    })
                    .await?;
                bot.send_message(
                    msg.chat.id,
                    "🔗 ~ Send a link to the song (optional, or /done)",
                )
                .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send it plain text, please")
                    .await?;
            }
        },
        State::RequestLink { artist, song } => {
            let text = msg.text().unwrap_or_default();
            let link = match text {
                "/done" | "" => String::new(),
                _ => sanitize_input(text),
            };
            create_dir_all(&*BOT_DATA_PATH).await?;
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(Path::new(&*BOT_DATA_PATH).join("api_requests.csv"))?;
            let new_row: String = [
                Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                match msg.from {
                    Some(user) => user.username.unwrap_or_else(|| user.id.0.to_string()),
                    None => String::new(),
                },
                artist,
                song,
                link,
            ]
            .join(",");
            writeln!(file, "{}", new_row)?;
            dialogue.update(State::Start).await?;
            bot.send_message(
                msg.chat.id,
                "💡 ~ Song requested successfully!\nSee you on the event",
            )
            .await?;
        }
    };
    Ok(())
}
