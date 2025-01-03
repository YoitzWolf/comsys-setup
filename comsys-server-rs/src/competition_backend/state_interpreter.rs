use core::f64;
use std::error::Error;

use crate::gen::comp::JudgeScheme;

use super::comp_state::{CompState, NominationMarks};


// pub trait InterpretationPolicy {
// 
// }


pub struct StateInterpreter {
    //pub policy: dyn InterpretationPolicy
}

impl StateInterpreter {
    pub fn analyse_one(&self, scheme: &JudgeScheme, marks: &NominationMarks) -> Result<(f64, Vec<f64>), Box<dyn Error>> {
        let sizes = scheme.get_lens();
        let results: Vec<f64> = sizes.iter().zip(marks.get_marks().iter())
            .map(
                |(n, marks)| {
                    let l = (*n) as usize;
                    let mut mn: f64 = f64::MAX;
                    let mut mx: f64 = 0.;
                    let mut sum: f64 = 0.;
                    for i in 0..l {
                        if let Some(x) = &marks[i] {
                            let v = f64::try_from(x.0).unwrap();
                            sum += v;
                            mn = f64::min(mn, v);
                            mx = f64::max(mx, v);
                        }
                    }
                    if *n > 2 {
                        (sum - mx - mn) / f64::try_from(*n - 2).unwrap()
                    } else {
                        (sum) / f64::try_from(*n).unwrap()
                    }
                }
            ).collect();
        let fin_res: f64 = results.iter().sum::<f64>() - marks.get_fines().iter().map(|x|{f64::try_from(*x).unwrap()}).sum::<f64>();
        
        Ok((fin_res, results))
    }
}