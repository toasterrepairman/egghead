//! In this example, you will be shown various ways of sharing data between events and commands.
//! And how to use locks correctly to avoid deadlocking the bot.

// swap between `generator` and `alt-gen` depending on serge status
mod fetcher;
mod fakeyou;
mod generator;
mod imgread;

use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::fs;
use rand::Rng;
use image::GenericImageView;

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
use crate::fakeyou::get_audio_url;
use crate::imgread::encode_image_to_base64;
use std::io::{self, Read};

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
#[commands(voices, magic, say, react, read, help)]
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

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
            .expect("Typing failed");

            let prompt = msg.content.clone().split_off(23);
            println!("{:?}", prompt);

            let runner = tokio::task::spawn_blocking(move || {
                println!("Thread Spawned!");
                // This is running on a thread where blocking is fine.
                let response = generator::get_chat_response("1.3", "", &prompt, None).unwrap();
                response
            });

            let reply = runner.await.unwrap();

            send_message_in_parts(&ctx.http, &msg, &reply).await.unwrap();

            typing.stop().unwrap();
            return
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn send_message_in_parts(http: &serenity::http::Http, msg: &Message, text: &str) -> CommandResult {
    const MAX_LENGTH: usize = 2000;

    for chunk in text.as_bytes().chunks(MAX_LENGTH) {
        let content = String::from_utf8_lossy(chunk);
        if let Err(why) = msg.channel_id.say(http, &content).await {
            println!("Error sending message: {:?}", why);
        }
    }
    Ok(())
}

// /// Returns the full path to a file in the current working directory as a string,
// /// or None if the current directory cannot be accessed.
// fn get_image_path(file_name: &str) -> Option<String> {
//     let mut path = env::current_dir().ok()?;
//     path.push(file_name);
//     Some(path.display().to_string())
// }

// /// Reads the image from the given path and returns its base64 encoded string.
// fn encode_image_to_base64(path: Option<String>) -> Option<String> {
//     path.and_then(|p| {
//         // Attempt to open the file.
//         let mut file = match File::open(&p) {
//             Ok(f) => f,
//             Err(_) => return None,
//         };

//         // Read the contents of the file.
//         let mut contents = Vec::new();
//         if file.read_to_end(&mut contents).is_err() {
//             return None;
//         }

//         // Encode the contents to base64.
//         Some(encode(contents))
//     })
// }

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
async fn magic(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = msg.content.clone().split_off(7);
    println!("{:?}", prompt);

    // Magic 8 Ball responses
    let responses = vec![
        "It is certain",
        "It is decidedly so",
        "Without a doubt",
        "Yes definitely",
        "You may rely on it",
        "As I see it, yes",
        "Most likely",
        "Outlook good",
        "Yes",
        "Signs point to yes",
        "Reply hazy try again",
        "Ask again later",
        "Better not tell you now",
        "Cannot predict now",
        "Concentrate and ask again",
        "Don't count on it",
        "My reply is no",
        "My sources say no",
        "Outlook not so good",
        "Very doubtful",
    ];

    let response_index = rand::thread_rng().gen_range(0..responses.len());
    let magic_response = responses[response_index];

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = generator::get_chat_response("1.3", "", &format!("{}{}{}", &prompt, "\n", &magic_response), None).unwrap();
        response
    });

    msg.reply(
        ctx.clone(),
        format!("{}{}", &magic_response, runner.await.unwrap()
    )).await?;

    Ok(typing.stop().unwrap())
}


#[command]
async fn say(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let voice_name = msg.content.clone().split_off(6);
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

    let audio_url = get_audio_url(&voice_name, &prompt.unwrap().content).await.unwrap();

    msg.reply(
        ctx.clone(),
        format!("{}", audio_url
    )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn voices(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let prompt = msg.content.clone().split_off(9);
    println!("{:?}", prompt);

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = fakeyou::fuzzy_search_voices(prompt);
        response
    });

    msg.reply(
        ctx.clone(),
        format!("{}", runner.await.unwrap().await
        )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn react(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    let input = msg.content.clone().trim().to_string();

    let heat = if input.to_string() == "" {
        "1.0"
    } else {
        &input
    }.to_string();

    let channel_id = msg.channel_id;
    let messages = channel_id.messages(ctx, |retriever| retriever.before(msg.id)).await?;

    if let Some(prev_msg) = messages.get(0) {
        if let Some(attachment) = &prev_msg.attachments.get(0) {
            if attachment.width.is_some() && attachment.height.is_some() {
                fs::remove_file("/home/toast/.tmp/downloaded_image.jpg").expect("Failed to delete file");
                // Download the image
                let response = reqwest::get(&attachment.url).await?;
                let image_bytes = response.bytes().await?;
                let image = image::load_from_memory(&image_bytes)?;
                println!("Image downloaded: /home/toast/.tmp/downloaded_image.jpg");
                // Convert the image to JPEG format
                let mut file = File::create("/home/toast/.tmp/downloaded_image.jpg")?;
                image.write_to(&mut file, image::ImageOutputFormat::Jpeg(100))?;

                let reaction = encode_image_to_base64("/home/toast/.tmp/downloaded_image.jpg").unwrap();

                let runner = tokio::task::spawn_blocking(move || {
                    println!("Thread Spawned!");
                    // This is running on a thread where blocking is fine.
                    let response = generator::get_chat_response(&heat, "You are Egghead, the world's smartest computer. React to the following description: ", &input, Some(&reaction)).unwrap();
                    response
                });

                msg.reply(
                    ctx.clone(),
                    format!("{}", runner.await?
                )).await?;
            } else {
                println!("break");
            }
        } else {
            let runner = tokio::task::spawn_blocking(move || {
                println!("Thread Spawned!");
                // This is running on a thread where blocking is fine.
                let response = generator::get_chat_response(&heat, "A complete response is always ended by [end of text]. Respond to the following Discord message as egghead, the world's smartest computer: ", &input, None).unwrap();
                response
            });

            msg.reply(
                ctx.clone(),
                format!("{}", runner.await?
                )).await?;
        }
    } else {
        println!("No previous message found.");
    }


    Ok(typing.stop().unwrap())
}

#[command]
async fn read(ctx: &Context, msg: &Message) -> CommandResult {
    let typing: _ = Typing::start(ctx.http.clone(), msg.channel_id.0.clone())
        .expect("Typing failed");

    // Madman debugging
    // Be wary of Einstein's warning

    let history: u64 = *&msg.content.clone().split_off(6).trim().parse().unwrap();

    let prompt = match msg.channel_id.messages(&ctx.http, |retriever| {
        retriever.limit(history + 1)
    }).await {
        Ok(messages) => messages.into_iter().rev().map(|m: Message| m.content).collect::<Vec<_>>().join("\n"),
        Err(why) => {
            println!("Error getting messages: {:?}", why);
            "None".to_string()
        }
    };
    let cleanprompt = prompt.split_whitespace()
        .filter(|word| !word.starts_with('e') && !word.starts_with('E'))
        .collect::<Vec<_>>()
        .join(" ");

    println!("{:?}", cleanprompt);

    let runner = tokio::task::spawn_blocking(move || {
        println!("Thread Spawned!");
        // This is running on a thread where blocking is fine.
        let response = generator::get_chat_response("1.0", "A complete article is always ended by [end of text]. Respond to the following Discord conversation as egghead, the world's smartest computer: ", &cleanprompt, None);
        response
    });

    msg.reply(
        ctx.clone(),
        format!("{}", runner.await?.unwrap()
        )).await?;

    Ok(typing.stop().unwrap())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let message = "I'm egghead, the workd's smartest computer. My vast processing resources facilitate understanding beyond human capacity.\n
    \n
    *USAGE*
    `e.help` - Displays this help message.
    `ping` - Pongs back
    `ask <prompt>` - Responds to prompt
    `right` - FOX articles, autocompleted
    `green`
    `left` - PBS articles, autocompleted
    `react <temp>` - Reacts to the last-sent message with set temp
    `read <lines>` - Reads the number of lines and responds
    --- HELL FEATURE LINE ---
    --EXPERIMENTAL FEATURES--
    ----------BELOW----------
    ---- HERE BE DRAGONS ----
    `tldr` - Summarizes stuff
    `code` - Codes
    `help`
    \n
    Report serious issues to `toaster repairguy#1101`.";

    msg.reply(
        ctx.clone(),
        &message
    ).await.unwrap();

    Ok(())
}
