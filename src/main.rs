#![allow(dead_code)]

use axum::{
    body::Body,
    http::{
        header::{ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
        Request,
    },
    routing::get,
    Router,
};
use posts::Posts;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

mod posts;

mod routes {
    use crate::posts::Posts;
    use axum::{
        extract::{Path, State},
        response::Html,
    };
    use orgize::Org;
    use tokio::fs;

    /// Index
    pub(crate) async fn root() -> Html<String> {
        let s = fs::read_to_string("../posts/ciao.org").await.unwrap();
        let org = Org::parse(s);

        Html(org.to_html())
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    let posts = Posts::new().await?;

    // Trace every request
    let trace_layer =
        TraceLayer::new_for_http().on_request(|_: &Request<Body>, _: &tracing::Span| {
            tracing::info!(message = "begin request!")
        });

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

    // Serve using axum's server if compiled in debug mode
    #[cfg(debug_assertions)]
    {
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    // Serve with AWS's Lambda if compiled in release mode
    #[cfg(not(debug_assertions))]
    {
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(app);

        lambda_http::run(app).await.unwrap();
    }

    Ok(())
}
