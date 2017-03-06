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
                       Plain("img/github@2x.png", "static/img/github@2x.png"),
                       Plain("img/linkedin.png", "static/img/linkedin.png"),
                       Plain("img/linkedin@2x.png", "static/img/linkedin@2x.png"),
                       Plain("img/otter.png", "static/img/otter.png"),
                       Plain("img/otter@2x.png", "static/img/otter@2x.png"),
                       Plain("img/newpost.png", "static/img/newpost.png"),
                       Plain("img/newpost@2x.png", "static/img/newpost@2x.png"),
                       Plain("fonts/fontawesome-webfont.woff2", "bower_components/font-awesome/fonts/fontawesome-webfont.woff2"),
                       Plain("fonts/fontawesome-webfont.woff", "bower_components/font-awesome/fonts/fontawesome-webfont.woff"),
                       Plain("fonts/fontawesome-webfont.ttf", "bower_components/font-awesome/fonts/fontawesome-webfont.ttf"),
                       Sass("css/all.css", "static/css/all.scss")],
                  &mut f)
}

fn main() {
    create().unwrap();
}
