extern crate generator;

use generator::*;
use generator::File::*;
use std::env;
use std::fs::File;
use std::io::Error;

fn create() -> Result<(), Error> {
    let out = env::var("OUT_DIR").unwrap();
    let mut f = File::create(out + "/generated.rs")?;

    generate_file(vec![Plain("Cargo.toml", "Cargo.toml"),
                       Plain("img/github.png", "static/img/github.png"),
                       Sass("css/all.css", "static/css/all.scss")],
                  &mut f)
}

fn main() {
    create().unwrap();
}
