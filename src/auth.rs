use sodiumoxide::crypto::pwhash;

pub fn verify(pw: &String) -> bool {
    pwhash::pwhash_verify(&pwhash::HashedPassword(*include_bytes!("../secret.password")), pw.as_bytes())
}

pub fn gen_password(pw: &[u8]) {
    println!("{:?}", pwhash::pwhash(pw, pwhash::OPSLIMIT_INTERACTIVE, pwhash::MEMLIMIT_INTERACTIVE).unwrap());
}
