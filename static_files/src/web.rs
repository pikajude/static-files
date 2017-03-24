use StaticFile;
use FILES;
use rocket::response::Responder;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::outcome::Outcome;
use std::path::PathBuf;
use rocket::response::Response;
use rocket::http::Status;
use std::io::Cursor;

pub struct StaticResponse(pub StaticFile);

impl<'r> Responder<'r> for StaticResponse {
    fn respond(self) -> Result<Response<'r>, Status> {
        Response::build()
            .header(self.0.mime)
            .raw_header("Etag", self.0.etag)
            .sized_body(Cursor::new(self.0.bytes))
            .ok()
    }
}

/// A wrapper for the `If-None-Match` HTTP header.
pub struct IfNoneMatch(pub String);
pub enum Cached<T> {
    Cached,
    Uncached(T),
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

/// Look up and possibly serve the given entry in the FILES map.
///
/// If the file is present, but its `etag` matches the value of the `inm` argument,
/// `Cached::Cached` will be returned. Note that this only happens in production, as
/// computing etags in dev wastes time.
pub fn lookup_file(path: PathBuf, inm: Option<IfNoneMatch>) -> Option<Cached<StaticResponse>> {
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
