use std::collections::HashMap;
use std::error::Error;
use std::ops::Index;

use serde::{Deserialize, Serialize};
use tonic::Status;

use crate::gen::comp::{
    CompDeclaration, CompetitionQueue, NominationDeclaration, Participant, Team,
};
use crate::gen::comp_handler::vote_list::VoteView;
use crate::gen::comp_handler::{ActiveActionState, FinesSetup, Verification, VoteList};
use crate::gen::impls::*;
use crate::gen::{comp::JudgeScheme, comp_handler::VoteMessage};
use crate::models::CompData;
use crate::schema::comp_data::{queues};

use super::state_interpreter::StateInterpreter;

/// (message_id, VoteMessage, author id)
type Mark = (i32, VoteMessage, i32);
type MarkGroup = Vec<Vec<Option<Mark>>>;
type Fines = Vec<i32>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NominationMarks {
    in_progress: bool,
    active_action: i32,
    team: Option<Team>,
    mark_groups: MarkGroup, // Vec<Vec<Option<(i32, VoteMessage)>>>,
    fines: Fines, //Vec<i32>,
    voted_users: Vec<i32>,
}

impl NominationMarks {

    pub fn get_marks(&self) -> &MarkGroup {
        &self.mark_groups
    }

    pub fn get_fines(&self) -> &Fines {
        &self.fines
    }

    pub fn with_scheme(scheme: &JudgeScheme) -> Self {
        match scheme {
            JudgeScheme::FourFourTwo => {
                Self {
                    //scheme: scheme.clone(),
                    in_progress: false,
                    active_action: -1,
                    team: None,
                    mark_groups: { vec![vec![None; 4], vec![None; 4], vec![None; 2]] },
                    voted_users: vec![],
                    fines: vec![],
                }
            },
            JudgeScheme::SixSixTwo => {
                Self {
                    //scheme: scheme.clone(),
                    in_progress: false,
                    active_action: -1,
                    team: None,
                    mark_groups: { vec![vec![None; 6], vec![None; 6], vec![None; 2]] },
                    voted_users: vec![],
                    fines: vec![],
                }
            },
            JudgeScheme::FourFourOne => {
                Self {
                    //scheme: scheme.clone(),
                    in_progress: false,
                    active_action: -1,
                    team: None,
                    mark_groups: { vec![vec![None; 4], vec![None; 4], vec![None; 1]] },
                    voted_users: vec![],
                    fines: vec![],
                }
            },
        }
    }

    pub fn clean_marks_with_scheme(&mut self, scheme: &JudgeScheme) {
        self.voted_users.clear();
        match scheme {
            JudgeScheme::FourFourTwo => {
                self.mark_groups = { vec![vec![None; 4], vec![None; 4], vec![None; 2]] };
            },
            JudgeScheme::SixSixTwo => {
                self.mark_groups = { vec![vec![None; 6], vec![None; 6], vec![None; 2]] };
            },
            JudgeScheme::FourFourOne => {
                self.mark_groups = { vec![vec![None; 4], vec![None; 4], vec![None; 1]] };
            }
        }
    }
    
    fn ready(&self) -> bool {
        self.mark_groups.iter().all(
            |marks| {
                marks.iter().all(
                    |v| {
                        match &v {
                            Some(x) => {true},
                            None => false
                        }
                    }
                )
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnRunnedCompDeclaration {
    title: String,
    public: bool,
    related_organisation_id: i32,
    descr: Option<String>,
    scheme: JudgeScheme,
    //part_list: Vec<Participant>,
    queues: Vec<CompetitionQueue>,
    ready_queues: Vec<
        // queue
        Vec<
            // nomination
            (String, Vec<NominationMarks>), // list of MarkPacks!
        >,
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
            //part_list: declaration.part_list,
            ready_queues: vec![vec![]; declaration.queues.len()], //Vec::with_capacity(declaration.queues.len()),
            queues: declaration.queues,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompState {
    declaration: OnRunnedCompDeclaration,
    queues: Vec<NominationMarks>,
}

impl CompState {
    pub fn get_queue_active_action(&self, qid: usize) -> Option<ActiveActionState> {
        let quid: i32 = qid.try_into().unwrap();
        match self.queues.get(qid) {
            Some(nomination) => {
                if nomination.active_action == -1 {
                    return None;
                } else {
                    /*let parts = match &nomination.team {
                        Some(t) => t
                            .participants
                            .iter()
                            .map(|p| {
                                let s: usize = p.clone().try_into().unwrap();
                                self.declaration.part_list.index(s).clone()
                            })
                            .collect::<Vec<Participant>>(),
                        None => Vec::<_>::new(),
                    };*/
                    Some(ActiveActionState {
                        qid: quid,
                        aid: nomination.active_action,
                        team: nomination.team.clone(),
                        //participants: parts,
                        marks: {
                            let mut mkh = HashMap::new();
                            for (i, marks) in nomination.mark_groups.iter().enumerate() {
                                mkh.insert(
                                    self.declaration
                                        .scheme
                                        .get_judgement_group_name(i.try_into().unwrap())
                                        .unwrap(),
                                    VoteList {
                                        votes: marks
                                            .iter()
                                            .map(|mks| {
                                                match mks {
                                                    Some(v) => {
                                                        VoteView {
                                                            //author_id: todo!(),
                                                            message_id: v.0,
                                                            verifyed: Verification::Approve.into(),
                                                            mark: v.1.mark,
                                                        }
                                                    }
                                                    None => VoteView {
                                                        //author_id: todo!(),
                                                        message_id: -1,
                                                        verifyed: Verification::Block.into(),
                                                        mark: 0,
                                                    },
                                                }
                                            })
                                            .collect(),
                                    },
                                );
                            }
                            mkh
                        },
                    })
                }
            }
            None => None,
        }
    }

    pub fn new_clean(declaration: CompDeclaration) -> Self {
        let N = declaration.queues.len();
        Self {
            declaration: OnRunnedCompDeclaration::from(declaration),
            queues: vec![NominationMarks::default(); N],
        }
    }

    pub fn voted(&self, author: i32, quid: i32) -> bool {
        match self.queues.get(quid as usize) {
            Some(nomination) => nomination.voted_users.contains(&author),
            None => false,
        }
    }

    pub fn try_to_add_vote(
        &mut self,
        author: i32,
        vote_author: i32,
        vote_mess_id: i32,
        vote: VoteMessage,
    ) -> Result<(), Status> {
        match self
            .declaration
            .scheme
            .get_judgement_group_id(vote.mark_type.clone())
        {
            Ok(mark_id) => {
                match self.queues.get_mut(vote.queue_id as usize) {
                    Some(nomination) => {
                        if nomination.in_progress && !nomination.voted_users.contains(&author) {
                            if nomination.active_action.eq(&vote.action_id) {
                                // correct vote
                                match nomination.mark_groups.get_mut(mark_id as usize) {
                                    Some(v) => {
                                        match v.iter_mut().find(|x| x.is_none()) {
                                            Some(vote_pointer) => {
                                                *vote_pointer = Some((vote_mess_id, vote, vote_author));
                                                nomination.voted_users.push(vote_author);
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
                    }
                    None => Err(Status::invalid_argument("Queue not found")),
                }
            }
            Err(e) => Err(Status::invalid_argument("Invalid judgement mark type name")),
        }
    }

    pub fn try_to_add_fines(
        &mut self,
        fines: FinesSetup,
    ) -> Result<(), Status> {
        match self.queues.get_mut(fines.queue_id as usize) {
            Some(nomination) => {
                if nomination.in_progress {
                    if nomination.active_action.eq(&fines.action_id) {
                        // correct vote
                        nomination.fines = fines.fines.clone();
                        Ok(())
                    } else {
                        // not correct
                        Err(Status::invalid_argument("Invalid action Id"))
                    }
                } else {
                    Err(Status::internal("Voting is blocked at the moment!"))
                }
            }
            None => Err(Status::invalid_argument("Queue not found")),
        }
    }

    pub fn able_to_add_vote(&self, author: i32, vote: &VoteMessage) -> Result<(), Status> {
        match self.queues.get(vote.queue_id as usize) {
            Some(nomination) => {
                println!("{:?}", nomination);
                if nomination.active_action.eq(&vote.action_id)
                    && nomination.in_progress
                    && !nomination.voted_users.contains(&author)
                {
                    Ok(())
                } else {
                    // not correct
                    Err(Status::invalid_argument(
                        "Invalid action Id or Voting is closed",
                    ))
                }
            }
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
                                    let redq = self
                                        .declaration
                                        .ready_queues
                                        .get_mut(qui as usize)
                                        .unwrap();
                                    if redq.len() == 0 || !redq.last().unwrap().0.eq(&nom.title) {
                                        redq.push((nom.title.clone(), vec![]));
                                    }
                                };
                                match nom.inner_queue.first_mut() {
                                    Some(aid) => {
                                        let team = nom.teams.get(aid).cloned();
                                        if team.is_none() {
                                            return Err(Status::data_loss(
                                                "Database data is corrupted",
                                            ));
                                        }
                                        nomination.active_action = *aid;
                                        nomination.team = team;
                                        nomination.in_progress = true;
                                        nomination
                                            .clean_marks_with_scheme(&self.declaration.scheme);
                                        break;
                                    }
                                    None => {
                                        queue.nomination_list.remove(0);
                                        continue;
                                    }
                                };
                            }
                            None => return Err(Status::cancelled("Queue is empty!")),
                        }
                    }
                    Ok(())
                } else {
                    // not correct
                    Err(Status::invalid_argument("Voting is in progress!"))
                }
            }
            None => Err(Status::invalid_argument("Queue not found")),
        }
    }

    pub fn try_finish_action(
        &mut self,
        qui: i32,
        verdict: Verification,
    ) -> Result<NominationMarks, Status> {

        if let Verification::NotChecked = verdict {
            return Err(Status::invalid_argument("Invalid Verdict!"));
        }

        match self.queues.get_mut(qui as usize) {
            Some(nomination) => {
                if nomination.in_progress {
                    match self.declaration.queues.get_mut(qui as usize) {
                        Some(queue) => match queue.nomination_list.first_mut() {
                            Some(nomdec) => match nomdec.inner_queue.first() {
                                Some(q) => {
                                    if nomination.active_action.eq(q) {
                                        
                                        if let Verification::Approve = verdict {
                                            if ! nomination.ready()  {
                                                return Err(Status::internal("Not finished by Supervisor!"));
                                            }
                                        }

                                        nomination.in_progress = false;
                                        let ret = nomination.clone();

                                        if let Verification::Approve = verdict {
                                            nomdec.inner_queue.remove(0);
                                            self.declaration
                                                .ready_queues
                                                .get_mut(qui as usize)
                                                .unwrap()
                                                .last_mut()
                                                .unwrap()
                                                .1
                                                .push(ret.clone());
                                        }
                                        nomination.active_action = -1;
                                        nomination.team = None;
                                        nomination
                                            .clean_marks_with_scheme(&self.declaration.scheme);

                                        Ok(ret)
                                    } else {
                                        Err(Status::internal("Report this error! active action is not first in nomination inner queue!"))
                                    }
                                }
                                None => Err(Status::internal(
                                    "Report this error! Unable to get queue to check active action",
                                )),
                            },
                            None => Err(Status::internal(
                                "Report this error! unable to get nomination queue!",
                            )),
                        },
                        None => Err(Status::invalid_argument("invalid queue id")),
                    }
                } else {
                    Err(Status::internal("No active action"))
                }
            }
            None => Err(Status::invalid_argument("Queue not found")),
        }
    }


    pub fn try_collect(&self, interpreter: &StateInterpreter)
        -> Result<
            Vec<(usize, HashMap<&String, Vec<(Option<Team>, Result<(f64, Vec<f64>), Box<dyn Error>>, (&Vec<Vec<Option<(i32, VoteMessage, i32)>>>, &Vec<i32>))>>)>,
            Status // Box<dyn Error>
        > 
    {
        for nom_marks in  self.queues.iter() {
            if nom_marks.in_progress {
                return Err(Status::failed_precondition("Some actions are not finished yet"));
            }
        }
        // let scheme = self.declaration.scheme.clone();
        let ready = &self.declaration.ready_queues;
        let mut resz = vec![];
        for (qid, queue) in ready.iter().enumerate() {
            let mut queue_marks = HashMap::new();
            for nomination in queue {
                let name = &nomination.0;
                let res_mks: Vec<_> = (&nomination.1).iter().map(
                    |mk| {
                        // Team | (total_res, mark_group_res) | (marks, fines)
                        (
                            mk.team.clone(),
                            { interpreter.analyse_one(&self.declaration.scheme, mk) },
                            (mk.get_marks(), mk.get_fines())
                        )
                    }
                ).collect();
                queue_marks.insert(name, res_mks);
            }
            resz.push((qid, queue_marks));
        }
        Ok(resz)
    }
}
