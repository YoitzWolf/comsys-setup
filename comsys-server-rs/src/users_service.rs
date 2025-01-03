use std::sync::Arc;

use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::{db_mng::{comp_mng::get_competition, org_mng::is_owner_of, user_mng::{self, setup_selfname}}, gen::{auth::UserView, generic::{self, Empty, StringMessage}, users::{role, user_manage_server::UserManage, Judge, Role, RoleMessage}}, Permissions, TokenClaim};





pub struct UsersService {
    db_con: Arc<Mutex<Pool<AsyncPgConnection>>>,
}

impl UsersService {
    pub fn new(db_con: Arc<Mutex<Pool<AsyncPgConnection>>>) -> Self {
        Self { db_con }
    }
}


#[tonic::async_trait]
impl UserManage for UsersService {
    async fn get_me(&self, request: Request<Empty>) -> Result<Response<UserView>, Status> {
        let (meta, ext, msg) = request.into_parts();
        if let Some(ext) = ext.get::<TokenClaim>() {

            if let Ok(mut con) = self.db_con.lock().await.get().await {
                match user_mng::get_by_id(&mut con, ext.user_id).await {
                    Ok(v) => {
                        Ok(
                            Response::new(UserView{ uid: ext.user_id, login: v.login, selfname: v.selfname })
                        )
                    },
                    Err(_) => return Err(Status::not_found("Not found")),
                }
            } else {
                return Err(Status::internal("Database Error"));
            }            
        } else {
            Err(Status::unauthenticated("Auth error!"))
        }
    }

    async fn setup_selfname(&self, request: Request<StringMessage>) -> Result<Response<Empty>, Status> {
        let (meta, ext, msg) = request.into_parts();
        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match setup_selfname(&mut conn, ext.user_id, &msg.str).await {
                    Ok(_) => Ok(Response::new(Empty{})),
                    Err(_) => return Err(Status::internal("Database Update Error"))
                }
            } else {
                return Err(Status::internal("Database Error"));
            }
        } else {
            Err(Status::unauthenticated("Auth error!"))
        }
    }

    async fn get_my_comp_role(
        &self,
        request: tonic::Request<generic::Id>,
    ) -> std::result::Result<tonic::Response<RoleMessage>, tonic::Status> {
        let (meta, ext, generic::Id{id: coid}) = request.into_parts();
        let mut ADMIN = false;
        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut con) = self.db_con.lock().await.get().await {
                match get_competition(&mut con, coid).await {
                    Ok(declaration) => {
                        if is_owner_of(&mut con, ext.user_id, declaration.organisation).await.is_ok() {
                            ADMIN = true;
                        }
                        let perms: Vec<Permissions> = ext.perms.iter().filter(|(oid, perms)| {
                            declaration.organisation.eq(oid)
                        }).map(|(_, perms)| {perms.clone()})
                          .collect::<Vec<Vec<Permissions>>>()
                          .concat()
                          .iter()
                          .filter(|perm| {
                            match perm {
                                Permissions::Administrate => {
                                    ADMIN = true;
                                    true
                                },
                                Permissions::Moderator(access) => match access {
                                    crate::AccessType::All => {
                                        ADMIN = true;
                                        true
                                    },
                                    crate::AccessType::List(oplist) => {
                                        if oplist.contains(&coid) {
                                            ADMIN = true;
                                            true
                                        } else {
                                            false
                                        }
                                    },
                                },
                                Permissions::Judge(_coid, _quid, _mark_group) => coid.eq(_coid),
                                Permissions::Secretary(_coid) => coid.eq(_coid),
                                Permissions::Arbitor(_coid, quid) => coid.eq(_coid),
                                _ => false,
                            }
                          }).cloned().collect();
                        println!("Admin: {}, perms {:?}", ADMIN, perms);
                        if ADMIN {
                            Ok(Response::new(
                                RoleMessage{
                                    roles: vec![Role{ role: Some(role::Role::Moderator(generic::Empty{}))}]
                                }
                            ))
                        } else {
                            Ok(Response::new(
                                RoleMessage{
                                    roles: perms.iter().filter_map(|perm| {
                                        match perm {
                                            Permissions::Watch(_) => Some(role::Role::Watcher(generic::Empty{})),
                                            Permissions::Create => None,
                                            Permissions::Administrate | Permissions::Moderator(_) => None,
                                            Permissions::Judge(cid, q, m) => Some(role::Role::Judge(Judge{ queue: q.clone(), mark_group: m.clone() })),
                                            Permissions::Secretary(_) => Some(role::Role::Secretary(generic::Empty{})),
                                            Permissions::Arbitor(cid, quid) => Some(role::Role::Arbitor(generic::Id{id:quid.clone()})),
                                        }
                                    }).map(
                                        |r| {
                                            Role {
                                                role: Some(r)
                                            }
                                        }
                                    ).collect()
                                }
                            ))
                        }
                    },
                    Err(_) => return Err(Status::not_found("Competition not found")),
                }
            } else {
                return Err(Status::internal("Database Error"));
            }            
        } else {
            Err(Status::unauthenticated("Auth error!"))
        }
    }
}