use crate::{_Dialogue, BOT_DATA_PATH, Error, State};

use chrono::prelude::*;
use regex::Regex;
use std::fs::OpenOptions;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::path::Path;
use teloxide::prelude::*;

const DATABASE_FILE: &str = "api_requests.csv";

fn sanitize_input(input: &str) -> String {
    let re = Regex::new(r"\s+").expect("your regex is ass. session terminated");
    let sanitized = re.replace_all(input.trim(), " ").into_owned();
    format!("\"{}\"", sanitized)
}

fn sender_uid(msg: &Message) -> String {
    match &msg.from {
        Some(user) => user
            .username
            .clone()
            .unwrap_or_else(|| user.id.0.to_string()),
        None => String::new(),
    }
}

pub async fn request_command(bot: Bot, msg: Message, dialogue: _Dialogue) -> Result<(), Error> {
    let state = dialogue.get_or_default().await?;
    match state {
        State::Start => {
            let db = Path::new(&*BOT_DATA_PATH).join(DATABASE_FILE);
            if db.is_file() {
                let uid = sender_uid(&msg);
                let mut song_count = 0;
                let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(db)?;
                for r in rdr.records() {
                    let record = r?;
                    if record[1] == uid {
                        song_count += 1;
                    }
                }
                if song_count != 0 {
                    bot.send_message(
                        msg.chat.id,
                        format!("📝 ~ You have already requested {} songs", song_count),
                    )
                    .await?;
                }
            }
            bot.send_message(
                msg.chat.id,
                "🎙 ~ Send an author of a song to be played\n(or /cancel)",
            )
            .await?;
            dialogue.update(State::RequestArtist).await?;
        }
        State::RequestArtist => match msg.text() {
            Some(text) => {
                if text == "/cancel" {
                    dialogue.update(State::Start).await?;
                    return Ok(());
                }
                dialogue
                    .update(State::RequestSong {
                        artist: sanitize_input(text),
                    })
                    .await?;
                bot.send_message(msg.chat.id, "🎧 ~ Send a name of the song\n(or /cancel)")
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send it plain text, please")
                    .await?;
            }
        },
        State::RequestSong { artist } => match msg.text() {
            Some(text) => {
                if text == "/cancel" {
                    dialogue.update(State::Start).await?;
                    return Ok(());
                }
                dialogue
                    .update(State::RequestLink {
                        artist: artist,
                        song: sanitize_input(text),
                    })
                    .await?;
                bot.send_message(
                    msg.chat.id,
                    "🔗 ~ Send a link to the song\n(optional, or /done)",
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
            create_dir_all(&*BOT_DATA_PATH)?;
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(Path::new(&*BOT_DATA_PATH).join(DATABASE_FILE))?;
            let new_row: String = [
                Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                sender_uid(&msg),
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
