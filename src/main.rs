#![allow(dead_code)]
#![allow(unused_imports)]

use std::fmt::Debug;

use anyhow::Ok;
use askama_axum::Response;
use axum::{
    extract::State,
    http::{
        header::{ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
        Request, StatusCode,
    },
    middleware,
    routing::get,
    Router,
};
use posts::Posts;
use state::AppState;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer},
};
use tracing::info;

mod posts;

mod templates {
    use askama_axum::Template;

    const TITLE_PREFIX: &str = "Asdrubalini's Blog";

    #[derive(Debug, Clone)]
    pub struct Partials {
        id: String,
    }

    #[derive(Template)]
    #[template(path = "index.html")]
    pub struct IndexTemplate {
        pub title: String,
        pub name: String,
    }
}

mod routes {
    use crate::{posts::Posts, state::AppState, templates::IndexTemplate};
    use axum::{
        extract::{Path, State},
        response::Html,
    };

    /// Index
    pub(crate) async fn root() -> IndexTemplate {
        let html = IndexTemplate {
            title: "Asdrubalini's Blog".to_string(),
            name: "Asdrubalini!".to_string(),
        };

        html
    }

    /// /post/:slug
    pub(crate) async fn get_post(
        Path(slug): Path<String>,
        State(state): State<AppState>,
    ) -> Html<String> {
        match state.posts.get(slug) {
            Some(post) => Html(post.inner_html.to_owned()),
            None => Html("404 post not found".to_string()),
        }
    }
}

mod state {
    use anyhow::Ok;
    use axum::extract::FromRef;

    use crate::posts::Posts;

    #[derive(Clone)]
    pub(crate) struct AppState {
        pub posts: Posts,
    }

    impl AppState {
        pub(crate) fn new() -> anyhow::Result<Self> {
            let state = AppState {
                posts: Posts::new()?,
            };

            Ok(state)
        }
    }

    impl FromRef<AppState> for Posts {
        fn from_ref(app_state: &AppState) -> Self {
            app_state.posts.clone()
        }
    }
}

async fn web() -> anyhow::Result<()> {
    // TODO: adjust logging
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    let state = AppState::new()?;

    // Trace every request
    let trace_layer = TraceLayer::new_for_http();

    // Set up CORS
    let cors_layer = CorsLayer::new()
        .allow_headers([ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN])
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    let app = Router::new()
        .nest(
            "/static",
            axum_static::static_router("static").with_state(()),
        )
        .route("/", get(routes::root))
        .route("/post/:slug", get(routes::get_post))
        .layer(cors_layer)
        .layer(trace_layer)
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let handle = tokio::task::spawn(async move { axum::serve(listener, app).await.unwrap() });
    println!("Started listening on {:?}", addr);
    handle.await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    web().await?;

    Ok(())
}
