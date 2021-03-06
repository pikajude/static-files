#![feature(const_fn)]
#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![plugin(maud_macros)]
#![plugin(mod_path)]
#![allow(non_snake_case)]

extern crate chrono;
extern crate rand;
extern crate static_files;
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

use static_files::{lookup_file, StaticResponse, Cached, IfNoneMatch};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use rocket::request::Form;
use rocket::State;
use rocket::response::content::HTML;
use std::path::PathBuf;

mod pages;
mod db;
mod highlighting;
mod session;
mod auth;

use db::Entry;
use db::Pool;
use session::Session;

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

#[get("/favicon.ico")]
fn get_favicon(inm: Option<IfNoneMatch>) -> Option<Cached<StaticResponse>> {
    get_static(PathBuf::from("favicon.ico"), inm)
}

#[get("/s/<path..>")]
fn get_static(path: PathBuf, inm: Option<IfNoneMatch>) -> Option<Cached<StaticResponse>> {
  lookup_file(path, inm)
}

#[get("/s/<path..>?<_query>")]
fn static_qs(path: PathBuf,
             _query: &str,
             inm: Option<IfNoneMatch>)
             -> Option<Cached<StaticResponse>> {
    get_static(path, inm)
}

#[get("/in")]
fn login(s: Session) -> Result<HTML<String>, rocket::response::Redirect> {
    if let Some(_) = s.get("user") {
        return Err(rocket::response::Redirect::to("/"));
    }

    Ok(pages::login::page(None))
}

#[derive(FromForm)]
struct Password { username: String, password: String }

#[post("/in", data = "<password>")]
fn post_login(mut s: Session, password: Form<Password>) -> Result<HTML<String>, rocket::response::Redirect> {
    let pw = password.get();
    if auth::verify(&pw.password) {
        s.set_user(pages::User(pw.username.clone()));
        return Err(rocket::response::Redirect::to("/"));
    }

    Ok(pages::login::page(Some((pw.username.clone(), "Invalid password"))))
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
        .mount("/", routes![home, get_favicon, get_static, static_qs, one, login, post_login])
        .launch()
}
