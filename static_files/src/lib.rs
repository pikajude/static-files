//! `static_files` is a library to facilitate development with and serving of various
//! types of static files from your Rocket server.
//!
//! In most cases, you'll want your website to serve unminified, unmodified assets
//! (such as CSS and JavaScript) in the development environment to facilitate debugging,
//! but serve minified and compressed assets in production. I found no easy way to do
//! this within `rocket` itself and thus wrote this crate.
//!
//! In development, your assets will be reloaded on every request, which includes
//! re-running them through the sass or coffeescript compilers, etc.
//!
//! When building your app in production mode, these assets will be loaded during the
//! compile process and embedded in the binary as bytestrings.
//!
//! # Setup
//!
//! To run arbitrary code at compile-time, we use a Cargo build script. Put this in a
//! `build.rs` file in the top level of your project:
//!
//!     extern crate static_files;
//!
//!     use static_files::load_files;
//!     use static_files::plain;
//!
//!     fn main() {
//!         load_files(vec![plain("Cargo.toml", "Cargo.toml")])
//!     }
//!
//! The first argument to `plain` refers to the path at which this file will be stored
//! in the hashmap. The second argument is the actual path on disk. In this case, they're
//! the same.
//!
//! # Loading the files
//!
//! The snippet above will generate a module called `files.rs` and place it in cargo's
//! $OUT_DIR.
//!
//! Somewhere in your `main` function, add this code:
//!
//!     mod_path! my_files { concat!(env!("OUT_DIR"), "/files.rs") }
//!     my_files::load_files();
//!
//! The `mod_path!` macro is currently required due to a limitation in rustc, but will
//! hopefully eventually no longer be necessary.
//!
//! This code snippet loads the files from disk (or simply registers their names, paths,
//! and types) and populates the top-level `FILES` hashmap.
//!
//! # Serving files
//!
//! In whatever route you want, use this code:
//!
//!     #[get("/static/<path..>")]
//!     fn get(path: PathBuf, inm: Option<IfNoneMatch>) -> Option<Cached<StaticResponse>> {
//!         web::lookup_file(path, inm)
//!     }
//!
//! Note that you do need to call `load_files` first; otherwise, this code will complain
//! that the FILES variable hasn't been initialized.

#![feature(const_fn)]
#![recursion_limit = "128"]

extern crate state;
extern crate rocket;
extern crate crypto;

#[macro_use]
extern crate quote;

use crypto::md5::Md5;
use quote::Tokens;
use rocket::http::ContentType;
use state::LocalStorage;
use std::collections::HashMap;
use crypto::digest::Digest;
use std::path::PathBuf;
use std::process::Command;
use std::io::{Write, Read};
use std::process::Stdio;
use std::fs;

#[derive(Debug)]
pub enum ByteString {
    Static(&'static [u8]),
    Dynamic(Vec<u8>),
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        match self {
            &ByteString::Static(s) => s,
            &ByteString::Dynamic(ref v) => v.as_ref(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum File {
    Plain(&'static str, &'static str),
    Sass(&'static str, &'static str, Vec<String>),
}

impl File {
    /// Does no transformations.
    pub fn plain(a: &'static str, b: &'static str) -> Self {
      File::Plain(a, b)
    }

    /// Runs `sassc` with no special arguments.
    pub fn sass(a: &'static str, b: &'static str) -> Self {
      Self::sass_args::<&str>(a, b, vec![])
    }

    /// Runs `sassc`, adding the provided arguments.
    pub fn sass_args<T>(a: &'static str, b: &'static str, args: Vec<T>) -> Self where T: Into<String>{
      File::Sass(a, b, args.into_iter().map(|x|x.into()).collect())
    }
}

/// # For internal use
impl File {
    /// Used by the generated module.
    pub fn filename(self) -> String {
        use File::*;
        String::from(match self {
            Plain(s, _) => s,
            Sass(s, _, _) => s,
        })
    }

    /// Load the file from disk and apply transformations.
    ///
    /// You should never need to use this function, but it needs to be exported
    /// in order for the generated module to use it.
    pub fn resolve(self) -> StaticFile {
        match self {
            File::Plain(f, p) => Self::load_plain(String::from(f), p),
            File::Sass(f, p, args) => Self::load_sass(String::from(f), p, args),
        }
    }

    fn load_plain(name: String, pb: &'static str) -> StaticFile {
        #[cfg(not(debug_assertions))]
        println!("cargo:warning=Loading plain file: {:?}", pb);
        let mut s = Vec::new();

        fs::File::open(PathBuf::from(pb)).expect(format!("{:?} doesn't exist", pb).as_str())
          .read_to_end(&mut s).expect(format!("{:?} couldn't be read", pb).as_str());

        let n1 = name.clone();
        let s1 = s.clone();
        StaticFile {
            name: name,
            bytes: ByteString::Dynamic(s),
            mime: ContentType::from_extension(PathBuf::from(n1)
                .extension()
                .unwrap()
                .to_str()
                .unwrap()),
            etag: Self::mk_etag(s1),
        }
    }

    fn load_sass(name: String, pb: &'static str, args: Vec<String>) -> StaticFile {
        #![allow(unused_mut)]
        println!("cargo:warning=Loading sass file: {:?}", pb);

        let mut child = Command::new("sass")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&args)
            .spawn()
            .expect("failed to execute process");

        let mut contents = String::new();
        contents.push_str("$static_prefix: '/s/';\n");
        fs::File::open(pb)
            .expect("Couldn't open file")
            .read_to_string(&mut contents)
            .expect("read from file failed");

        child.stdin
            .as_mut()
            .expect("Stdin not mut")
            .write_all(contents.as_bytes())
            .expect("Write failed");

        let output = child.wait_with_output().expect("Child didn't wait");

        if output.status.success() {
            let o1 = output.stdout.clone();
            StaticFile {
                name: name,
                bytes: ByteString::Dynamic(output.stdout),
                mime: ContentType::CSS,
                etag: Self::mk_etag(o1),
            }
        } else {
            panic!("sass failed: {}",
                   String::from_utf8_lossy(output.stderr.as_slice()))
        }
    }

    fn mk_etag<T>(s: T) -> String
        where T: AsRef<[u8]>
    {
        let mut m = Md5::new();
        m.input(s.as_ref());
        let suffix: String = m.result_str().chars().take(16).collect();
        format!("W/\"{}\"", suffix)
    }
}

struct StringTok<'a>(pub &'a String);

impl<'a> quote::ToTokens for StringTok<'a> {
  fn to_tokens(&self, tokens: &mut Tokens) {
    tokens.append("String::from(");
    self.0.to_tokens(tokens);
    tokens.append(")");
  }
}

impl quote::ToTokens for File {
  fn to_tokens(&self, tokens: &mut Tokens) {
    match self {
      &File::Sass(ref p, ref f, ref args) => {
        let arg_strings: Vec<StringTok> = args.iter().map(|x|StringTok(x)).collect();
        tokens.append("::static_files::File::Sass(");
        p.to_tokens(tokens);
        tokens.append(",");
        f.to_tokens(tokens);
        tokens.append(", vec!");
        arg_strings.to_tokens(tokens);
        tokens.append(")");
      },
      &File::Plain(p, f) => {
        tokens.append("::static_files::File::Plain(");
        p.to_tokens(tokens);
        tokens.append(",");
        f.to_tokens(tokens);
        tokens.append(")");
      }
    }
  }
}

impl quote::ToTokens for ByteString {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let vec = match self {
            &ByteString::Static(u) => u.to_vec(),
            &ByteString::Dynamic(ref v) => v.clone(),
        };

        let mut s = String::new();
        s.push_str("::static_files::ByteString::Static(b\"");
        for b in vec {
            s.push_str(format!("\\x{:02x}", b).as_str())
        }
        s.push_str("\")");
        tokens.append(s)
    }
}

pub struct StaticFile {
    pub name: String,
    pub bytes: ByteString,
    pub mime: ContentType,
    pub etag: String,
}

impl quote::ToTokens for StaticFile {
  fn to_tokens(&self, tokens: &mut Tokens) {
    tokens.append("::static_files::StaticFile { name: ");
    StringTok(&self.name).to_tokens(tokens);
    tokens.append(", bytes: ");
    self.bytes.to_tokens(tokens);
    tokens.append(format!(", mime: ::rocket::http::ContentType::new({:?}, {:?})", self.mime.ttype.to_string(), self.mime.subtype.to_string()));
    tokens.append(", etag: ");
    StringTok(&self.etag).to_tokens(tokens);
    tokens.append(" }");
  }
}

pub mod web;
pub use web::*;

/// Top-level static storage of a map of filepaths to files.
///
/// When built in development mode, calling a function returned by a lookup in this
/// hashmap will load the file from disk.
///
/// When built in production mode, calling the function returns a reference to the file,
/// which is stored in the binary's __DATA section.
pub static FILES: LocalStorage<HashMap<String, Box<Fn() -> StaticFile + Send>>> =
    LocalStorage::new();

/// This highly magical function loads all the `File`s given in the input
/// and places them in the top-level `FILES` static hashmap.
pub fn load_files(v: Vec<File>) {
    let out = ::std::env::var("OUT_DIR").unwrap();
    let mut f = fs::File::create(out + "/generated.rs").expect("File not created");
    let mut loaders = Tokens::new();

    #[cfg(debug_assertions)]
    {
      for static_file in v {
        let fname = static_file.clone().filename();
        let name = StringTok(&fname);
        loaders.append(quote! {
          m.insert(#name, Box::new(|| #static_file.resolve()));
        })
      }
    }

    #[cfg(not(debug_assertions))]
    {
      for static_file in v {
        let file = static_file.resolve();
        let fname = file.name.clone();
        let name = StringTok(&fname);
        loaders.append(quote! {
          m.insert(#name, Box::new(|| #file));
        });
      }
    }

    let tokens = quote! {
      use std::collections::HashMap;

      pub fn load_files() {
        ::static_files::FILES.set(|| {
          let mut m: HashMap<String, Box<Fn() -> ::static_files::StaticFile + Send>> = HashMap::new();
          #loaders
          m
        });
      }
    };

    f.write_all(tokens.to_string().as_bytes()).expect("Didn't write to file")
}
