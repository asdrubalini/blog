#![allow(dead_code)]

use anyhow::Ok;
use axum::{
    http::header::{ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
    routing::get,
    Router,
};
use posts::Posts;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

mod posts;

// const CURRENT_GIT_HASH: &str = include_str!("../.git/refs/heads/master");
const CURRENT_GIT_HASH: &str = "ciao";

mod routes {
    use crate::{posts::Posts, CURRENT_GIT_HASH};
    use axum::{
        extract::{Path, State},
        response::Html,
    };

    /// Index
    pub(crate) async fn root() -> Html<String> {
        Html(CURRENT_GIT_HASH.to_string())
    }

    /// /post/:slug
    pub(crate) async fn get_post(
        Path(slug): Path<String>,
        State(posts): State<Posts>,
    ) -> Html<String> {
        match posts.get(slug) {
            Some(post) => Html(post.inner_html.to_owned()),
            None => Html("404 post not found".to_string()),
        }
    }
}

async fn web() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    let posts = Posts::new()?;

    // Trace every request
    let trace_layer = TraceLayer::new_for_http();

    // Set up CORS
    let cors_layer = CorsLayer::new()
        .allow_headers([ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN])
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    let app = Router::new()
        .route("/", get(routes::root))
        .route("/post/:slug", get(routes::get_post))
        .layer(cors_layer)
        .layer(trace_layer)
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        .with_state(posts);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let j = tokio::task::spawn(async move { axum::serve(listener, app).await.unwrap() });
    println!("Started listening on {:?}", addr);
    j.await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    web().await?;

    Ok(())
}
