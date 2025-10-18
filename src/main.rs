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
use rusqlite::{params, Connection, OptionalExtension};

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
    let media_root = &state.config.get_string("media_root").unwrap();

    // Set variable only if video_id is present
    let video_id = params
    .video_id
    .unwrap_or(-1);

    let template = state.env.get_template("home").unwrap();

    let (video_path, mut suggestions) = pick_movie_and_thumbnail(&state.config, video_id);

    for s in &mut suggestions {
        s.path = s.path.replace(media_root, "");
    }

    let rendered = template
        .render(context! {video_path => video_path.replace(media_root, ""), suggestions => suggestions})
        .unwrap();

    Ok(Html(rendered))
}

fn pick_movie_and_thumbnail(config: &Config, video_id: i32) -> (String, Vec<VideoSuggestion>) {
    // open DB (config key "media_db" or fallback to ./media.db)
    let db_path = config.get_string("media_db").unwrap_or_else(|_| "media.db".to_string());
    let conn = Connection::open(db_path).expect("failed to open media db");

    // collect distinct types
    let mut stmt = conn.prepare("SELECT DISTINCT type FROM media").expect("prepare");
    let types: Vec<i32> = stmt
        .query_map([], |r| r.get(0))
        .expect("query_map")
        .map(|r| r.expect("row"))
        .collect();

    let n_total = config.get_int("n_video_suggestions").unwrap() as usize;
    let n_types = types.len().max(1);
    let base = n_total / n_types;
    let mut remainder = n_total % n_types;

    let mut suggestions: Vec<VideoSuggestion> = Vec::new();

    // for each type select base (+1 if remainder) random rows
    for t in types {
        let mut limit = base;
        if remainder > 0 {
            limit += 1;
            remainder -= 1;
        }
        if limit == 0 {
            continue;
        }

        let mut s = conn
            .prepare("SELECT id, thumbnail FROM media WHERE type = ?1 ORDER BY RANDOM() LIMIT ?2")
            .expect("prepare select");
        let rows = s
            .query_map(params![t, limit as i64], |r| {
                Ok(VideoSuggestion {
                    path: r.get(1)?,
                    video_id: r.get::<_, i64>(0)? as usize,
                })
            })
            .expect("query_map");

        for r in rows { 
            suggestions.push(r.expect("row"));
        }
    }

    // choose main video: use provided id if valid, otherwise random row
    let video_path = if video_id >= 0 {
        conn.query_row(
            "SELECT path FROM media WHERE id = ?1",
            params![video_id],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .expect("query_row optional")
        .unwrap_or_else(|| {
            conn.query_row(
                "SELECT path FROM media ORDER BY RANDOM() LIMIT 1",
                [],
                |r| r.get(0),
            )
            .expect("random fallback")
        })
    } else {
        conn.query_row(
            "SELECT path FROM media ORDER BY RANDOM() LIMIT 1",
            [],
            |r| r.get(0),
        )
        .expect("random select")
    };

    (video_path, suggestions)
}