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
