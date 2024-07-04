use crate::{posts::Posts, state::AppState, templates::IndexTemplate};
use axum::{
    extract::{Path, State},
    response::Html,
};

/// Index
pub(crate) async fn root() -> IndexTemplate {
    IndexTemplate {
        title: "Asdrubalini's Blog".to_string(),
        name: "Asdrubalini!".to_string(),
    }
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
