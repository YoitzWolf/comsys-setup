use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ops::Index;
use std::time::SystemTime;
use chrono::{Days, NaiveDate, TimeZone, Utc};
use prost_wkt_types::Timestamp;
use crate::grpc::comp::{NominationDeclaration, Participant, Team};
use crate::grpc::comp::participant::{Gender};

impl TryFrom<(Vec<String>, Vec<String>)> for Participant {
    type Error = Box<dyn Error>;
    // Тут жестко задана структура первых четырех столбцов. А вот остальные столбцы могут быть любые. 
    fn try_from((head, data): (Vec<String>, Vec<String>)) -> Result<Self, Self::Error> {
        let h0 :String = head.index(0).to_lowercase().trim().to_string();
        let h1 :String = head.index(1).to_lowercase().trim().to_string();
        let h2 :String = head.index(2).to_lowercase().trim().to_string();
        let h3 :String = head.index(3).to_lowercase().trim().to_string();
        if (
            h0.eq(&"id") || h0.eq(&"номер участника")
        ) && (
            h1.eq(&"фио") || h1.eq(&"имя") || h1.eq(&"name")
        ) && (
            h2.eq(&"пол") || h2.eq(&"гендер") || h2.eq(&"sex") || h2.eq(&"gender")
        ) && (
            h3.eq(&"дата рождения") || h3.eq(&"день рождения") || h3.eq(&"birthdate") || h3.eq(&"birthday")
        ) {
            Ok(
                Self {
                    uid: data.index(0).parse::<i32>()?,
                    name: data.index(1).clone(),
                    gender: Gender::from(data.index(2).clone()).into(),
                    birthdate: Some(
                        {
                            let n = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(); // Это из-за устройства EXCEL
                            let n = n.checked_add_days(Days::new(data.index(3).clone().parse::<u64>()?)).unwrap();
                            let sys: SystemTime = Utc.from_utc_datetime(&n.into()).into();
                            Timestamp::from(sys)
                        }
                    ),
                    extra_personal: data.as_slice()[2..].to_vec(),
                }
            )
        } else {
            Err("Bad structure".into())
        }
    }
}


pub struct TeamVecToNominations(pub(crate) Vec<Team>);

impl TeamVecToNominations {
    pub fn convert(&self) -> Vec<NominationDeclaration> {
        let teams = &self.0;
        let mut mp: HashMap<String, NominationDeclaration> = HashMap::new();
        teams.iter()
            .for_each(
                |x| {
                    if let Some(pointer) = mp.get_mut(&x.nom) {
                        pointer.teams.insert(x.tid, x.clone());
                        pointer.inner_queue.push(x.tid);
                    } else {
                        mp.insert(
                            x.nom.clone(),
                            NominationDeclaration {
                                title: x.nom.clone(),
                                teams: {
                                    let mut tmp = HashMap::new();
                                    tmp.insert(x.tid, x.clone());
                                    tmp
                                },
                                inner_queue: vec![x.tid],
                            }
                        );
                    }
                }
            );
        mp.into_values().collect()
    }
}