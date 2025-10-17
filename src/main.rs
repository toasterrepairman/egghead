mod generator;
mod blog;

use std::collections::HashMap;
use std::env;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::StandardFramework;
use serenity::http::Typing;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio::sync::RwLock;

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

struct BlogDatabasePath;

impl TypeMapKey for BlogDatabasePath {
    type Value = Arc<String>;
}

#[group]
#[commands(help, blog)]
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
                let response = generator::get_chat_response("1.3", "You are Egghead, the world's smartest computer.", &prompt, None).unwrap();
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

async fn blog_post_generator_task(db_path: String, interval_minutes: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60));

    loop {
        interval.tick().await;

        println!("Generating new blog post...");

        // Run the blog post generation in a blocking task
        let db_path_clone = db_path.clone();
        let result = tokio::task::spawn_blocking(move || {
            match blog::generate_blog_post() {
                Ok(post) => {
                    // Open connection for this operation only
                    match rusqlite::Connection::open(&db_path_clone) {
                        Ok(conn) => {
                            match blog::save_blog_post(&conn, &post) {
                                Ok(id) => {
                                    println!("Successfully saved blog post with ID: {}", id);
                                    println!("Location: {}", post.location);
                                    println!("Activity: {}", post.activity);
                                    Ok(())
                                }
                                Err(e) => {
                                    eprintln!("Failed to save blog post: {:?}", e);
                                    Err(format!("Save error: {:?}", e))
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to open database: {:?}", e);
                            Err(format!("DB open error: {:?}", e))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to generate blog post: {:?}", e);
                    Err(format!("Generation error: {:?}", e))
                }
            }
        }).await;

        match result {
            Ok(Ok(_)) => println!("Blog post generation completed successfully"),
            Ok(Err(e)) => eprintln!("Blog post generation error: {}", e),
            Err(e) => eprintln!("Task join error: {:?}", e),
        }
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

    // Initialize the blog database
    // Use ~/.config/egghead/blog.sqlite as the default path
    let db_path = env::var("BLOG_DB_PATH").unwrap_or_else(|_| {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = format!("{}/.config/egghead", home);
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&config_dir).ok();
        format!("{}/blog.sqlite", config_dir)
    });

    // Initialize the database (create table if needed)
    match blog::init_database(&db_path) {
        Ok(_) => {
            println!("Blog database initialized at: {}", db_path);
        }
        Err(e) => {
            eprintln!("Failed to initialize blog database: {:?}", e);
            panic!("Cannot start without blog database");
        }
    };

    let db_path_arc = Arc::new(db_path.clone());

    // Get the blog post generation interval (default: 20 minutes = 3 times per hour)
    let blog_interval = env::var("BLOG_INTERVAL_MINUTES")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u64>()
        .unwrap_or(20);

    // Spawn the blog post generator task
    tokio::spawn(async move {
        blog_post_generator_task(db_path, blog_interval).await;
    });

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        // The CommandCounter Value has the following type:
        // Arc<RwLock<HashMap<String, u64>>>
        // So, we have to insert the same type to it.
        data.insert::<CommandCounter>(Arc::new(RwLock::new(HashMap::default())));

        data.insert::<MessageCount>(Arc::new(AtomicUsize::new(0)));

        data.insert::<BlogDatabasePath>(db_path_arc);
    }

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
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
    `blog` - Shows a random blog post from Egghead's life
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

#[command]
async fn blog(ctx: &Context, msg: &Message) -> CommandResult {
    let args: Vec<String> = msg.content.split_whitespace().skip(1).map(|s| s.to_string()).collect();

    let db_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<BlogDatabasePath>().expect("Expected BlogDatabasePath in TypeMap.").clone()
    };

    let response = tokio::task::spawn_blocking(move || {
        let db = match rusqlite::Connection::open(db_lock.as_ref()) {
            Ok(conn) => conn,
            Err(e) => return format!("Error opening database: {:?}", e),
        };

        if args.is_empty() || args[0].as_str() == "latest" {
            // Show latest 5 posts
            match blog::get_latest_blog_posts(&db, 5) {
                Ok(posts) => {
                    if posts.is_empty() {
                        "No blog posts yet! Wait for the next generation cycle.".to_string()
                    } else {
                        let mut response = "**Latest Blog Posts:**\n\n".to_string();
                        for post in posts {
                            response.push_str(&format!(
                                "**Post #{}** ({})\n📍 {}\n💭 {}\n🖼️ {}\n\n",
                                post.id.unwrap_or(0),
                                post.timestamp.format("%Y-%m-%d %H:%M UTC"),
                                post.location,
                                post.activity,
                                post.image_url
                            ));
                        }
                        response
                    }
                }
                Err(e) => format!("Error fetching blog posts: {:?}", e)
            }
        } else if let Ok(id) = args[0].parse::<i64>() {
            // Show specific post by ID
            match blog::get_blog_post_by_id(&db, id) {
                Ok(post) => {
                    format!(
                        "**Blog Post #{}**\n\n**When:** {}\n\n**What I'm passionate about:**\n{}\n\n**Where I am:** {}\n\n**What I'm doing:** {}\n\n**Photo:** {}",
                        post.id.unwrap_or(0),
                        post.timestamp.format("%Y-%m-%d %H:%M UTC"),
                        post.passion,
                        post.location,
                        post.activity,
                        post.image_url
                    )
                }
                Err(_) => format!("Blog post #{} not found.", id)
            }
        } else {
            "Usage: `e.blog [id|latest]`".to_string()
        }
    }).await.unwrap();

    send_message_in_parts(&ctx.http, msg, &response).await?;

    Ok(())
}
