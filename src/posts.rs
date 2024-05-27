use std::{path::Path, sync::Arc};

use chrono::NaiveDate;
use futures::future::join_all;
use indexmap::IndexMap;
use org::org_date_parse;
use orgize::Org;
use tokio::{fs, task};

#[derive(Debug, Ord, Eq)]
pub struct Post {
    pub slug: String,
    pub title: Option<String>,
    pub date: Option<NaiveDate>,

    pub inner_html: String,
}

/// Sort by date
impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

/// Sort by date
impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

mod org {
    use chrono::NaiveDate;
    use orgize::Org;

    pub(super) fn org_keyword_get(org: &Org, name: impl AsRef<str>) -> Option<String> {
        let name = name.as_ref();

        org.keywords()
            .filter(|kw| kw.key().eq_ignore_ascii_case(name))
            .fold(Option::<String>::None, |acc, cur| {
                let mut s = acc.unwrap_or_default();

                if !s.is_empty() {
                    s.push(' ');
                }

                s.push_str(cur.value().trim());

                Some(s)
            })
    }

    const ORG_DATE_FMT: &str = "<%Y-%m-%d %a>";

    pub(super) fn org_date_parse(date: impl AsRef<str>) -> anyhow::Result<NaiveDate> {
        Ok(NaiveDate::parse_from_str(date.as_ref(), ORG_DATE_FMT)?)
    }

    pub(super) fn org_date_format(date: NaiveDate) -> String {
        date.format(ORG_DATE_FMT).to_string()
    }
}

impl Post {
    pub async fn post_load_from_fs(path: impl AsRef<Path>) -> anyhow::Result<Post> {
        let path = path.as_ref();
        let s = fs::read_to_string(path).await?;
        let org = Org::parse(&s);

        let slug = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .replace(".org", "");
        let title = org::org_keyword_get(&org, "title");
        let date = org::org_keyword_get(&org, "date")
            .map(org_date_parse)
            .map(|date| date.ok())
            .flatten();
        let html = org.to_html();

        Ok(Post {
            slug,
            title,
            date,
            inner_html: html,
        })
    }

    pub fn org_date(&self) -> Option<String> {
        self.date.map(org::org_date_format)
    }
}

#[derive(Clone)]
pub struct Posts {
    posts: Arc<IndexMap<String, Post>>,
}

const POSTS_PATH: &str = "./posts";

impl Posts {
    pub async fn new() -> anyhow::Result<Self> {
        let posts = Posts::posts_read(POSTS_PATH).await?;
        let posts = IndexMap::from_iter(posts.into_iter().map(|p| (p.slug.to_owned(), p)));
        let posts = Arc::new(posts);

        Ok(Posts { posts })
    }

    pub fn get(&self, slug: impl AsRef<str>) -> Option<&Post> {
        self.posts.get(slug.as_ref())
    }

    async fn posts_read(path: impl AsRef<Path>) -> anyhow::Result<Vec<Post>> {
        let mut entries = fs::read_dir(path).await.unwrap();
        let mut futures = Vec::new();

        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("org") {
                continue;
            }

            let fut = task::spawn(async move { Post::post_load_from_fs(path).await });
            futures.push(fut);
        }

        let mut posts = join_all(futures)
            .await
            .into_iter()
            .map(|res| res.unwrap().unwrap())
            .collect::<Vec<_>>();

        // sort them by date
        posts.sort();
        Ok(posts)
    }
}
