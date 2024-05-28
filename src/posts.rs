use std::{path::Path, sync::Arc};

use chrono::NaiveDate;
use include_directory::{include_directory, Dir};
use indexmap::IndexMap;
use org::org_date_parse;
use orgize::Org;

#[derive(Debug, PartialEq, Eq)]
pub struct Post {
    pub slug: String,
    pub title: Option<String>,
    pub date: Option<NaiveDate>,

    pub inner_html: String,
}

impl Ord for Post {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

/// Sort by date
impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
    pub fn parse(path: impl AsRef<Path>, data: &'static str) -> anyhow::Result<Post> {
        let org = Org::parse(data);

        let slug = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .replace(".org", "");
        let title = org::org_keyword_get(&org, "title");
        let date = org::org_keyword_get(&org, "date")
            .map(org_date_parse)
            .and_then(|date| date.ok());
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

/// Posts are loaded into the binary
static POSTS: Dir<'_> = include_directory!("$CARGO_MANIFEST_DIR/posts");

impl Posts {
    pub fn new() -> anyhow::Result<Self> {
        let posts = Posts::posts_read()?;
        let posts = IndexMap::from_iter(posts.into_iter().map(|p| (p.slug.to_owned(), p)));
        let posts = Arc::new(posts);

        Ok(Posts { posts })
    }

    pub fn get(&self, slug: impl AsRef<str>) -> Option<&Post> {
        self.posts.get(slug.as_ref())
    }

    fn posts_read() -> anyhow::Result<Vec<Post>> {
        let mut posts: Vec<_> = POSTS
            .files()
            .filter(|f| f.path().extension().and_then(|s| s.to_str()) == Some("org"))
            .filter_map(|f| Post::parse(f.path(), f.contents_utf8().unwrap()).ok())
            .collect();

        // sort them by date
        posts.sort();
        Ok(posts)
    }
}
