use std::{collections::HashMap, rc::Rc, sync::Arc};

use yew::{virtual_dom::Listener, Reducible, UseReducerHandle};

use crate::grpc::{comp::{comps_list::CompView, JudgeScheme, Participant, Team}, comp_handler::{EqHistoryMessage, Verification, VoteMessage}, generic::IdsList};

pub enum DropSetAction<T>  {
    Set(T),
    Drop
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct IdListContext(pub Vec<i32>);

impl Reducible for IdListContext {
    type Action = DropSetAction<IdListContext>;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            DropSetAction::Set(vals) => {
                let mut newc = self.as_ref().clone();
                newc.0 = vals.0;
                Rc::new(newc)
            },
            DropSetAction::Drop => {
                let mut newc = self.as_ref().clone();
                newc.0.clear();
                Rc::new(newc)
            },
        }
    }
}


#[derive(Debug)]
pub enum EqmContextAction<T> {
    Set(Vec<T>),
    Drop,
    Add(T),
    Connect(Vec<T>),
    Remove(i32),
}


pub type EventQueueMessageContext = UseReducerHandle<EQMContext>;

/// qid , <Mark Type name : <Vec of Votes>>
pub type CompQueueState =  HashMap::< String, Vec<(i32, VoteMessage)>>;


#[derive(Clone, Debug, Default, PartialEq)]
pub struct EQMContext(pub Vec<EqHistoryMessage>, HashMap<i32, CompQueueState>);

impl Reducible for EQMContext {
    type Action = EqmContextAction<EqHistoryMessage>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        
        match action {
            EqmContextAction::Set(vals) => {
                let mut newc = self.as_ref().clone();
                newc.0 = vals;
                Rc::new(newc)
            },
            EqmContextAction::Drop => {
                let mut newc = self.as_ref().clone();
                newc.0 = vec![];
                Rc::new(newc)
            },
            EqmContextAction::Add(val) => {
                let mut newc = self.as_ref().clone();
                let uid = &val.message_id;
                match newc.0.binary_search_by(|x| {
                    let delta = x.message_id - uid;
                    if delta == 0 { std::cmp::Ordering::Equal }
                    else if delta < 0 { std::cmp::Ordering::Less }
                    else              { std::cmp::Ordering::Greater }
                }) {
                    Ok(i) => *newc.0.get_mut(i).unwrap() = val,
                    Err(i) => {
                        if i <= newc.0.len() {
                            newc.0.push(val);
                        } else {
                            let interest = newc.0.get(i).unwrap();
                            if interest.message_id < val.message_id {
                                newc.0.insert(i+1, val);
                            } else {
                                newc.0.insert(i, val);
                            }
                        }
                        
                    },
                }
                Rc::new(newc)
            },
            EqmContextAction::Remove(id) => {
                let mut newc = self.as_ref().clone();
                newc.0.remove(id as usize);
                Rc::new(newc)
            },
            EqmContextAction::Connect(vals) => {
                let mut newc = self.as_ref().clone();
                for val in vals.into_iter() {
                    let uid = &val.message_id;
                    match newc.0.binary_search_by(|x| {
                        let delta = x.message_id - uid;
                        if delta == 0 { std::cmp::Ordering::Equal }
                        else if delta < 0 { std::cmp::Ordering::Less }
                        else              { std::cmp::Ordering::Greater }
                    }) {
                        Ok(i) => *newc.0.get_mut(i).unwrap() = val,
                        Err(i) => {
                            if newc.0.len() <= i {
                                newc.0.push(val);
                            } else {
                                let interest = newc.0.get(i).unwrap();
                                if interest.message_id < val.message_id {
                                    newc.0.insert(i+1, val);
                                } else {
                                    newc.0.insert(i, val);
                                }
                            }
                        },
                    }
                }
                Rc::new(newc)
            },
        }
    }
}


/*-------------------------------------------------------------------------------------------- */


#[derive(Debug, Clone, PartialEq)]
pub struct ActiveActionView {
    pub nomination: String,
    pub action_id: i32,
    pub active_team: (Team, Vec<Participant>),
    pub schema: JudgeScheme,
    pub marks: HashMap<
            String,
            Vec<(i32, Option<Verification>)>
        >
}

impl TryFrom<(&CompView, CompQueueState)> for ActiveActionView {
    type Error = ();
    fn try_from((view, state): (&CompView, CompQueueState)) -> Result<Self, Self::Error> {
        Ok(
            Self {
                nomination: todo!(),
                action_id: todo!(),
                active_team: todo!(),
                schema: todo!(),
                marks: todo!(),
            }
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActiveActionViewContext(pub Option<ActiveActionView>);