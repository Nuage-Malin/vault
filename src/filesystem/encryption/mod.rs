use std::error::Error;
use crate::my_eprintln;

use super::MyError;

use std::io::{Read, Write};

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

pub struct FileEncryption {
}

impl FileEncryption {

    /// user id is encryption (public) key
    pub fn encrypt(input: &[u8], key: &str) -> Result<Vec<u8>> {

        // let key = age::x25519::Identity::generate();
        // let pubkey = key.to_public();
        // todo get key from password hash generated (not the hash stored in users DB)

        let encrypted = {
            let encryptor = age::Encryptor::with_user_passphrase(age::secrecy::Secret::new(key.to_owned()));

            let mut encrypted = vec![];
            match encryptor.wrap_output(&mut encrypted) {
            Ok (mut writer) => {
                match writer.write_all(input) {
                    Ok(_) => {
                        match writer.finish() {
                            Ok(_) => {
                                encrypted
                            }
                            Err(err) => {
                                return Err(Box::new(MyError::new(&err.to_string())));
                            }
                        }
                    }
                    Err(err) => {
                        return Err(Box::new(MyError::new(&err.to_string())));
                    }
                }
            }
            Err(err) => {
                return Err(Box::new(MyError::new(&err.to_string())));
            }
        }
        };
        return Ok(encrypted);
    }

    pub fn decrypt(input: &[u8], key: &str) -> Result<Vec<u8>> {
        // let key = age::x25519::Identity::generate();
        let decrypted = {
            let decryptor = match age::Decryptor::new(&input[..]).ok() {
                Some(age::Decryptor::Passphrase(d)) => d,
                _ => unreachable!(),
            };

            let mut decrypted = vec![];

            match decryptor.decrypt(&age::secrecy::Secret::new(key.to_owned()), None) {
                Ok(mut reader) => {
                    match reader.read_to_end(&mut decrypted) {
                        Ok(_read_size) => {

                        }
                        Err(r) => {
                            my_eprintln!("Could not read from encrypted file: {}", r.to_string());
                        }
                    }

                    decrypted
                }
                Err(err) => {
                    return Err(Box::new(MyError::new(&err.to_string())));
                }
            }
        };

        return Ok(decrypted);
    }
}
