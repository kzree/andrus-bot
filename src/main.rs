use std::env;

use dotenv::dotenv;
use regex::Regex;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[group]
#[commands(play)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

static YOUTUBE_REGEX: &str =
    r"http(?:s?)://(?:www\.)?youtu(?:be\.com/watch\?v=|\.be/)([\w\-_]*)(&(amp;)?‌​[\w\?‌​=]*)?";

#[tokio::main]
async fn main() {
    dotenv().ok();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("-"))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    let arguments = msg
        .content
        .split_whitespace()
        .skip(1)
        .collect::<Vec<&str>>()
        .join(" ");

    if arguments == "" {
        msg.reply(ctx, "**Missing link**").await?;
    }

    let yt_regex = Regex::new(YOUTUBE_REGEX).unwrap();

    if !yt_regex.is_match(&arguments) {
        msg.reply(ctx, "**Not a valid youtube link**").await?;
    }

    Ok(())
}
