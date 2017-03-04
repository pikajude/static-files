#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate generator;

use std::collections::HashMap;
use generator::{FILES, StaticFile, File};
use std::path::PathBuf;
use generator::ByteString;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

use rocket::response::Responder;
use rocket::request::FromRequest;
use rocket::response::Response;
use rocket::request::Request;
use rocket::outcome::Outcome;
use rocket::http::Status;
use std::io::Cursor;

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

fn main() {
    load_files();
    rocket::ignite().mount("/", routes![get_static]).launch()
}
