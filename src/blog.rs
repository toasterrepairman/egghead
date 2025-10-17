use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply, http::StatusCode};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub passion: String,
    pub location: String,
    pub activity: String,
    pub image_url: String,
}

#[derive(Debug)]
struct DatabaseError;

impl warp::reject::Reject for DatabaseError {}

pub fn init_database(db_path: &str) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS blog_posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            passion TEXT NOT NULL,
            location TEXT NOT NULL,
            activity TEXT NOT NULL,
            image_url TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

pub fn fetch_guardian_headlines() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client
        .get("https://www.theguardian.com/world/rss")
        .send()?
        .bytes()?;

    let channel = rss::Channel::read_from(&response[..])?;

    let titles: Vec<String> = channel
        .items()
        .iter()
        .take(5)
        .filter_map(|item| item.title().map(|t| t.to_string()))
        .collect();

    Ok(titles)
}

pub fn generate_location_with_context(context: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let prompt = format!(
        "Based on these recent world news headlines:\n{}\n\nWhere in the world might you want to be right now? Reply with just a city and country name, nothing else.",
        context
    );

    let request_data = serde_json::json!({
        "model": "gemma3:270m",
        "prompt": prompt,
        "stream": false,
        "temperature": 0.8,
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    let location = response_json["response"]
        .as_str()
        .unwrap_or("Sydney, Australia")
        .trim()
        .to_string();

    Ok(location)
}

pub fn generate_activity_with_context(location: &str, context: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let prompt = format!(
        "Given this context from world news:\n{}\n\nYou're in {}. What are you doing right now? Reply with a short, engaging description (one sentence) of your activity.",
        context, location
    );

    let request_data = serde_json::json!({
        "model": "gemma3:270m",
        "prompt": prompt,
        "stream": false,
        "temperature": 0.9,
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .header("Content-Type", "application/json")
        .json(&request_data)
        .send()?;

    let response_json: serde_json::Value = response.json()?;
    let activity = response_json["response"]
        .as_str()
        .unwrap_or("Enjoying the sun!")
        .trim()
        .to_string();

    Ok(activity)
}

pub fn search_unsplash_image(location: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Using Unsplash Source for simple, direct image URLs without API key
    // Format: https://source.unsplash.com/1600x900/?{query}
    let query = location.replace(" ", ",").replace(",", "%2C");
    let image_url = format!("https://source.unsplash.com/1600x900/?{}", query);

    // Verify the URL is accessible
    match client.head(&image_url).send() {
        Ok(_) => Ok(image_url),
        Err(_) => {
            // Fallback to a generic scenic image
            Ok("https://source.unsplash.com/1600x900/?scenic,travel".to_string())
        }
    }
}

pub fn save_blog_post(conn: &Connection, post: &BlogPost) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO blog_posts (timestamp, passion, location, activity, image_url)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            post.timestamp.to_rfc3339(),
            post.passion,
            post.location,
            post.activity,
            post.image_url,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn generate_blog_post() -> Result<BlogPost, Box<dyn std::error::Error>> {
    // Fetch Guardian headlines
    let headlines = fetch_guardian_headlines()?;
    let context = headlines.join("\n");

    // Generate location based on context
    let location = generate_location_with_context(&context)?;

    // Generate activity based on location and context
    let activity = generate_activity_with_context(&location, &context)?;

    // Search for image
    let image_url = search_unsplash_image(&location)?;

    // Use the passion text from the blog post image
    let passion = "Exploring new technologies: I'm constantly learning and experimenting with new gadgets, apps, and platforms. I'm constantly trying to understand how these innovations are shaping the future and how they can be used to improve lives. Learning new things: I'm always eager to learn and expand my knowledge base. I'm passionate about staying up-to-date with the latest trends, developments, and exciting innovations. Understanding the world: I'm fascinated by the interconnectedness of the world and the ways technology can influence our daily lives. I'm constantly looking for ways to make a positive impact and contribute to a more sustainable and equitable future. Building relationships: I'm a good listener and always willing to help others. I enjoy collaborating with people from different backgrounds and cultures.".to_string();

    Ok(BlogPost {
        id: None,
        timestamp: Utc::now(),
        passion,
        location,
        activity,
        image_url,
    })
}

pub fn get_blog_post_by_id(conn: &Connection, id: i64) -> Result<BlogPost, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, passion, location, activity, image_url FROM blog_posts WHERE id = ?1"
    )?;

    let post = stmt.query_row(params![id], |row| {
        Ok(BlogPost {
            id: Some(row.get(0)?),
            timestamp: row.get::<_, String>(1)?.parse().unwrap_or_else(|_| Utc::now()),
            passion: row.get(2)?,
            location: row.get(3)?,
            activity: row.get(4)?,
            image_url: row.get(5)?,
        })
    })?;

    Ok(post)
}

pub fn get_latest_blog_posts(conn: &Connection, limit: usize) -> Result<Vec<BlogPost>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, passion, location, activity, image_url
         FROM blog_posts
         ORDER BY id DESC
         LIMIT ?1"
    )?;

    let posts = stmt.query_map(params![limit], |row| {
        Ok(BlogPost {
            id: Some(row.get(0)?),
            timestamp: row.get::<_, String>(1)?.parse().unwrap_or_else(|_| Utc::now()),
            passion: row.get(2)?,
            location: row.get(3)?,
            activity: row.get(4)?,
            image_url: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(posts)
}

pub fn get_random_blog_post(conn: &Connection) -> Result<BlogPost, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, passion, location, activity, image_url
         FROM blog_posts
         ORDER BY RANDOM()
         LIMIT 1"
    )?;

    let post = stmt.query_row([], |row| {
        Ok(BlogPost {
            id: Some(row.get(0)?),
            timestamp: row.get::<_, String>(1)?.parse().unwrap_or_else(|_| Utc::now()),
            passion: row.get(2)?,
            location: row.get(3)?,
            activity: row.get(4)?,
            image_url: row.get(5)?,
        })
    })?;

    Ok(post)
}

// ===== HTTP API SERVER IMPLEMENTATION =====

type DbPool = Arc<Mutex<String>>; // Store the database path

// Handler to get latest blog posts
async fn handle_get_posts(
    limit: Option<usize>,
    db_path: DbPool,
) -> Result<impl Reply, Rejection> {
    let limit = limit.unwrap_or(10).min(100); // Default to 10, max 100
    let db_path = db_path.lock().await;

    // Open a new connection for this request
    let conn = Connection::open(db_path.as_str())
        .map_err(|_| warp::reject::custom(DatabaseError))?;

    let posts = get_latest_blog_posts(&conn, limit)
        .map_err(|_| warp::reject::custom(DatabaseError))?;

    Ok(warp::reply::json(&posts))
}

// Handler to get a specific blog post by ID
async fn handle_get_post(
    id: i64,
    db_path: DbPool,
) -> Result<impl Reply, Rejection> {
    let db_path = db_path.lock().await;

    let conn = Connection::open(db_path.as_str())
        .map_err(|_| warp::reject::custom(DatabaseError))?;

    let post = get_blog_post_by_id(&conn, id)
        .map_err(|_| warp::reject::custom(DatabaseError))?;

    Ok(warp::reply::json(&post))
}

// Handler to get a random blog post
async fn handle_get_random(
    db_path: DbPool,
) -> Result<impl Reply, Rejection> {
    let db_path = db_path.lock().await;

    let conn = Connection::open(db_path.as_str())
        .map_err(|_| warp::reject::custom(DatabaseError))?;

    let post = get_random_blog_post(&conn)
        .map_err(|_| warp::reject::custom(DatabaseError))?;

    Ok(warp::reply::json(&post))
}

// Custom error handler
async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(DatabaseError) = err.find() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Database Error";
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&serde_json::json!({
        "error": message,
        "status": code.as_u16(),
    }));

    Ok(warp::reply::with_status(json, code))
}

// Start the HTTP server
pub async fn start_api_server(db_path: String, port: u16) {
    let db_path = Arc::new(Mutex::new(db_path));

    // CORS configuration
    // CORS configuration - more permissive for development
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "Accept", "Authorization", "Origin"])
        .expose_headers(vec!["Content-Length", "Content-Type"])
        .max_age(3600);

    // GET /api/posts?limit=10
    let get_posts = warp::path!("api" / "posts")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map(|params: std::collections::HashMap<String, String>| {
            params.get("limit")
                .and_then(|l| l.parse::<usize>().ok())
        })
        .and(with_db(db_path.clone()))
        .and_then(handle_get_posts);

    // GET /api/posts/:id
    let get_post = warp::path!("api" / "posts" / i64)
        .and(warp::get())
        .and(with_db(db_path.clone()))
        .and_then(handle_get_post);

    // GET /api/posts/random
    let get_random = warp::path!("api" / "posts" / "random")
        .and(warp::get())
        .and(with_db(db_path.clone()))
        .and_then(handle_get_random);

    // Health check endpoint
    let health = warp::path!("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

    // Combine all routes
    let routes = get_posts
        .or(get_random)
        .or(get_post)
        .or(health)
        .recover(handle_rejection)
        .with(cors);

    println!("Starting API server on http://0.0.0.0:{}", port);
    println!("Endpoints:");
    println!("  GET /api/posts?limit=10 - Get latest posts");
    println!("  GET /api/posts/:id      - Get post by ID");
    println!("  GET /api/posts/random   - Get random post");
    println!("  GET /health            - Health check");

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

// Helper function to pass db_path to handlers
fn with_db(db_path: DbPool) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_path.clone())
}
