use sodiumoxide::crypto::pwhash;

pub fn verify(pw: &String) -> bool {
    pwhash::pwhash_verify(&pwhash::HashedPassword(*include_bytes!("../secret.password")), pw.as_bytes())
}
