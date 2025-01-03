use std::fmt::Debug;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AccessType<T: Clone+Debug+Serialize> {
    All,
    List(Vec<T>)
}

impl<T: Clone+Debug+Serialize+PartialEq> PartialEq for AccessType<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<T: Clone+Debug+Serialize+PartialEq> AccessType<T> {
    pub fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            (AccessType::All, AccessType::All) => true,
            (AccessType::All, AccessType::List(_)) => true,
            (AccessType::List(_), AccessType::All) => false,
            (AccessType::List(sf), AccessType::List(ot)) => {
                ot.iter().all(|t| { sf.contains(t) })
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Permissions {
    /// Watch only current Private copms
    Watch(AccessType<i32>),
    /// Ability to create new comps
    Create,
    /// Moderate comps. i.e. change them, setup judges etc.
    Moderator(AccessType<i32>),
    /// Administrator of Organisation. Don't need if you pointed as owner.
    Administrate,
    /// Judge(competition, queue, mark group type)
    Judge(i32, i32, i32),
    /// Secretary(competition)
    Secretary(i32),
    /// Arbitor(competition, queue)
    Arbitor(i32, i32),
}

impl Permissions {

    pub fn collect(dat: &Vec<Self>) -> serde_json::Result<String> {
        serde_json::to_string(dat)
    }

    pub fn parse(dat: &str) -> serde_json::Result<Self> {
        serde_json::from_str(dat)
    }

    pub fn parse_arr(dat: &str) -> serde_json::Result<Vec<Self>> {
        serde_json::from_str(dat)
    }
}

impl ToString for Permissions {
    fn to_string(&self) -> String {
        match self {
            Permissions::Watch(_) => "Watch",
            Permissions::Create => "Create",
            Permissions::Moderator(_) => "Moderator",
            Permissions::Administrate => "Administrate",
            Permissions::Judge(_, _, _) => "Judge",
            Permissions::Secretary(_) => "Secreatary",
            Permissions::Arbitor(_, _) => "Arbitor",
        }.into()
    }
}

impl PartialEq for Permissions {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Watch(l0), Self::Watch(r0)) => l0 == r0,
            (Self::Moderator(l0), Self::Moderator(r0)) => l0 == r0,
            (Self::Judge(l0, l1, l2), Self::Judge(r0, r1, r2)) => l0 == r0 && l1 == r1 && l2 == r2,
            (Self::Secretary(l0), Self::Secretary(r0)) => l0 == r0,
            (Self::Arbitor(l0, l1), Self::Arbitor(r0, r1)) => l0 == r0 && l1==r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaim {
    /// Time of Living
    pub(crate) exp: i64,
    /// Creation Time
    pub(crate) iat: i64,
    /// TokenType casted to i32
    pub(crate) token_type: i32,
    /// Subject identifier, i.e. <user IP/Client>
    pub(crate) sub: String,
    /// Token value, used as ID of token
    pub(crate) value: String,
    /// Users name
    pub(crate) user_id: i32,
    /// 
    pub perms: Vec<(i32, Vec<Permissions>)>,
}

impl TokenClaim {
    /// Do not use for creating new tokens! \\
    /// Use for decoding only!
    pub fn new(
        exp: i64,
        iat: i64,
        token_type: i32,
        sub: String,
        value: String,
        user_id: i32,
        perms: Vec<(i32, Vec<Permissions>)>
    )->Self {
        Self {
            exp,
            iat,
            token_type,
            sub,
            value,
            //organisation_id,
            user_id,
            perms
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
        user_id: i32,
        perms: Vec<(i32, Vec<Permissions>)>
    )->Self {
        Self {
            exp: 0,
            iat: 0,
            token_type,
            sub,
            value,
            user_id,
            perms
        }
    }

    pub fn random(
        token_type: i32,
        sub: String,
        //organisation_id: String,
        user_id: i32,
        perms: Vec<(i32, Vec<Permissions>)>
    )->Self {
        Self::with_data (
            token_type,
            sub,
            String::from_utf8(
                rand::thread_rng().sample_iter(&Alphanumeric).take(TOKENSIZE).collect::<Vec<u8>>()
            ).unwrap(),
            //organisation_id,
            user_id,
            perms
        )
    }

    /// Setup lifetime for token
    pub fn setup_time(&mut self, ttl: i64, iat: i64) {
        self.exp = iat + ttl;
        self.iat = iat;
    }

    pub fn view_tuple(&self) -> (i64, i64, i32, String, String) {
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
    access_token_ttl: i64,
    auth_token_ttl: i64,
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

    pub fn get_acc_ttl(&self) -> i64 {
        self.access_token_ttl
    }

    pub fn get_auth_ttl(&self) -> i64 {
        self.auth_token_ttl
    }

    pub async fn generate(&self, meta: &mut TokenClaim) -> Result<(String, i32), ()> {
        match meta.token_type {
            x if x == TokenType::Auth as i32 => {
                //println!("ACHTUNG-1");
                meta.setup_time(self.auth_token_ttl, chrono::Utc::now().timestamp());
                return if let Ok(token) = encode(&Header::default(), meta, &self.encoding) {
                    Ok((token, x))
                } else {
                    Err(())
                }
            },
            x if x == TokenType::Access as i32 => {
                //println!("ACHTUNG-2");
                meta.setup_time(self.access_token_ttl, chrono::Utc::now().timestamp());
                return match encode(&Header::default(), meta, &self.encoding) {
                    Ok(token) => {
                        //println!("ACHTUNG-2");
                        Ok((token, x))
                    }
                    Err(..) => {
                        //println!("ACHTUNG-2-2");
                        Err(())
                    }
                };
            },
            _ => {
                //println!("ACHTUNG-3");
                return Err(());
            }
        };
    }

    pub async fn generate_as_preset(&self, meta: &mut TokenClaim) -> Result<(String, i32), ()> {
        match meta.token_type {
            x if x == TokenType::Auth as i32 => {
                return if let Ok(token) = encode(&Header::default(), meta, &self.encoding) {
                    Ok((token, x))
                } else {
                    Err(())
                }
            },
            x if x == TokenType::Access as i32 => {
                return match encode(&Header::default(), meta, &self.encoding) {
                    Ok(token) => {
                        Ok((token, x))
                    }
                    Err(..) => {
                        Err(())
                    }
                };
            },
            _ => {
                //println!("ACHTUNG-3");
                return Err(());
            }
        };
    }


    ///
    /// `author: String` - author in meaning of `trait GetAuthor::get_author`
    /// `token: String` - token string to decode
    pub async fn decode(&self, author: String, token: String) -> Result<TokenData<TokenClaim>, ()> {
        /*let mut validation = Validation::default();//new(jsonwebtoken::Algorithm::HS256);
        validation.sub = Some(author);
        validation.set_required_spec_claims(&["exp", "sub"]);*/
        let mut validation = Validation::default();
        validation.sub = Some(author);
        match jsonwebtoken::decode::<TokenClaim>(&token, &self.decoding, &validation) {
            Ok(t) => {
                //println!("Token OK!");
                Ok(t)
            }
            Err(_e) => {
                //println!("Token Err {:?}!", e);
                Err(())
            }
        }
    }
}




#[tokio::test]
async fn token_gen_test() {
    let mut gen = crate::auth_backend::tokens::TokenGenerator::new();
    let mut claim = TokenClaim::with_data(
        1,
        "I am".to_string(),
        "value".to_string(),
        0,
        vec![(0, vec![Permissions::Administrate])]
    );
    claim.exp=1815978530;
    let mut claim2 = claim.clone();
    claim2.perms = vec![];
    println!("Claim: {:?}", claim);
    println!("Claim2: {:?}", claim2);
    let code  = gen.generate_as_preset(&mut claim).await.unwrap();
    let code2 = gen.generate_as_preset(&mut claim2).await.unwrap();
    println!("Enc: {:?}",  code);
    println!("Enc2: {:?}", code2);
    let dec = gen.decode("I am".to_string(), code.0.clone()).await.unwrap();
    let dec2 = gen.decode("I am".to_string(), code2.0.clone()).await.unwrap();
    println!("Dec: {:?}", dec);
    println!("Dec2: {:?}", dec2);
}

#[tokio::test]
async fn argon_test() {
    let password = b"69N2GRDB3CSE"; // Bad password; don't actually use!
    let salt = SaltString::generate(&mut OsRng);

// Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

// Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    println!("{}", password_hash);
}  