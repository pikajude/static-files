use std::process::Command;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use fs;
use std::io::Read;
use quote::Tokens;
use crypto::md5::Md5;
use quote;
use rocket::http::ContentType;
use crypto::digest::Digest;

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

#[doc(hidden)]
pub struct StringTok<'a>(pub &'a String);

impl<'a> quote::ToTokens for StringTok<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append("String::from(");
        self.0.to_tokens(tokens);
        tokens.append(")");
    }
}

impl quote::ToTokens for ByteString {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let vec = match self {
            &ByteString::Static(u) => u.to_vec(),
            &ByteString::Dynamic(ref v) => v.clone(),
        };

        let mut s = String::new();
        s.push_str("::static_files::file::ByteString::Static(b\"");
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
        tokens.append("::static_files::file::StaticFile { name: ");
        StringTok(&self.name).to_tokens(tokens);
        tokens.append(", bytes: ");
        self.bytes.to_tokens(tokens);
        tokens.append(format!(", mime: ::rocket::http::ContentType::new({:?}, {:?})",
                              self.mime.ttype.to_string(),
                              self.mime.subtype.to_string()));
        tokens.append(", etag: ");
        StringTok(&self.etag).to_tokens(tokens);
        tokens.append(" }");
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum File {
    Plain(&'static str, &'static str),
    Sass(&'static str, &'static str, Vec<String>),
}

/// Does no transformations.
pub fn plain(a: &'static str, b: &'static str) -> File {
    File::Plain(a, b)
}

/// Runs `sassc` with no special arguments.
pub fn sass(a: &'static str, b: &'static str) -> File {
    sass_args::<&str>(a, b, vec![])
}

/// Runs `sassc`, adding the provided arguments.
pub fn sass_args<T>(a: &'static str, b: &'static str, args: Vec<T>) -> File
    where T: Into<String>
{
    File::Sass(a, b, args.into_iter().map(|x| x.into()).collect())
}

/// # For internal use
impl File {
    /// Used by the generated module.
    pub fn filename(self) -> String {
        use self::File::*;
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

        fs::File::open(PathBuf::from(pb))
            .expect(format!("{:?} doesn't exist", pb).as_str())
            .read_to_end(&mut s)
            .expect(format!("{:?} couldn't be read", pb).as_str());

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
        let suffix: String = m.result_str()
            .chars()
            .take(16)
            .collect();
        format!("W/\"{}\"", suffix)
    }
}

impl quote::ToTokens for File {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match self {
            &File::Sass(ref p, ref f, ref args) => {
                let arg_strings: Vec<StringTok> = args.iter().map(|x| StringTok(x)).collect();
                tokens.append("::static_files::file::File::Sass(");
                p.to_tokens(tokens);
                tokens.append(",");
                f.to_tokens(tokens);
                tokens.append(", vec!");
                arg_strings.to_tokens(tokens);
                tokens.append(")");
            }
            &File::Plain(p, f) => {
                tokens.append("::static_files::file::File::Plain(");
                p.to_tokens(tokens);
                tokens.append(",");
                f.to_tokens(tokens);
                tokens.append(")");
            }
        }
    }
}
