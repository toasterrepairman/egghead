//! In this example, you will be shown various ways of sharing data between events and commands.
//! And how to use locks correctly to avoid deadlocking the bot.

mod generator;
mod fetcher;

use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{Args, CommandResult, StandardFramework};
use serenity::http::Typing;
use serenity::model::channel::{AttachmentType, Message};
use serenity::model::gateway::Ready;
use serenity::model::prelude::Activity;
use serenity::prelude::*;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use tokio::runtime::Runtime;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Duration;
use serenity::futures::TryFutureExt;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
// These places are usually where either Context or Client is present.
//
// Documentation about TypeMap can be found here:
// https://docs.rs/typemap_rev/0.1/typemap_rev/struct.TypeMap.html
struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}

struct MessageCount;

impl TypeMapKey for MessageCount {
    // While you will be using RwLock or Mutex most of the time you want to modify data,
    // sometimes it's not required; like for example, with static data, or if you are using other
    // kinds of atomic operators.
    //
    // Arc should stay, to allow for the data lock to be closed early.
    type Value = Arc<AtomicUsize>;
}

#[group]
#[commands(ping, command_usage, ask, help, news, wiki, hn, script, feel, say)]
struct General;

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Running command '{}' invoked by '{}'", command_name, msg.author.tag());

    let counter_lock = {
        // While data is a RwLock, it's recommended that you always open the lock as read.
        // This is mainly done to avoid Deadlocks for having a possible writer waiting for multiple
        // readers to close.
        let data_read = ctx.data.read().await;

        // Since the CommandCounter Value is wrapped in an Arc, cloning will not duplicate the
        // data, instead the reference is cloned.
        // We wrap every value on in an Arc, as to keep the data lock open for the least time possible,
        // to again, avoid deadlocking it.
        data_read.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.").clone()
    };

    // Just like with client.data in main, we want to keep write locks open the least time
    // possible, so we wrap them on a block so they get automatically closed at the end.
    {
        // The HashMap of CommandCounter is wrapped in an RwLock; since we want to write to it, we will
        // open the lock in write mode.
        let mut counter = counter_lock.write().await;

        // And we write the amount of times the command has been called to it.
        let entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;
    }

    true
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // We are verifying if the bot id is the same as the message author id.
        if msg.author.id != ctx.cache.current_user_id()
            && msg.content.to_lowercase().contains("owo")
        {
            // Since data is located in Context, this means you are also able to use it within events!
            let count = {
                let data_read = ctx.data.read().await;
                data_read.get::<MessageCount>().expect("Expected MessageCount in TypeMap.").clone()
            };

            // Here, we are checking how many "owo" there are in the message content.
            let owo_in_msg = msg.content.to_ascii_lowercase().matches("owo").count();

            // Atomic operations with ordering do not require mut to be modified.
            // In this case, we want to increase the message count by 1.
            // https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_add
            count.fetch_add(owo_in_msg, Ordering::SeqCst);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("e."))
        .before(before)
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    // This is where we can initially insert the data we desire into the "global" data TypeMap.
    // client.data is wrapped on a RwLock, and since we want to insert to it, we have to open it in
    // write mode, but there's a small thing catch:
    // There can only be a single writer to a given lock open in the entire application, this means
    // you can't open a new write lock until the previous write lock has closed.
    // This is not the case with read locks, read locks can be open indefinitely, BUT as soon as
    // you need to open the lock in write mode, all the read locks must be closed.
    //
    // You can find more information about deadlocks in the Rust Book, ch16-03:
    // https://doc.rust-lang.org/book/ch16-03-shared-state.html
    //
    // All of this means that we have to keep locks open for the least time possible, so we put
    // them inside a block, so they get closed automatically when dropped.
    // If we don't do this, we would never be able to open the data lock anywhere else.
    //
    // Alternatively, you can also use `ClientBuilder::type_map_insert` or
    // `ClientBuilder::type_map` to populate the global TypeMap without dealing with the RwLock.
    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        // The CommandCounter Value has the following type:
        // Arc<RwLock<HashMap<String, u64>>>
        // So, we have to insert the same type to it.
        data.insert::<CommandCounter>(Arc::new(RwLock::new(HashMap::default())));

        data.insert::<MessageCount>(Arc::new(AtomicUsize::new(0)));
    }

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

/// Usage: `~command_usage <command_name>`
/// Example: `~command_usage ping`
#[command]
async fn command_usage(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "I require an argument to run this command.").await?;
            return Ok(());
        },
    };

    // Yet again, we want to keep the locks open for the least time possible.
    let amount = {
        // Since we only want to read the data and not write to it, we open it in read mode,
        // and since this is open in read mode, it means that there can be multiple locks open at
        // the same time, and as mentioned earlier, it's heavily recommended that you only open
        // the data lock in read mode, as it will avoid a lot of possible deadlocks.
        let data_read = ctx.data.read().await;

        // Then we obtain the value we need from data, in this case, we want the command counter.
        // The returned value from get() is an Arc, so the reference will be cloned, rather than
        // the data.
        let command_counter_lock =
            data_read.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.").clone();

        let command_counter = command_counter_lock.read().await;
        // And we return a usable value from it.
        // This time, the value is not Arc, so the data will be cloned.
        command_counter.get(&command_name).map_or(0, |x| *x)
    };

    if amount == 0 {
        msg.reply(ctx, format!("The command `{}` has not yet been used.", command_name)).await?;
    } else {
        msg.reply(
            ctx,
            format!("The command `{}` has been used {} time/s this session!", command_name, amount),
        )
            .await?;
    }

    Ok(())
}

#[command]
async fn ask(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = msg.content.clone().split_off(6);
    println!("{:?}", prompt);

    let runner = tokio::task::spawn_blocking(async move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = generator::call_api(&prompt).unwrap();
        response
    });

    msg.reply_mention(
        ctx.clone(),
        format!("{:?}", runner.await
    )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn script(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = msg.content.clone().split_off(9);
    println!("{:?}", prompt);

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = format!("{}", generator::script(
            &prompt,
            "",
        ));
        println!("{}", &response);
        response
    });

    msg.reply_mention(
        ctx.clone(),
        format!("{}", runner.await?,
        )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn news(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = fetcher::get_random_headline_from_rss_link(
        "https://moxie.foxnews.com/google-publisher/latest.xml"
    ).await.expect("couldnt rss right");
    let title = prompt.clone();
    println!("{:?}", &prompt);

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = format!("{}", generator::ask(
            &prompt,
            "",
        ));
        println!("{}", &response);
        response
    });

    msg.reply(
        ctx.clone(),
        format!("Title: {:0} \n{:1}", title, runner.await?,
        )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn wiki(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let input = match &msg.content.len() {
        6 => None,
        _ => Some(msg.content.as_str().split_at(7).1)
    };

    let prompt = fetcher::get_wikipedia_summary(input).await.unwrap();
    println!("{:?}", &input);

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = format!("{}", generator::wiki(
            &prompt,
            "",
        ));
        println!("{}", &response);
        response
    });

    msg.reply_mention(
        ctx.clone(),
        format!("{}", runner.await?,
        )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn hn(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = fetcher::get_latest_hn_comment().await.unwrap();

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = format!("{}", generator::hn(
            &prompt,
            "",
        ));
        println!("{}", &response);
        response
    });

    msg.reply_mention(
        ctx.clone(),
        format!("{}", runner.await?,
        )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn feel(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = match msg.channel_id.messages(&ctx.http, |retriever| {
        retriever.limit(2)
    }).await {
        Ok(messages) => messages.last().cloned(),
        Err(why) => {
            println!("Error getting messages: {:?}", why);
            None
        }
    };
    println!("{:?}", prompt);

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = format!("{:?}", generator::analyze(
            &prompt.unwrap().content
        ));
        println!("{}", &response);
        response
    });

    msg.reply_mention(
        ctx.clone(),
        format!("{}", runner.await?
        )).await?;

    Ok(typing.stop().unwrap())
}

// This next function will be borderline demonic.
// This (should) enable TTS over API though. This will eventually break :)
const TTS_API_URL: &str = "https://api.fakeyou.com/tts/inference";

#[derive(Debug, Serialize, Deserialize)]
struct TTSJobResponse {
    job_token: String,
}


#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let message = "I'm egghead, the workd's smartest computer. My vast processing resources facilitate understanding beyond human capacity.\n
    \n
    *USAGE*
    `e.help` - Displays this help message.
    `e.ask <PROMPT>` - Asks the model a user-submitted question. May fail with elaborate prompts.
    (Coming soon) `e.see <PROMPT>` - Generate an image with Stable Diffusion.
    `e.wiki <PROMPT>` - Finishes the listed (or random if <PROMPT> is blank) article with AI.
    `e.hn` - Finishes the latest HN comment with AI.
    `e.feel` - Sentiment analysis for the last-sent message.
    \n
    Report serious issues to `toaster repairguy#1101`.";

    msg.reply(
        ctx.clone(),
        &message
    ).await.unwrap();

    Ok(())
}

