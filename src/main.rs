use axum::{response::Html, routing::get, Router};
use axum::http::StatusCode;
use axum::extract::{State, Query};
use tower_http::{
    services::ServeDir,
};
use config::Config;
use minijinja::{context, Environment};
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use rand::Rng;

#[derive(Deserialize, Debug)]
struct MediaEntry {
    movie: String,
    thumbnail: String,
}

struct AppState {
    env: Environment<'static>,
    config: Config
}

#[derive(serde::Serialize)]
struct VideoSuggestion {
    path: String,
    video_id: usize
}

#[derive(Debug, Deserialize)]
struct VideoQuery {
    video_id: Option<i32>,
}

#[tokio::main]
async fn main() {
    let mut env = Environment::new();
    env.add_template("home", include_str!("../templates/home.jinja")).unwrap();

    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();

    let media_root = config.get_string("media_root").unwrap();

    println!("media root is {}", media_root);

    // pass env to handlers via state
    let app_state = Arc::new(AppState { env, config });

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .nest_service("/video", ServeDir::new(media_root))
        .with_state(app_state);

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(state): State<Arc<AppState>>, Query(params): Query<VideoQuery>) -> Result<Html<String>, StatusCode> {
    // Set variable only if video_id is present
    let video_id = params
    .video_id
    .unwrap_or(-1);

    let template = state.env.get_template("home").unwrap();

    let (video_path, suggestions) = pick_movie_and_thumbnail(&state.config, video_id);

    let rendered = template
        .render(context! {video_path => video_path, suggestions => suggestions})
        .unwrap();

    Ok(Html(rendered))
}

fn pick_movie_and_thumbnail(config: &Config, video_id: i32) -> (String, Vec<VideoSuggestion>) {
    let json = fs::read_to_string("media.json").expect("Failed to read file");
    let mut entries: Vec<MediaEntry> = serde_json::from_str(&json).expect("Invalid JSON");
    let media_root = config.get_string("media_root").unwrap();

    let n_video_suggestions = config.get_int("n_video_suggestions").unwrap();

    let mut rng = rand::rng();

    let mut video_idx: i32 = video_id;
    if video_idx < 0 || video_idx > entries.len() as _ {
        video_idx = rng.random_range(0..entries.len()) as i32;
    }

    let video_path = entries.remove(video_idx as usize).movie.replace(&media_root, "");
    let mut suggestions: Vec<VideoSuggestion> = Vec::new();

    for _ in 0 .. n_video_suggestions {
        let id = rng.random_range(0..entries.len()) as _;
        let suggestion = VideoSuggestion { path : entries.remove(id).thumbnail.replace(&media_root, ""), video_id: id };
        suggestions.push(suggestion);
    }
    (video_path, suggestions)
}