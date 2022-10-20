use dryoc::pwhash::*;
use dryoc::Error;

pub fn hash_password(password: String) -> Result<String, Error> {
    // Generate a random salt
    let mut salt = Salt::default();
    salt.resize(dryoc::constants::CRYPTO_PWHASH_SALTBYTES, 0);
    dryoc::rng::copy_randombytes(&mut salt);

    // With customized configuration parameters, return type must be explicit
    let pwhash: VecPwHash = PwHash::hash_with_salt(
        &password.into_bytes(),
        salt,
        Config::interactive().with_opslimit(1).with_memlimit(8192),
    )?;

    Ok(pwhash.to_string())
}

pub fn verify_password(hash: String, password: String) -> Result<(), Error> {
    let pwhash = VecPwHash::from_string(hash.as_str())?;
    pwhash.verify(&password.into_bytes())
}
