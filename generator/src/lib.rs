#![feature(const_fn)]

extern crate state;
extern crate rocket;
extern crate crypto;

use crypto::md5::Md5;
use std::fmt::Display;
use rocket::http::ContentType;
use state::LocalStorage;
use std::collections::HashMap;
use std::io::Error;
use crypto::digest::Digest;
use std::path::PathBuf;
use std::process::Command;
use std::io::{Write, Read};
use std::ffi::OsStr;
use std::process::Stdio;
use std::fs;

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
    Sass(&'static str, &'static str),
}

impl File {
    pub fn filename(self) -> String {
        use File::*;
        String::from(match self {
            Plain(s, _) => s,
            Sass(s, _) => s,
        })
    }

    pub fn resolve(self) -> StaticFile {
        match self {
            File::Plain(f, p) => Self::load_plain(String::from(f), p),
            File::Sass(f, p) => Self::load_sass(String::from(f), p),
        }
    }

    fn load_plain(name: String, pb: &'static str) -> StaticFile {
        #[cfg(not(debug_assertions))]
        println!("cargo:warning=Loading plain file: {:?}", pb);
        let mut s = Vec::new();

        fs::File::open(PathBuf::from(pb)).unwrap().read_to_end(&mut s).unwrap();

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

    fn load_sass(name: String, pb: &'static str) -> StaticFile {
        #![allow(unused_mut)]
        println!("cargo:warning=Loading sass file: {:?}", pb);

        let mut args = vec!["--scss",
                            "-Istatic/css",
                            "-Ibower_components/foundation-sites/scss",
                            "-Ibower_components/font-awesome/scss"];

        #[cfg(not(debug_assertions))]
        args.append(&mut vec!["--style", "compact"]);

        let mut contents = String::new();
        contents.push_str("$static_prefix: '/s/';\n");
        fs::File::open(pb)
            .expect("Couldn't open file")
            .read_to_string(&mut contents)
            .expect("read from file failed");

        let bytes = run_subcommand("sass", &args, contents.as_bytes());

        #[cfg(not(debug_assertions))]
        let bytes = run_subcommand("cssnano", &[], bytes.as_slice());

        let b1 = bytes.clone();

        StaticFile {
            name: name,
            bytes: ByteString::Dynamic(bytes),
            mime: ContentType::CSS,
            etag: Self::mk_etag(b1),
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

    fn as_literal(self) -> String {
        use File::*;
        match self {
            Sass(p, f) => format!("File::Sass({:?}, {:?})", p, f),
            Plain(p, f) => format!("File::Plain({:?}, {:?})", p, f),
        }
    }
}

fn run_subcommand<S: AsRef<OsStr> + Display + Copy>(cmd: S, args: &[S], stdin: &[u8]) -> Vec<u8> {
    let mut child = Command::new(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()
        .expect("failed to execute process");

    child.stdin.as_mut().expect("Stdin not mut").write_all(stdin).expect("Write failed");

    let output = child.wait_with_output().expect("Child didn't wait");
    if output.status.success() {
        output.stdout
    } else {
        panic!("{} failed: {}", cmd, String::from_utf8_lossy(output.stderr.as_slice()))
    }
}

impl ByteString {
    fn as_literal(self) -> String {
        let vec = match self {
            ByteString::Static(u) => u.to_vec(),
            ByteString::Dynamic(v) => v,
        };

        let mut s = String::new();
        s.push_str("ByteString::Static(b\"");
        for b in vec {
            s.push_str(format!("\\x{:02x}", b).as_str())
        }
        s.push_str("\")");
        s
    }
}

pub struct StaticFile {
    pub name: String,
    pub bytes: ByteString,
    pub mime: ContentType,
    pub etag: String,
}

impl StaticFile {
    pub fn as_literal(self) -> String {
        format!("StaticFile {{ name: String::from({:?}), bytes: {}, mime: ContentType::new({:?}, \
                 {:?}), etag: String::from({:?}) }}",
                self.name,
                self.bytes.as_literal(),
                self.mime.ttype.to_string(),
                self.mime.subtype.to_string(),
                self.etag)
    }
}

pub static FILES: LocalStorage<HashMap<String, Box<Fn() -> StaticFile + Send>>> =
    LocalStorage::new();

pub fn generate_file(v: Vec<File>, f: &mut fs::File) -> Result<(), Error> {
    f.write_all(b"
    #![allow(unused_imports)]
    use generator::FILES;
    use std::collections::HashMap;
    use generator::StaticFile;
    use generator::File;
    use rocket::http::ContentType;
    use generator::ByteString;

    pub fn load_files() {
      FILES.set(|| {
        let mut m: HashMap<String, Box<Fn() -> StaticFile + Send>> = HashMap::new();

  ")?;

  #[cfg(debug_assertions)]
    {
        for sf in v {
            let name = sf.clone().filename();
            f.write_all(format!("
            m.insert(String::from({:?}), Box::new(|| \
                                    {}.resolve()));
      ",
                                   name,
                                   sf.as_literal())
                    .as_bytes())?;
        }
    }

  #[cfg(not(debug_assertions))]
    {
        for sf in v {
            let file = sf.resolve();
            let n = file.name.clone();
            f.write_all(format!("
            m.insert(String::from({:?}), Box::new(|| {}));
      \
                                    ",
                                   n,
                                   file.as_literal())
                    .as_bytes())?;
        }
    }

    f.write_all(b"
          m
      });
    }
  ")
}
