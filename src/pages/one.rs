use db::Entry;
use highlighting::markdown;
use pages::*;
use rocket::response::content::HTML;

pub fn page(user: Option<User>, entry: Entry) -> HTML<String> {
    let loggedin = user.is_some();
    default_layout(Page {
        title: None,
        user: user,
        body: html! {
      article.bubble.blog-post {
        h1.post-title {
            (entry.title)
            @if loggedin {
                " "
                a.edit-link.fa.fa-pencil href={ "/e/" (entry.slug) } {}
                " "
                form.delete-form method="post" action={ "/d/" (entry.slug) } {
                    input type="hidden" name="_method" value="DELETE" /
                    button.fa.fa-trash-o type="submit"
                        data-confirm="Are you sure you want to delete this post?" {}
                }
            }
        }
        (PreEscaped(markdown(entry.content)))
      }
    },
    })
}
