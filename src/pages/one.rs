use pages::*;
use rocket::response::content::HTML;
use db::Entry;
use highlighting::markdown;

pub fn page(entry: Entry) -> HTML<String> {
    default_layout(Page {
        title: None,
        user: None,
        body: html! {
      article.bubble.blog-post {
        h1.post-title (entry.title)
        (PreEscaped(markdown(entry.content)))
      }
    },
    })
}
