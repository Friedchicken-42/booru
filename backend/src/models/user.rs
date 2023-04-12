use std::num::NonZeroU32;

use ring::{
    digest,
    pbkdf2::{self, derive, verify},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

const ITERATIONS: u32 = 100_000;
const SALT_SIZE: usize = 64;
const CREDENTIAL_SIZE: usize = digest::SHA512_OUTPUT_LEN;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub name: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub salt: [u8; SALT_SIZE],
    #[serde_as(as = "serde_with::hex::Hex")]
    pub hash: [u8; CREDENTIAL_SIZE],
}

impl User {
    pub fn hash(name: String, password: String) -> Result<Self, Error> {
        let mut salt = [0u8; SALT_SIZE];
        let iterations = NonZeroU32::new(ITERATIONS).ok_or(Error::Hashing)?;

        let rng = SystemRandom::new();
        rng.fill(&mut salt).map_err(|_| Error::Hashing)?;

        let mut hash = [0u8; CREDENTIAL_SIZE];

        derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            iterations,
            &salt,
            password.as_bytes(),
            &mut hash,
        );

        Ok(Self {
            id: None,
            name,
            salt,
            hash,
        })
    }

    pub fn verify(&self, password: String) -> Result<(), Error> {
        let iterations = NonZeroU32::new(ITERATIONS).ok_or(Error::Hashing)?;

        verify(
            pbkdf2::PBKDF2_HMAC_SHA512,
            iterations,
            &self.salt,
            &password.as_bytes(),
            &self.hash,
        )
        .map_err(|_| Error::WrongCredential)
    }
}
