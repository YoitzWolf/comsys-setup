use std::error::Error;
use std::ops::Index;

use clerk::xslx_conv::prelude::ConvertablePackage;
use comp::{EasyParticipant, Team};
use comp_handler::eq_message;

use crate::grpc::comp::participant::Gender;
use crate::grpc::comp::{CompDeclaration, NominationDeclaration};

use self::comp::JudgeScheme;

pub mod auth;
pub mod auth_client_interceptor;
pub mod comp;
pub mod comp_handler;
pub mod generic;
pub mod users;

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
                x if (x == 'f') || (x == 'w') || (x == 'ж') || (x == 'д') => Self::Female,
                x if (x == 'm') || (x == 'м') => Self::Male,
                _ => Self::Unknown,
            }
        } else {
            Self::Unknown
        }
    }
}

impl Gender {
    pub fn into_rus_flag(&self) -> &'static str {
        match self {
            Gender::Unknown => "?",
            Gender::Male => "м",
            Gender::Female => "ж",
        }
    }
}

impl eq_message::Message {
    pub fn represent(&self) -> &'static str {
        match self {
            eq_message::Message::VoteMessage(_) => &"Оценка",
            eq_message::Message::VerifyMessage(_) => &"Подтверждение оценки",
            eq_message::Message::FixVoting(_) => &"Закрытие голосования за участника",
            eq_message::Message::TryNext(_) => &"Переход к следеющему участнику",
            eq_message::Message::Block(_) => &"Перевод соревнования в режим ожидания",
            eq_message::Message::SetActiveAction(_) => &"Обновление активного выступления",
            eq_message::Message::ClearQueueAction(_) => &"Очистка активного выступления",
            eq_message::Message::FinesSetup(_) => &"Сбавки",
        }
    }
}

impl users::role::Role {
    pub fn weight(&self) -> i32 {
        match self {
            users::role::Role::Moderator(_) => 4,
            users::role::Role::Secretary(_) => 3,
            users::role::Role::Arbitor(_) => 2,
            users::role::Role::Judge(_) => 1,
            users::role::Role::Watcher(_) => 0,
        }
    }
}

impl JudgeScheme {
    pub fn get_judgement_group_name(&self, quid: i32) -> Result<String, ()> {
        match quid {
            0 => Ok("Исполнение".to_string()),    // 4 or 6
            1 => Ok("Артистичность".to_string()), // 4 or 6
            2 => Ok("Сложность".to_string()),     // 2
            _ => Err(()),
        }
        // На самом деле лучше делать так, но у нас группы по составу одинаковые, различаются только размеры
        /*match self {
            JudgeScheme::FourFourTwo => {
                match quid {
                    1 => Ok("".to_string()),
                    2 => Ok("".to_string()),
                    3 => Ok("".to_string()),
                    _ => Err(())
                }
            },
            JudgeScheme::SixSixTwo => {
                match quid {
                    1 => Ok("".to_string()),
                    2 => Ok("".to_string()),
                    3 => Ok("".to_string()),
                    _ => Err(())
                }
            },
        }*/
    }

    pub fn get_judgement_group_id(&self, quid: String) -> Result<i32, ()> {
        match quid.as_str() {
            "Исполнение" => Ok(0),    // 4 or 6 max times
            "Артистичность" => Ok(1), // 4 or 6 max times
            "Сложность" => Ok(2),     // 2
            _ => Err(()),
        }
    }

    pub fn get_vec(&self) -> Vec<Vec<i32>> {
        match self {
            JudgeScheme::FourFourTwo => {
                vec![vec![0; 4], vec![0; 4], vec![0; 2]]
            }
            JudgeScheme::SixSixTwo => {
                vec![vec![0; 6], vec![0; 6], vec![0; 2]]
            }
            JudgeScheme::FourFourOne => {
                vec![vec![0; 4], vec![0; 4], vec![0; 1]]
            }
        }
    }

    pub fn sizes(&self) -> Vec<i32> {
        match self {
            JudgeScheme::FourFourTwo => {
                vec![4, 4, 2]
            }
            JudgeScheme::SixSixTwo => {
                vec![6, 6, 2]
            }
            JudgeScheme::FourFourOne => {
                vec![4, 4, 1]
            }
        }
    }
}

impl From<(std::string::String, Vec<Team>)> for NominationDeclaration {
    fn from(value: (std::string::String, Vec<Team>)) -> Self {
        Self {
            title: value.0.clone(),
            inner_queue: value.1.iter().map(|x| x.tid.clone()).collect(),
            teams: value.1.iter().map(|x| (x.tid.clone(), {let mut x = x.clone(); x.nom=value.0.clone(); x})).collect(),
        }
    }
}

impl TryFrom<(i32, Vec<String>)> for Team {
    type Error = Box<dyn Error>;

    fn try_from(value: (i32, Vec<String>)) -> Result<Self, Self::Error> {
        Ok(Team {
            tid: value.0,
            //nom: todo!(),
            participants: value
                .1
                .index(0)
                .split(',')
                .map(|x| EasyParticipant {
                    name: x.to_string(),
                    extra_personal: value.1.iter().skip(2).cloned().collect(),
                })
                .collect(),
            organisation: value.1.index(1).clone(),
            nom: "None".to_string()
        })
    }
}

impl ConvertablePackage for NominationDeclaration {
    type Item = Team;
}
