use pages;
use rocket::response::content::HTML;

pub fn page(error: Option<(String, &'static str)>) -> HTML<String> {
    let username = error.clone().map(|x|x.0).unwrap_or(String::new());
    let errmsg = error.map(|x|x.1);
    pages::default_layout(pages::Page {
        title: Some(String::from("Log in")),
        user: None,
        body: html! {
      article.bubble {
        h3.form-title "Log in"
        form role="form" method="post" action="/in" {
          div.row {
            div class="large-6 columns" {
                div.form-group {
                    label for="username" {
                        "Username"
                        input type="text" name="username" value=(username) /
                    }
                }
            }

            div class="large-6 columns" {
                div class={
                    "form-group"
                    @if errmsg.is_some() {
                        " error"
                    }
                } {
                    @if let Some(s) = errmsg {
                        label.is-invalid-label for="password" {
                            "Password"
                            input.is-invalid-input type="password" name="password" /
                            span.form-error.is-visible (s.clone())
                        }
                    } @else {
                        label for="password" {
                            "Password"
                            input type="password" name="password" /
                        }
                    }
                }
            }
          }
          button.button.small type="submit" "Try it"
        }
      }
    },
    })
}
