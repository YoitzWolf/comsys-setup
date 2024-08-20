use crate::competition_backend::comp_state::CompState;
use crate::competition_backend::msq::MessagePool;
use crate::db_mng::comp_mng::{self, get_comp_data, get_competition};
use crate::db_mng::org_mng::is_owner_of;
use crate::db_mng::time_convert::into_timestamp;
use crate::gen::comp::CompDeclaration;
use crate::r#gen::comp::{CompStatus, JudgeScheme};
use crate::r#gen::comp_handler::eq_message::Message;
use crate::gen::comp_handler::Verification;
use crate::r#gen::comp_handler::VoteMessage;
use crate::gen::generic::{
    id_result, DatePair, Empty, GenericResult, GenericResultMessage, IdResult, IdsList,
};

use crate::gen::comp_handler::{
    EqHistoryMessage,EqMessage,EqHistory,EqHistoryRequest,
    competition_handler_server::CompetitionHandler
};

use crate::r#gen::generic;
use crate::{has_ability_to_modify, PermissionComparator, Permissions, TokenClaim};

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use tokio_stream::StreamExt;
use std::collections::HashMap;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::sync::broadcast::*;
use tokio_stream::{Stream, wrappers::BroadcastStream, wrappers::ReceiverStream};
use tonic::{Request, Response, Status};


pub struct CompHandlerService {
    //token_generator: Arc<Mutex<TokenGenerator>>,
    db_con: Arc<Mutex<Pool<AsyncPgConnection>>>,
    message_pool: Arc<Mutex< HashMap<i32, Arc<Mutex<MessagePool>> > >>,
    comp_state: Arc<Mutex< HashMap<i32, Arc<Mutex<CompState>> > >>
}

impl CompHandlerService {
    pub(crate) fn new(
        //token_generator: Arc<Mutex<TokenGenerator>>,
        db_con: Arc<Mutex<Pool<AsyncPgConnection>>>,
        

    ) -> Self {
        Self {
            //token_generator, //: TokenGenerator::new(),
            db_con,
            message_pool: Arc::new(Mutex::new(HashMap::new())),
            comp_state: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}


#[tonic::async_trait]
impl CompetitionHandler for CompHandlerService {

    async fn run(
        &self, request: tonic::Request<generic::Id>
    ) -> Result<Response<Empty>, Status> {
        let (meta, ext, req) = request.into_parts();
        let coid = req.id; // Competition ID

        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match has_ability_to_modify(&mut conn, ext.user_id, coid, &ext.perms).await {
                    true => {
                        match (get_competition(&mut conn, coid).await, get_comp_data(&mut conn, coid).await) {
                            (Ok(declaration), Ok(data)) => {
                                let moment_status = CompStatus::try_from(declaration.status).unwrap_or(CompStatus::Unknown);
                                match moment_status {
                                    CompStatus::Declaration | CompStatus::Registration | CompStatus::Waiting => {
                                        // TODO
                                        
                                        if self.message_pool.lock().await.contains_key(&declaration.id) {
                                            // already created
                                            Err(Status::already_exists("Already runned"))
                                        } else {
                                            {
                                                match 
                                                    (
                                                        self.message_pool.lock().await.insert(
                                                            coid, Arc::new(Mutex::new(
                                                                MessagePool::new()
                                                            ))
                                                        ),
                                                        self.comp_state.lock().await.insert(
                                                            coid, Arc::new(Mutex::new(
                                                                CompState::new_clean(
                                                                    {
                                                                        CompDeclaration {
                                                                            title: declaration.title,
                                                                            public: declaration.public,
                                                                            related_organisation_id: declaration.organisation,
                                                                            dates: Some(DatePair {
                                                                                begins: into_timestamp(declaration.start_date),
                                                                                ends: into_timestamp(declaration.ends_date),
                                                                            }),
                                                                            place: declaration.place,
                                                                            descr: declaration.descr,
                                                                            scheme: declaration.scheme,
                                                    
                                                                            queues: bincode::deserialize(&data.queues).unwrap(),
                                                                            part_list: bincode::deserialize(&data.participants).unwrap(),
                                                                        }
                                                                    }
                                                                )
                                                            ))
                                                        )
                                                    )
                                                
                                                {
                                                    (Some(_), _) | (_, Some(_)) => Err(Status::internal("Pool broken!")),
                                                    (None, None) => Ok(Response::new(Empty{})),
                                                }
                                            }
                                        }
                                    },
                                    CompStatus::Running => {
                                        // TODO check exsistance of message pool
                                        // if not exist - try to re-create it and upload older history
                                        // if exist - send GenericResult::Error("Already runned")
                                        Err(Status::already_exists("Already runned"))
                                    },
                                    //CompStatus::Waiting => {
                                    //    Err(Status::internal("Waiting.."))
                                    //},
                                    _ => Err(Status::internal("Invalid Competition status")),
                                }
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                Err(Status::not_found("Comp not found"))
                            }
                        }                
                    },
                    false => {
                        Err(Status::permission_denied("Permission Denied!"))
                    }
                }
            } else {
                Err(Status::internal("Database error"))
            }
        } else {
            Err(Status::permission_denied("No authorisation found!"))
        }
    }

    type startEQMessageStreamStream = Pin<Box<dyn Stream<Item = Result<EqHistoryMessage, Status>> + Send + Sync>>;

    async fn start_eq_message_stream(
        &self, request: Request<generic::Id>
    ) -> Result<Response<Self::startEQMessageStreamStream>, Status> {
        let (meta, ext, req) = request.into_parts();
        let coid = req.id; // Competition ID

        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match get_competition(&mut conn, coid).await {
                    Ok(declaration) => {
                        let perms: Vec<Permissions> = ext.perms.iter().filter(|(oid, perms)| {
                            declaration.organisation.eq(oid)
                        }).map(|(_, perms)| {perms.clone()})
                          .collect::<Vec<Vec<Permissions>>>()
                          .concat()
                          .iter()
                          .cloned()
                          .filter(|perm| {
                            match perm {
                                Permissions::Administrate => true,
                                Permissions::Moderator(access) | Permissions::Watch(access) => match access {
                                    crate::AccessType::All => true,
                                    crate::AccessType::List(oplist) => oplist.contains(&coid),
                                },
                                Permissions::Judge(_coid, _quid, _mark_group) => coid.eq(_coid),
                                Permissions::Secretary(_coid) => coid.eq(_coid),
                                Permissions::Supervisor(_coid) => coid.eq(_coid),
                                _ => false,
                            }
                          }).collect();
                        if perms.len() == 0 && is_owner_of(&mut conn, ext.user_id, declaration.organisation).await.is_err() {
                            Err(Status::permission_denied("Permission Denied!"))
                        } else {
                            if let Some(pool) = self.message_pool.lock().await.get(&coid) {
                                let (tx, rx) = mpsc::channel(128);
                                { pool.lock().await.subscribe(tx).await; }
                                Ok(
                                    Response::new(
                                        Box::pin( ReceiverStream::new(rx))// as Self::startEQMessageStreamStream
                                    )
                                )
                            } else {
                                Err(Status::not_found("Comp not found"))
                            }
                        }
                    },
                    Err(x) => Err(Status::not_found("No such competition info!")),
                }

            } else {
                Err(Status::internal("Database error"))
            }
        } else {
            Err(Status::permission_denied("No authorisation found!"))
        }
    }

    async fn push_eq_message(
        &self,
        request: Request<EqMessage>,
    ) -> Result<Response<generic::Id>, Status> {
        let (meta, ext, req) = request.into_parts();
        let coid = req.comp_id;
        if req.message.is_none() {
            return Err(Status::invalid_argument("no message in request"));
        }

        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match get_competition(&mut conn, coid).await {
                    Ok(declaration) => {
                        match CompStatus::try_from(declaration.status).unwrap_or(CompStatus::Unknown) {
                            CompStatus::Declaration => Err(Status::internal("Not runned yet!")),
                            CompStatus::Registration => Err(Status::internal("Not runned yet!")),
                            CompStatus::Waiting => Err(Status::internal("Waiting..")),
                            CompStatus::Running => Ok(()),
                            CompStatus::Completed => Err(Status::internal("Competition already completed")),
                            CompStatus::Unknown => Err(Status::internal("Competition status unknown")),
                        }?;

                        let perms: Vec<Permissions> = ext.perms.iter().filter(|(oid, perms)| {
                            declaration.organisation.eq(oid)
                        }).map(|(_, perms)| {perms.clone()})
                          .collect::<Vec<Vec<Permissions>>>()
                          .concat()
                          .iter()
                          .cloned()
                          .filter(|perm| {
                            match perm {
                                Permissions::Administrate => true,
                                Permissions::Moderator(access) => match access {
                                    crate::AccessType::All => true,
                                    crate::AccessType::List(oplist) => oplist.contains(&coid),
                                },
                                Permissions::Judge(_coid, _quid, _mark_group) => coid.eq(_coid),
                                Permissions::Secretary(_coid) => coid.eq(_coid),
                                Permissions::Supervisor(_coid) => coid.eq(_coid),
                                _ => false,
                            }
                          }).collect();
                        match JudgeScheme::try_from(declaration.scheme) {
                            Ok(scheme) => {
                                // generate closure to check permissions
                                let perm_required = |x: PermissionComparator| {
                                    // check, this `contains` is not Vec::contains; Its PermissionComparator::contains !!
                                    x.contains(
                                        &Permissions::Administrate
                                    ) ||
                                    x.contains(
                                        &Permissions::Moderator(crate::AccessType::List(vec![coid]))
                                    ) ||
                                    match req.message.clone().unwrap() {
                                        Message::VoteMessage(vote) => { 
                                            x.contains(
                                                &Permissions::Judge(coid, vote.queue_id, scheme.get_judgement_group_id(vote.mark_type).unwrap_or(-1))
                                            )   
                                        },
                                        Message::VerifyMessage(verify) => {
                                            x.contains(
                                                &Permissions::Supervisor(coid)
                                            )
                                        },
                                        Message::FixVoting(fix) => {
                                            x.contains(
                                                &Permissions::Secretary(coid)
                                            )
                                        },
                                        Message::TryNext(next) => {
                                            x.contains(
                                                &Permissions::Secretary(coid)
                                            )
                                        },   
                                        Message::Block(b) => {
                                            x.contains(
                                                &Permissions::Secretary(coid)
                                            )
                                        }
                                    }
                                };
                                // Here Perms are checked after if and we can verify how correct is message in position of compability with Competition state
                                if perm_required(PermissionComparator(perms)) {
                                    // check compability with Comp state
                                    match self.message_pool.lock().await.get(&coid) {
                                        Some(pool) => {
                                            let task_done : Result<(), Status> = match req.message.clone().unwrap() {
                                                Message::VoteMessage(vote) => {
                                                    match self.comp_state.lock().await.get(&coid) {
                                                        Some(c_state) => {
                                                            c_state.lock().await.able_to_add_vote(&vote)
                                                        },
                                                        None => {
                                                            Err(Status::not_found("Not started competition state control!"))
                                                        },
                                                    }
                                                },
                                                Message::VerifyMessage(verify) => {
                                                    {
                                                        let queue = pool.lock().await;
                                                        if queue.not_verified(verify.target_message_id).await {
                                                            // verify
                                                            let verified = queue.remove_from_unverifyed(verify.target_message_id).await?;
                                                            let verified_msg = queue.index(verified).await.unwrap();
                                                            if let Verification::Approve = verify.verdict() {
                                                                // vote message approved
                                                                match verified_msg.message.unwrap().message.unwrap() {
                                                                    Message::VoteMessage(vote) => {
                                                                        match self.comp_state.lock().await.get(&coid) {
                                                                            Some(c_state) => {
                                                                                c_state.lock().await.try_to_add_vote(vote)
                                                                            },
                                                                            None => {
                                                                                Err(Status::not_found("Not started competition state control!"))
                                                                            },
                                                                        }
                                                                    },
                                                                    _ => Err(Status::internal("REPORT THIS ERROR: Competition Handler Err")),
                                                                }
                                                            } else {
                                                                // vote message blocked
                                                                Ok(())
                                                            }
                                                        } else {
                                                            Err(Status::invalid_argument("Unable to verify"))
                                                        }
                                                    }
                                                },
                                                Message::FixVoting(fix) => {
                                                    match self.comp_state.lock().await.get_mut(&coid) {
                                                        Some(a) => {
                                                            let marks = a.lock().await.try_finish_action(fix.queue_id, fix.verdict())?;
                                                            Ok(())
                                                        },
                                                        None => Err(Status::not_found("Comp State not found")),
                                                    }
                                                },
                                                Message::TryNext(next) => {
                                                    match self.comp_state.lock().await.get_mut(&coid) {
                                                        Some(a) => {
                                                            a.lock().await.try_next_action(next.queue_id)?;
                                                            Ok(())
                                                        },
                                                        None => Err(Status::not_found("Comp State not found")),
                                                    }
                                                },
                                                Message::Block(block) => {
                                                    todo!()
                                                }
                                            };
                                            match task_done {
                                                Ok(_) => {
                                                    match pool.lock().await.send(req).await {
                                                        Ok(mid) => {
                                                            Ok(Response::new(generic::Id{id: mid as i32}))
                                                        },
                                                        Err(e) =>{
                                                            Err(e)
                                                        },
                                                    }
                                                },
                                                Err(e) => Err(e),
                                            }
                                        },
                                        None => {
                                            Err(Status::not_found("Not started"))
                                        },
                                    }
                                } else {
                                    Err(Status::permission_denied("Permission denied!"))
                                }
                            },
                            Err(_) => Err(Status::internal("Database error, incompatible enum interpretation!")),
                        }
                    },
                    Err(err) => {
                        Err(Status::not_found("Comp not found"))
                    },
                }
            } else {
                Err(Status::internal("Database error"))
            }
        } else {
            Err(Status::permission_denied("No authorisation found!"))
        }

        /*match self.message_pool.lock().await.get(&coid) {
            Some(pool) => {
                match pool.lock().await.send(req).await {
                    Ok(x) => {
                        // println!("Push message return;");
                        Ok(Response::new(generic::Id{id: x as i32}))
                    },
                    Err(e) => Err(Status::aborted("Send error")),
                }
            },
            None => Err(Status::not_found("not found")),
        }*/
    }

    async fn pull_eq_message_history(
        &self,
        request: Request<EqHistoryRequest>,
    ) -> Result<Response<EqHistory>, Status> {
        todo!()
    }
}