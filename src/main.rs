#![feature(const_fn)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![plugin(maud_macros)]
#![plugin(mod_path)]

extern crate chrono;
extern crate rand;
extern crate generator;
extern crate maud;
extern crate postgres;
extern crate pulldown_cmark;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rocket;
extern crate rustc_serialize;
extern crate sodiumoxide;
extern crate state;
extern crate syntect;

use generator::{FILES, StaticFile};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use rocket::State;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::response::Response;
use rocket::response::content::HTML;
use std::io::Cursor;
use std::path::PathBuf;

mod pages;
mod db;
mod highlighting;
mod session;

use db::Entry;
use db::Pool;
use session::Session;

struct IfNoneMatch(String);
struct StaticResponse(StaticFile);
enum Cached<T> {
    Cached,
    Uncached(T),
}

impl<'r> Responder<'r> for StaticResponse {
    fn respond(self) -> Result<Response<'r>, Status> {
        Response::build()
            .header(self.0.mime)
            .raw_header("Etag", self.0.etag)
            .sized_body(Cursor::new(self.0.bytes))
            .ok()
    }
}

impl<'r, T> Responder<'r> for Cached<T>
    where T: Responder<'r>
{
    fn respond(self) -> Result<Response<'r>, Status> {
        match self {
            Cached::Cached => Response::build().status(Status::NotModified).ok(),
            Cached::Uncached(t) => t.respond(),
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for IfNoneMatch {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, ()), Self::Error> {
        match request.headers().get_one("If-None-Match") {
            Some(inm) => Outcome::Success(IfNoneMatch(String::from(inm))),
            None => Outcome::Forward(()),
        }
    }
}

#[get("/")]
fn home(db: State<Pool>, s: Session) -> HTML<String> {
    let conn = db.get().unwrap();
    let user = s.user();
    let entries: Vec<Entry> = conn.query("SELECT * FROM essay ORDER BY created_at DESC", &[])
        .unwrap()
        .iter()
        .map(|r| Entry::from_row(r))
        .collect();

    pages::home::page(user, entries)
}

#[get("/r/<slug>")]
fn one(db: State<Pool>, slug: String, s: Session) -> Option<HTML<String>> {
    let conn = db.get().unwrap();
    let rows = conn.query("SELECT * FROM essay WHERE slug = $1", &[&slug]).unwrap();
    match rows.len() {
        1 => Some(pages::one::page(s.user(), Entry::from_row(rows.get(0)))),
        _ => None,
    }
}

#[get("/s/<path..>")]
fn get_static(path: PathBuf, inm: Option<IfNoneMatch>) -> Option<Cached<StaticResponse>> {
    FILES.get().get(&String::from(path.to_str().unwrap())).map(|x| {
        let sf = x();

        if let Some(IfNoneMatch(ref i)) = inm {
            if &sf.etag == i {
                return Cached::Cached;
            }
        }

        Cached::Uncached(StaticResponse(sf))
    })
}

#[get("/s/<path..>?<_query>")]
fn static_qs(path: PathBuf,
             _query: &str,
             inm: Option<IfNoneMatch>)
             -> Option<Cached<StaticResponse>> {
    get_static(path, inm)
}

#[get("/login")]
fn login(s: Session) -> Result<HTML<String>, rocket::response::Redirect> {
    if let Some(_) = s.get("user") {
        return Err(rocket::response::Redirect::to("/"));
    }

    Ok(pages::default_layout(pages::Page {
        title: Some(String::from("Log in")),
        user: None,
        body: html! {
      article.bubble {
        h3.form-title "Log in"
        form role="form" method="post" action="/login" {
          div.row {
            div class="large-6 columns" {
              "Username field"
            }

            div class="large-6 columns" {
              "Password field"
            }
          }
          button.button.tiny type="submit" "Try it"
        }
      }
    },
    }))
}

fn main() {
    mod_path! generated { concat!(env!("OUT_DIR"), "/generated.rs") }
    generated::load_files();

    session::load_keys();

    let username = env!("PGUSER");
    let manager = PostgresConnectionManager::new(format!("postgres://{}@localhost", username),
                                                 TlsMode::None)
        .unwrap();
    let config = r2d2::Config::default();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    rocket::ignite()
        .manage(pool)
        .mount("/", routes![home, get_static, static_qs, one, login])
        .launch()
}
