use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize, Serializer};
use jsonwebtoken::{encode, Header, EncodingKey, Validation, DecodingKey, TokenData};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand_core::OsRng;
use serde::de::Error;
use crate::gen::auth::TokenType;

pub const SECRETSIZE: usize = 1024;
pub const TOKENSIZE: usize = 64;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaim {
    /// Time of Living
    pub(crate) exp: usize,
    /// Creation Time
    pub(crate) iat: usize,
    /// TokenType casted to i32
    pub(crate) token_type: i32,
    /// Subject identifier, i.e. <user IP/Client>
    pub(crate) sub: String,
    /// Token value, used as ID of token
    pub(crate) value: String,
    /// Users name
    pub(crate) user_id: String,
    ///// Creator of token
    //pub(crate) iss: String,
}

impl TokenClaim {
    /// Do not use for creating new tokens! \\
    /// Use for decoding only!
    pub fn new(
        exp: usize,
        iat: usize,
        token_type: i32,
        sub: String,
        value: String,
        //organisation_id: String,
        user_id: String
    )->Self {
        Self {
            exp,
            iat,
            token_type,
            sub,
            value,
            //organisation_id,
            user_id
        }
    }

    /// Create token with `exp=iat=0`.
    /// `iss` setted to `comsys`.
    /// Use for create new token only with `setup_time` after that.
    pub fn with_data(
        token_type: i32,
        sub: String,
        value: String,
        //organisation_id: String,
        user_id: String)->Self {
        Self {
            exp: 0,
            iat: 0,
            token_type,
            sub,
            value,
            //organisation_id,
            user_id
            //iss: "comsys".to_string(),
        }
    }

    pub fn random(
        token_type: i32,
        sub: String,
        //organisation_id: String,
        user_id: String
    )->Self {
        Self::with_data (
            token_type,
            sub,
            String::from_utf8(
                rand::thread_rng().sample_iter(&Alphanumeric).take(TOKENSIZE).collect::<Vec<u8>>()
            ).unwrap(),
            //organisation_id,
            user_id
        )
    }

    /// Setup lifetime for token
    pub fn setup_time(&mut self, ttl: usize, iat: usize) {
        self.exp = iat + ttl;
        self.iat = iat;
    }

    pub fn view_tuple(&self) -> (usize, usize, i32, String, String) {
        (
            self.exp,
            self.iat,
            self.token_type,
            self.sub.clone(),
            self.value.clone(),
        )
    }
}

pub struct TokenGenerator {
    encoding: EncodingKey,
    decoding: DecodingKey,
    access_token_ttl: usize,
    auth_token_ttl: usize,
}

impl TokenGenerator {
    pub fn new() -> Self {
        let mut s: [u8; SECRETSIZE] = [0u8; SECRETSIZE];
        //rand_core::RngCore::fill_bytes(&mut s);
        rand::thread_rng().fill_bytes(&mut s);
        Self {
            encoding: EncodingKey::from_secret(&s),
            decoding: DecodingKey::from_secret(&s),
            access_token_ttl: 2_678_400,
            auth_token_ttl: 14_400,
        }
    }

    pub fn with_secet(s: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(&s),
            decoding: DecodingKey::from_secret(&s),
            access_token_ttl: 2_678_400,
            auth_token_ttl: 14_400,
        }
    }

    pub fn get_acc_ttl(&self) -> usize {
        self.access_token_ttl
    }

    pub fn get_auth_ttl(&self) -> usize {
        self.auth_token_ttl
    }

    pub async fn generate(&self, meta: &mut TokenClaim) -> Result<(String, i32), ()> {
        match meta.token_type {
            x if x == TokenType::Auth as i32 => {
                //println!("ACHTUNG-1");
                meta.setup_time(self.auth_token_ttl, chrono::Utc::now().timestamp() as usize);
                if let Ok(token) = encode(&Header::default(), meta, &self.encoding) {
                    return Ok((token, x));
                } else {
                    return Err(());
                }
            },
            x if x == TokenType::Access as i32 => {
                //println!("ACHTUNG-2");
                meta.setup_time(self.access_token_ttl, chrono::Utc::now().timestamp() as usize);
                return if let Ok(token) = encode(&Header::default(), meta, &self.encoding) {
                    //println!("ACHTUNG-2");
                    Ok((token, x))
                } else {
                    //println!("ACHTUNG-2-2");
                    Err(())
                }
            },
            _ => {
                //println!("ACHTUNG-3");
                return Err(());
            }
        };
    }

    pub async fn decode(&self, author: String, token: String) -> Result<TokenData<TokenClaim>, ()> {
        /*let mut validation = Validation::default();//new(jsonwebtoken::Algorithm::HS256);
        validation.sub = Some(author);
        validation.set_required_spec_claims(&["exp", "sub"]);*/

        match jsonwebtoken::decode::<TokenClaim>(&token, &self.decoding, &Validation::default()) {
            Ok(t) => {
                Ok(t)
            }
            Err(_) => {
                Err(())
            }
        }
    }
}




#[tokio::test]
async fn token_gen_test() {
    let mut gen = crate::auth_backend::tokens::TokenGenerator::new();
    let mut claim = TokenClaim::random(1, "I am".to_string(), "Debug".to_string());
    println!("Claim: {:?}", claim);
    let code = gen.generate(&mut claim).await.unwrap();
    println!("Enc: {:?}", code);
    let dec = gen.decode("I am".to_string(), code.0.clone()).await.unwrap();
    println!("Dec: {:?}", dec)
}

#[tokio::test]
async fn argon_test() {
/*let password = b"69N2GRDB3CSE"; // Bad password; don't actually use!
    let salt = SaltString::generate(&mut OsRng);

// Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

// Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    println!("{}", password_hash);*/
}  