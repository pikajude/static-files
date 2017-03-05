use sodiumoxide::crypto::secretbox;
use std::collections::HashMap;
use rocket::request::FromRequest;
use std::fs::File;
use rand::Rng;
use std::path::Path;
use rocket::http::Cookies;
use std::io::Read;
use sodiumoxide::crypto::secretbox::{Nonce, Key};
use rocket::response::Response;
use rustc_serialize::base64;
use rocket::http::Cookie;
use rustc_serialize::json;
use std::io::Write;
use rand::thread_rng;
use rocket::response::Responder;
use rustc_serialize::base64::{FromBase64, ToBase64};
use state::LocalStorage;
use rocket::http::Status;
use rocket::request::Request;
use rocket::Outcome;

use pages;

static _KN: LocalStorage<(Key, Nonce)> = LocalStorage::new();

pub fn load_keys() {
    fn init_key() -> Vec<u8> {
        let mut v = [0u8; 56];
        thread_rng().fill_bytes(&mut v);
        v.to_vec()
    }

    fn read_key_file() -> Vec<u8> {
        let key_path = Path::new(env!("KEY_FILE"));
        if !key_path.exists() {
            let mut f = File::create(key_path).expect("Could not open key file");
            f.write_all(init_key().as_slice()).expect("Unable to write to key file");
        }

        let mut v = Vec::new();
        {
            let mut f = File::open(key_path).expect("Could not open key file");
            f.read_to_end(&mut v).expect("Unable to read key file");
        }
        v
    }

    fn bytes_to_pair(mut bytes: Vec<u8>) -> (Key, Nonce) {
        assert!(bytes.len() == 56, "secret keyfile wrong length");
        let mut key: &mut Vec<u8> = bytes.as_mut();
        let nonce: Vec<u8> = key.split_off(32);
        (Key::from_slice(key.as_slice()).expect("Key wrong length"),
         Nonce::from_slice(nonce.as_slice()).expect("Nonce wrong length"))
    }

    _KN.set(|| bytes_to_pair(read_key_file()));
}

#[derive(Debug)]
pub struct Session<'a> {
    _store: HashMap<String, Vec<u8>>,
    _cookiejar: &'a Cookies,
}

impl<'a> Session<'a> {
    pub fn new(cj: &'a Cookies) -> Session {
        Session {
            _store: HashMap::new(),
            _cookiejar: cj,
        }
    }

    pub fn get(&self, key: &'static str) -> Option<&Vec<u8>> {
        self._store.get(key)
    }

    pub fn get_string(&self, key: &'static str) -> Option<String> {
        self.get(key).and_then(|s| String::from_utf8(s.to_vec()).ok())
    }

    pub fn user(&self) -> Option<pages::User> {
      self.get_string("user").map(|u| pages::User(u))
    }

    pub fn insert<R>(&mut self, key: String, value: R) -> Option<Vec<u8>>
        where R: Into<Vec<u8>>
    {
        self._store.insert(key, value.into())
    }

    fn to_cookie(&self) -> String {
        let text = json::encode(&self._store).unwrap();
        let bytes = text.as_bytes();
        let &(ref k, ref n) = _KN.get();
        let ciphertext = secretbox::seal(bytes, n, k);
        ciphertext.as_slice().to_base64(base64::URL_SAFE)
    }

    fn from_cookie(cj: &'a Cookies, s: String) -> Option<Session<'a>> {
        let &(ref k, ref n) = _KN.get();
        s.from_base64()
            .ok()
            .and_then(|bytes| secretbox::open(&bytes, n, k).ok())
            .and_then(|plaintext| String::from_utf8(plaintext).ok())
            .and_then(|string| json::decode(string.as_str()).ok())
            .map(|store| {
                Session {
                    _store: store,
                    _cookiejar: cj,
                }
            })
    }
}

impl<'a> Drop for Session<'a> {
    fn drop(&mut self) {
        if self._store.len() > 0 {
            self._cookiejar.add(Cookie::new("_SESSION", self.to_cookie()));
        }
    }
}

impl<'r, 'a> FromRequest<'a, 'r> for Session<'a> {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, ()), ()> {
        let cookiejar = req.cookies();
        Outcome::Success(cookiejar.find("_SESSION")
            .and_then(|cookie| Session::from_cookie(cookiejar, String::from(cookie.value())))
            .unwrap_or(Session::new(cookiejar)))
    }
}
