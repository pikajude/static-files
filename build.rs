extern crate static_files;

use static_files::load_files;
use static_files::File;

fn main() {
    #![allow(unused_mut)]
    let mut args = vec!["--scss",
                       "-Istatic/css",
                       "-Ibower_components/foundation-sites/scss",
                       "-Ibower_components/font-awesome/scss"];

    #[cfg(not(debug_assertions))]
    args.append(&mut vec!["--style", "compact"]);

    load_files(vec![Plain("favicon.ico", "static/img/favicon.ico"),
                    File::plain("img/github.png", "static/img/github.png"),
                    File::plain("img/github@2x.png", "static/img/github@2x.png"),
                    File::plain("img/linkedin.png", "static/img/linkedin.png"),
                    File::plain("img/linkedin@2x.png", "static/img/linkedin@2x.png"),
                    File::plain("img/otter.png", "static/img/otter.png"),
                    File::plain("img/otter@2x.png", "static/img/otter@2x.png"),
                    File::plain("img/newpost.png", "static/img/newpost.png"),
                    File::plain("img/newpost@2x.png", "static/img/newpost@2x.png"),
                    File::plain("fonts/fontawesome-webfont.woff2", "bower_components/font-awesome/fonts/fontawesome-webfont.woff2"),
                    File::plain("fonts/fontawesome-webfont.woff", "bower_components/font-awesome/fonts/fontawesome-webfont.woff"),
                    File::plain("fonts/fontawesome-webfont.ttf", "bower_components/font-awesome/fonts/fontawesome-webfont.ttf"),
                    File::sass_args("css/all.css", "static/css/all.scss", args)
                    ])
}
