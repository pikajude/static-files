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
//!     use static_files::file::plain;
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

use quote::Tokens;
use state::LocalStorage;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

pub mod file;
pub mod web;
pub use web::*;

use file::StringTok;
use file::StaticFile;

pub type FileStorage = HashMap<String, Box<Fn() -> StaticFile + Send>>;

/// Top-level static storage of a map of filepaths to files.
///
/// When built in development mode, calling a function returned by a lookup in this
/// hashmap will load the file from disk.
///
/// When built in production mode, calling the function returns a reference to the file,
/// which is stored in the binary's __DATA section.
pub static FILES: LocalStorage<FileStorage> = LocalStorage::new();

/// This highly magical function loads all the `File`s given in the input
/// and places them in the top-level `FILES` static hashmap.
pub fn load_files(v: Vec<file::File>) {
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

      pub fn load_files_with(store: &::state::LocalStorage<::static_files::FileStorage>) {
        store.set(|| {
          let mut m: ::static_files::FileStorage = HashMap::new();
          #loaders
          m
        });
      }

      pub fn load_files() {
        load_files_with(&::static_files::FILES);
      }
    };

    f.write_all(tokens.to_string().as_bytes()).expect("Didn't write to file")
}
