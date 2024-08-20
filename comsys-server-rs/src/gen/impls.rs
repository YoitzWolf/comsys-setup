use super::comp::JudgeScheme;




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
            "Исполнение" => Ok(0), // 4 or 6
            "Артистичность" => Ok(1),  // 4 or 6
            "Сложность" => Ok(2),  // 2
            _ => Err(())
        }
    }

}