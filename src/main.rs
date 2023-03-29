use std::env;

use dotenv::dotenv;
use regex::Regex;
use songbird::SerenityInit;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[group]
#[commands(play,leave,join)]
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
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await?;
        }

        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    let arguments = msg
        .content
        .split_whitespace()
        .skip(1)
        .collect::<Vec<&str>>()
        .join(" ");

    if arguments == "" {
        msg.channel_id.say(&ctx.http, "**Missing link**").await?;
    }

    let yt_regex = Regex::new(YOUTUBE_REGEX).unwrap();

    if !yt_regex.is_match(&arguments) {
        msg.channel_id.say(&ctx.http, "**Not a valid youtube link**").await?;
    }

    let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.").clone();

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&arguments).await {
            Ok(source) => source,
            Err(why) => {
                msg.channel_id.say(&ctx.http,&format!("Error: {}", why)).await?;
                return Ok(());
            },
        };

        handler.play_source(source);
        msg.channel_id.say(&ctx.http, "Playing song").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await?;
    }

    Ok(())
}
