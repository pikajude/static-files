extern crate static_files;

use static_files::load_files;
use static_files::file::*;

fn main() {
    #![allow(unused_mut)]
    let mut args = vec!["--scss",
                       "-Istatic/css",
                       "-Ibower_components/foundation-sites/scss",
                       "-Ibower_components/font-awesome/scss"];

    #[cfg(not(debug_assertions))]
    args.append(&mut vec!["--style", "compact"]);

    load_files(vec![plain("favicon.ico", "static/img/favicon.ico"),
                    plain("img/github.png", "static/img/github.png"),
                    plain("img/github@2x.png", "static/img/github@2x.png"),
                    plain("img/linkedin.png", "static/img/linkedin.png"),
                    plain("img/linkedin@2x.png", "static/img/linkedin@2x.png"),
                    plain("img/otter.png", "static/img/otter.png"),
                    plain("img/otter@2x.png", "static/img/otter@2x.png"),
                    plain("img/newpost.png", "static/img/newpost.png"),
                    plain("img/newpost@2x.png", "static/img/newpost@2x.png"),
                    plain("fonts/fontawesome-webfont.woff2", "bower_components/font-awesome/fonts/fontawesome-webfont.woff2"),
                    plain("fonts/fontawesome-webfont.woff", "bower_components/font-awesome/fonts/fontawesome-webfont.woff"),
                    plain("fonts/fontawesome-webfont.ttf", "bower_components/font-awesome/fonts/fontawesome-webfont.ttf"),
                    sass_args("css/all.css", "static/css/all.scss", args)
                    ])
}
