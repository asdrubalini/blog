use axum::{routing::get, Router};
use posts::Posts;

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
        let org = Org::parse(&s);

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
    let posts = Posts::new().await?;

    let app = Router::new()
        .route("/", get(routes::root))
        .route("/post/:slug", get(routes::get_post))
        .with_state(posts);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
