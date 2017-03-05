use sodiumoxide::crypto::secretbox;
use std::collections::HashMap;
use rocket::request::FromRequest;
use sodiumoxide::crypto::secretbox::{Nonce, Key};
use rocket::response::Response;
use rustc_serialize::base64;
use rustc_serialize::json;
use rocket::response::Responder;
use rustc_serialize::base64::{FromBase64, ToBase64};
use state::LocalStorage;
use rocket::http::Status;
use rocket::request::Request;
use rocket::Outcome;

static _KN: LocalStorage<(Key, Nonce)> = LocalStorage::new();

pub fn load_keys() {
    _KN.set(|| (secretbox::gen_key(), secretbox::gen_nonce()) );
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Session {
    _store: HashMap<String, Vec<u8>>
}

impl Session {
    pub fn new() -> Session {
        Session { _store: HashMap::new() }
    }

    pub fn to_cookie(&self) -> String {
        let text = json::encode(self).unwrap();
        let bytes = text.as_bytes();
        let &(ref k, ref n) = _KN.get();
        let ciphertext = secretbox::seal(bytes, n, k);
        ciphertext.as_slice().to_base64(base64::URL_SAFE)
    }

    pub fn from_cookie(s: String) -> Option<Session> {
        let &(ref k, ref n) = _KN.get();
        s.from_base64().ok().and_then(|bytes| {
            secretbox::open(&bytes, n, k).ok()
        }).and_then(|plaintext| {
            String::from_utf8(plaintext).ok()
        }).and_then(|string| json::decode(string.as_str()).ok())
    }
}

impl<'r, 'a> FromRequest<'a, 'r> for Session {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, ()), ()> {
        Outcome::Success(req.cookies().find("_SESSION").and_then(|cookie| {
            Session::from_cookie(String::from(cookie.value()))
        }).unwrap_or(Session::new()))
    }
}
