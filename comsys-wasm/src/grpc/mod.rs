use crate::grpc::comp::{CompDeclaration};
use crate::grpc::comp::participant::{Gender};

pub mod auth;
pub mod comp;
pub mod comp_handler;
pub mod generic;
pub mod auth_client_interceptor;


impl CompDeclaration {
    pub fn is_valid(&self) -> bool {
        self.title.len() > 0 &&
            //self.nominations.len() > 0 &&
            self.dates.is_some()
    }
}


impl From<String> for Gender {
    fn from(value: String) -> Self {
        if let Some(x) = value.trim().to_lowercase().chars().next() {
            match x {
                x if (x == 'f') || (x == 'w') || (x=='ж') || (x=='д') => Self::Female,
                x if (x == 'm') || (x=='м') => Self::Male,
                _ => Self::Unknown
            }
        } else {
            Self::Unknown
        }
    }
}

impl Gender {
    pub fn into_rus_flag(&self) -> &'static str{
        match self {
            Gender::Unknown => "?",
            Gender::Male => "м",
            Gender::Female => "ж",
        }
    }
}