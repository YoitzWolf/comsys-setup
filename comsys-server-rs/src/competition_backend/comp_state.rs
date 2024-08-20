use serde::{Deserialize, Serialize};
use tonic::Status;

use crate::gen::comp::{CompDeclaration, CompetitionQueue, NominationDeclaration, Participant, Team};
use crate::gen::comp_handler::Verification;
use crate::gen::{comp::JudgeScheme, comp_handler::VoteMessage};
use crate::gen::impls::*;
use crate::models::CompData;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NominationMarks{
    in_progress: bool,
    active_action: i32,
    team: Option<Team>,
    mark_groups: Vec<
        Vec< Option<VoteMessage> >
    >
}


impl NominationMarks {
    pub fn with_scheme(scheme: &JudgeScheme) -> Self {
        match scheme {
            JudgeScheme::FourFourTwo => {
                Self {
                    //scheme: scheme.clone(),
                    in_progress: false,
                    active_action: -1,
                    team: None,
                    mark_groups: {
                        let mut v = Vec::with_capacity(3);
                        v[0] = Vec::with_capacity(4);
                        v[1] = Vec::with_capacity(4);
                        v[2] = Vec::with_capacity(2);
                        v
                    }
                }
            },
            JudgeScheme::SixSixTwo => {
                Self {
                    //scheme: scheme.clone(),
                    in_progress: false,
                    active_action: -1,
                    team: None,
                    mark_groups: {
                        let mut v = Vec::with_capacity(3);
                        v[0] = Vec::with_capacity(6);
                        v[1] = Vec::with_capacity(6);
                        v[2] = Vec::with_capacity(2);
                        v
                    }
                }
            },
        }
    }

    pub fn clean_marks_with_scheme(&mut self, scheme: &JudgeScheme) {
        match scheme {
            JudgeScheme::FourFourTwo => {
                
                self.mark_groups = {
                    let mut v = Vec::with_capacity(3);
                    v[0] = Vec::with_capacity(4);
                    v[1] = Vec::with_capacity(4);
                    v[2] = Vec::with_capacity(2);
                    v
                };
            },
            JudgeScheme::SixSixTwo => {
                self.mark_groups = {
                    let mut v = Vec::with_capacity(3);
                    v[0] = Vec::with_capacity(6);
                    v[1] = Vec::with_capacity(6);
                    v[2] = Vec::with_capacity(2);
                    v
                };
            },
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnRunnedCompDeclaration {
    title: String,
    public: bool,
    related_organisation_id: i32,
    descr: Option<String>,
    scheme: JudgeScheme, 
    part_list: Vec<Participant>,
    queues:       Vec<CompetitionQueue>,
    ready_queues:
        Vec< // queue
            Vec < // nomination
                (String, Vec<NominationMarks>)// list of MarkPacks!
            >
        >,
}

impl From<CompDeclaration> for OnRunnedCompDeclaration {
    fn from(declaration: CompDeclaration) -> Self {
        Self {
            title: declaration.title.clone(),
            public: declaration.public,
            related_organisation_id: declaration.related_organisation_id,
            scheme: declaration.scheme(),
            descr: declaration.descr,
            part_list: declaration.part_list,
            ready_queues: Vec::with_capacity(declaration.queues.len()),
            queues: declaration.queues,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompState {
    declaration: OnRunnedCompDeclaration,
    queues: Vec< NominationMarks >
}

impl CompState {
    pub fn new_clean(declaration: CompDeclaration) -> Self {
        let N = declaration.queues.len();
        Self {
            declaration: OnRunnedCompDeclaration::from(declaration),
            queues: Vec::with_capacity(N),
        }
    }

    pub fn try_to_add_vote(&mut self, vote: VoteMessage) -> Result<(), Status> {  

        match self.declaration.scheme.get_judgement_group_id(vote.mark_type.clone()) {
            Ok(mark_id) => {
                match self.queues.get_mut(vote.queue_id as usize) {
                    Some(nomination) => {
                        if nomination.in_progress {
                            if nomination.active_action.eq(&vote.action_id) {
                                // correct vote
                                match nomination.mark_groups.get_mut(mark_id as usize) {
                                    Some(v) => {
                                        match v.iter_mut().find(|x| x.is_none()) {
                                            Some(vote_pointer) => {
                                                *vote_pointer = Some(vote);
                                                Ok(())
                                            },
                                            None => {
                                                Err(Status::internal("Voting is full"))
                                            },
                                        }
                                    },
                                    None => Err(Status::internal("REPORT THIS: mark type is correct, but state is not created!")),
                                }
                            } else {
                                // not correct
                                Err(Status::invalid_argument("Invalid action Id"))
                            }
                        } else {
                            Err(Status::internal("Voting is blocked at the moment!"))
                        }
                    },
                    None => Err(Status::invalid_argument("Queue not found")),
                }
            },
            Err(e) => {
                Err(Status::invalid_argument("Invalid judgement mark type name"))
            },
        }
    }

    pub fn able_to_add_vote(&self, vote: &VoteMessage) -> Result<(), Status> {
        match self.queues.get(vote.queue_id as usize) {
            Some(nomination) => {
                if nomination.active_action.eq(&vote.action_id) && nomination.in_progress {
                    Ok(())
                } else {
                    // not correct
                    Err(Status::invalid_argument("Invalid action Id or Voting is closed"))
                }
            },
            None => Err(Status::invalid_argument("Queue not found")),
        }
    }

    pub fn try_next_action(&mut self, qui: i32) -> Result<(), Status> {
        match self.queues.get_mut(qui as usize) {
            Some(nomination) => {
                if !nomination.in_progress {
                    let mut queue = self.declaration.queues.get_mut(qui as usize).unwrap();
                    loop {
                        match queue.nomination_list.first_mut() {
                            Some(nom) => {
                                {
                                    let redq = self.declaration.ready_queues.get_mut(qui as usize).unwrap();
                                    if redq.len() == 0 || !redq.last().unwrap().0.eq(&nom.title)
                                        { redq.push((nom.title.clone(), vec![])); }
                                };
                                match nom.inner_queue.first_mut() {
                                    Some(aid) => {
                                        let team = nom.teams.get(aid).cloned();
                                        if team.is_none() {
                                            return Err(Status::data_loss("Database data is corrupted"));
                                        }
                                        nomination.active_action = *aid;
                                        nomination.team = team;
                                        nomination.in_progress = true;
                                        nomination.clean_marks_with_scheme(&self.declaration.scheme);
                                        break;
                                    },
                                    None => {
                                        queue.nomination_list.remove(0);
                                        continue;
                                    },
                                };
                            },
                            None => return Err(Status::cancelled("Queue is empty!")),
                        }
                    }
                    Ok(())
                } else {
                    // not correct
                    Err(Status::invalid_argument("Voting is in progress!"))
                }
            },
            None => Err(Status::invalid_argument("Queue not found")),
        }
    }

    pub fn try_finish_action(&mut self, qui: i32, verdict: Verification) -> Result< NominationMarks, Status> {
        match self.queues.get_mut(qui as usize) {
            Some(nomination) => {
                if nomination.in_progress {                    
                    match self.declaration.queues.get_mut(qui as usize) {
                        Some(queue) => {
                            match queue.nomination_list.first_mut() {
                                Some(nomdec) => {
                                    match nomdec.inner_queue.first() {
                                        Some(q) => {
                                            if nomination.active_action.eq(q) {
                                                nomination.in_progress = false;
                                                let ret = nomination.clone();

                                                if let Verification::Approve = verdict {
                                                    nomdec.inner_queue.remove(0);
                                                    self.declaration.ready_queues.get_mut(qui as usize).unwrap().last_mut().unwrap().1.push(ret.clone());
                                                }
                                                nomination.active_action = -1;
                                                nomination.team = None;
                                                nomination.clean_marks_with_scheme(&self.declaration.scheme);

                                                Ok(ret)
                                            } else {
                                                Err(Status::internal("Report this error! active action is not first in nomination inner queue!"))
                                            }
                                        },
                                        None => Err(Status::internal("Report this error! Unable to get queue to check active action"))
                                    }
                                },
                                None => Err(Status::internal("Report this error! unable to get nomination queue!")),
                            }
                        },
                        None => Err(Status::invalid_argument("invalid queue id")),
                    }
                } else {
                    Err(Status::internal("No active action"))
                }
            },
            None => Err(Status::invalid_argument("Queue not found")),
        }
    }


}