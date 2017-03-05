use maud::PreEscaped;
use maud::DOCTYPE;
use rocket::response::content::HTML;

pub mod home;
pub mod one;

pub struct User(pub String);

pub struct Page {
    pub title: Option<String>,
    pub body: PreEscaped<String>,
    pub user: Option<User>,
}

pub fn default_layout(page: Page) -> HTML<String> {
    HTML(html! {
    (DOCTYPE)
    html lang="en" {
      head {
        meta charset="UTF-8" /
        meta http-equiv="X-UA-Compatible" content="IE=edge" /
        meta name="viewport" content="width=device-width,initial-scale=1" /

        link rel="shortcut icon" href="/s/favicon.ico" /

        link rel="stylesheet" href="/s/css/all.css" type="text/css" /

        (PreEscaped("<!--[if lt IE 9]>"))
        (PreEscaped("<script src=\"http://html5shiv.googlecode.com/svn/trunk/html5.js\"></script>"))
        (PreEscaped("<![endif]-->"))

        title {
          "jude.bio"
          @if let Some(t) = page.title {
            " » "
            (t)
          }
        }
      }
      body {
        div.row role="main" {
          div class="speech large-12 columns" {
            header {
              a#head href="/" "jude.bio"
              span.arrow {}
              div#dots {
                span.up-arrow {}
                a.dot#github href="https://github.com/pikajude" data-tipsy?
                  title="I'm on GitHub!" "I'm on GitHub!"
                a.dot#linkedin href="http://www.linkedin.com/in/pikajude" data-tipsy?
                  title="I'm on LinkedIn!" "I'm on LinkedIn!"
                @if let Some(_) = page.user {
                  a.dot#new-post href="/n" title="Make a new post" "New post"
                }
              }
            }
            (page.body)
            footer {
              "Talk to me: "
              a href="mailto:me@jude.bio" "me@jude.bio"
              "."
            }
          }
        }
      }
    }
  }
        .into_string())
}
