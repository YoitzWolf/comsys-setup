use crate::{db_mng::time_convert::into_timestamp, models::Competition};

use super::{comp::{comps_list::CompView, CompDeclaration, JudgeScheme}, generic::DatePair};




impl JudgeScheme {
        
    pub fn get_judgement_group_name(&self, quid: i32) -> Result<String, ()> {
        match quid {
            0 => Ok("Исполнение".to_string()), // 4 or 6
            1 => Ok("Артистичность".to_string()),  // 4 or 6
            2 => Ok("Сложность".to_string()),  // 2
            _ => Err(())
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
            "Исполнение" => Ok(0), // 4 or 6 max times
            "Артистичность" => Ok(1),  // 4 or 6 max times
            "Сложность" => Ok(2),  // 2
            _ => Err(())
        }
    }

    pub fn get_vec(&self) -> Vec<Vec<i32>> {
        match self {
            JudgeScheme::FourFourTwo => {
                vec![ vec![0; 4],vec![0; 4],vec![0; 2] ]
            },
            JudgeScheme::SixSixTwo => {
                vec![ vec![0; 6],vec![0; 6],vec![0; 2] ]
            },
            JudgeScheme::FourFourOne => {
                vec![ vec![0; 4],vec![0; 4],vec![0; 1] ]
            },
        }
    }

    pub fn get_lens(&self) -> Vec<i32> {
        match self {
            JudgeScheme::FourFourTwo => {
                vec![ 4,4,2 ]
            },
            JudgeScheme::SixSixTwo => {
                vec![ 6,6,2 ]
            },
            JudgeScheme::FourFourOne => {
                vec![ 4,4,1 ]
            },
        }
    }

}


impl From<&Competition> for CompDeclaration {
    fn from(value: &Competition) -> Self {
        Self {
            title: value.title.clone(),
            public: value.public,
            related_organisation_id: value.organisation,
            dates: Some(
                DatePair {
                    begins: into_timestamp(value.start_date),
                    ends: into_timestamp(value.ends_date),
                }
            ),
            place: value.place.clone(),
            descr: value.descr.clone(),
            scheme: value.scheme,
            //part_list: vec![],
            queues: vec![Default::default();value.queues.try_into().unwrap()],
        }
    }
}


impl From<&Competition> for CompView {
    fn from(value: &Competition) -> Self {
        Self {
            id: value.id,
            declaration: Some(CompDeclaration::from(value)),
        }
    }
}