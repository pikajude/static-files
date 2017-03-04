use pages::*;
use rocket::response::content::HTML;
use db::Entry;

pub fn page(es: Vec<Entry>) -> HTML<String> {
    default_layout(Page {
        title: None,
        body: html! {
      article.bubble.last-bubble {
        h5.site-title "I’m Jude, a functional programmer with a colorful head."
      }

      @for entry in es {
        article.bubble.preview-bubble {
          h3.post-preview {
            a.post-title href={ "/r/" (entry.slug) } (entry.title)
          }
        }
      }
    },
    })
}
