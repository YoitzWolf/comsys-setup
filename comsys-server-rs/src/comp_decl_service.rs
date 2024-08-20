use crate::auth_backend::prelude::*;
use crate::auth_backend::tokens::{TokenClaim, TokenGenerator};
use crate::db_mng::comp_mng::{self, generate_comp_staff, get_comp_data, get_competition};
use crate::db_mng::comp_mng::{insert_comp_data, insert_competition};
use crate::db_mng::time_convert::into_timestamp;
use crate::gen::comp::competition_declarator_server::CompetitionDeclarator;
use crate::gen::comp::mod_comp_declaration_request::Command;
use crate::gen::comp::{
    CompDeclaration, CompsList, CompsStatusMessage, DeclareCompetitionResult,
    ModCompDeclarationRequest, ModDeclarationCommand,
};
use crate::gen::generic::{
    id_result, DatePair, Empty, GenericResult, GenericResultMessage, IdResult, IdsList,
};
use crate::has_ability_to_modify;
use crate::r#gen::comp::PasswordPackage;
use crate::r#gen::generic;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub struct CompDeclService {
    token_generator: Arc<Mutex<TokenGenerator>>,
    db_con: Arc<Mutex<Pool<AsyncPgConnection>>>,
}

impl CompDeclService {
    pub(crate) fn new(
        token_generator: Arc<Mutex<TokenGenerator>>,
        db_con: Arc<Mutex<Pool<AsyncPgConnection>>>,
    ) -> Self {
        Self {
            token_generator, //: TokenGenerator::new(),
            db_con,
        }
    }
}

#[tonic::async_trait]
impl CompetitionDeclarator for CompDeclService {
    // TODO: Needs to be optimized!
    async fn declare_competition(
        &self,
        request: Request<CompDeclaration>,
    ) -> Result<Response<DeclareCompetitionResult>, Status> {
        //println!(">>> Declaration:: {:?}", request);
        let (meta, ext, declaration) = request.into_parts();
        if let Some(ext) = ext.get::<TokenClaim>() {
            let organisation = declaration.related_organisation_id;
            let mut ability = false;
            let mut connection = None;
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                ability =
                    has_ability_to_create(&mut conn, ext.user_id, organisation, &ext.perms).await;
                connection = Some(conn);
            }
            //println!("> Abil: {}", ability);
            if let (Some(mut conn), true) = (connection, ability) {
                let transact_res:Result<(i32, PasswordPackage), diesel::result::Error> = conn
                    .transaction(move |conn| {
                        Box::pin(async move {
                            let comp_id = insert_competition(conn, (&declaration).into()).await?.id;
                            insert_comp_data(
                                conn,
                                comp_id,
                                bincode::serialize(&declaration.queues).unwrap_or_default(),
                                bincode::serialize(&declaration.part_list).unwrap_or_default(),
                            )
                            .await?;
                            //println!("> Comp setup");
                            Ok((comp_id, generate_comp_staff(conn, comp_id, &declaration).await?))
                        })
                    })
                    .await;
                match transact_res {
                    Ok((cid, ppg)) => Ok(Response::new(DeclareCompetitionResult {
                        result: Some(IdResult {
                            result: Some(id_result::Result::ObjId(cid)),
                        }),
                        staff: Some(ppg),
                    })),
                    Err(e) => Err(Status::internal("Unable to declare!")),
                    _ => Err(Status::internal("Unable to get staff data!")),
                }
            } else {
                Err(Status::permission_denied("Access denied!"))
            }
        } else {
            Err(Status::unauthenticated("Auth error!"))
        }
    }

    async fn modify_competition(
        &self,
        request: Request<ModCompDeclarationRequest>,
    ) -> Result<Response<GenericResultMessage>, Status> {
        let (meta, ext, req) = request.into_parts();
        //let declaration = req.declaration.unwrap();
        let mut ability = false;
        let mut connection = None;

        let coid = req.comp_id; // Competition ID

        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match comp_mng::get_competition(&mut conn, coid).await {
                    Ok(x) => {
                        let organisation = x.organisation; //declaration.related_organisation_id;
                        ability = has_ability_to_modify(&mut conn, ext.user_id, coid, &ext.perms).await;
                        connection = Some(conn);
                    }
                    Err(e) => {
                        return Err(Status::not_found("Not found"));
                    }
                }
            } else {
                return Err(Status::internal("Database error"));
            }
        }

        if !ability {
            return Err(Status::unauthenticated("Auth required"));
        }

        if let Some(mut connection) = connection {
            match req.command {
                None => Err(Status::unimplemented("Not implemented")),
                Some(comm) => match comm {
                    Command::Redeclare(_) => Err(Status::unimplemented("Not implemented")),
                    Command::SingleCommand(sigle) => match ModDeclarationCommand::try_from(sigle) {
                        Ok(com) => match (com) {
                            ModDeclarationCommand::Delete => {
                                match comp_mng::safe_delete_competition(&mut connection, coid).await
                                {
                                    Ok(_) => Ok(Response::new(GenericResultMessage {
                                        r: GenericResult::Ok as i32,
                                    })),
                                    Err(_) => Err(Status::internal("Unable to delete.")),
                                }
                            }
                            ModDeclarationCommand::RemakeTempPwds => {
                                Err(Status::unimplemented("Not implemented"))
                            }
                        },
                        Err(_) => Err(Status::unimplemented("Not implemented")),
                    },
                },
            }
        } else {
            Err(Status::internal("Database error"))
        }
    }

    async fn get_comps_status(
        &self,
        request: Request<IdsList>,
    ) -> Result<Response<CompsStatusMessage>, Status> {
        todo!()
    }

    async fn get_comps_ids(&self, request: Request<Empty>) -> Result<Response<IdsList>, Status> {
        todo!()
    }

    async fn get_comps_views(
        &self,
        request: Request<IdsList>,
    ) -> Result<Response<CompsList>, Status> {
        let (meta, ext, req) = request.into_parts();

        if let Ok(mut conn) = self.db_con.lock().await.get().await {
            if let Some(ext) = ext.get::<TokenClaim>() {
                Err(Status::internal("NOT IMPLEMENTED"))
            } else {
                // TODO make public visible
                Err(Status::permission_denied(
                    "Not implemented for unauthorized users",
                ))
            }
        } else {
            Err(Status::internal("Database error"))
        }
    }

    async fn get_comp_declaration(
        &self,
        request: Request<crate::gen::generic::Id>,
    ) -> Result<Response<CompDeclaration>, Status> {
        let (meta, ext, req) = request.into_parts();
        //let declaration = req.declaration.unwrap();
        let mut ability = false;
        let mut connection = None;

        let coid = req.id; // Competition ID

        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match comp_mng::get_competition(&mut conn, coid).await {
                    Ok(x) => {
                        let organisation = x.organisation; //declaration.related_organisation_id;
                        ability =
                            has_ability_to_create(&mut conn, ext.user_id, organisation, &ext.perms)
                                .await; // TODO
                        connection = Some(conn);
                    }
                    Err(e) => {
                        return Err(Status::not_found("Not found"));
                    }
                }
            } else {
                return Err(Status::internal("Database error"));
            }
        }

        if !ability {
            return Err(Status::unauthenticated("Auth required"));
        }

        if let Some(mut connection) = connection {
            // bincode::serialize(&declaration.nomination_list).unwrap_or_default(),
            // bincode::serialize(&declaration.part_list).unwrap_or_default()

            match (
                get_competition(&mut connection, coid).await,
                get_comp_data(&mut connection, coid).await,
            ) {
                (Ok(competition), Ok(data)) => {
                    let declaration = CompDeclaration {
                        title: competition.title,
                        public: competition.public,
                        related_organisation_id: competition.organisation,
                        dates: Some(DatePair {
                            begins: into_timestamp(competition.start_date),
                            ends: into_timestamp(competition.ends_date),
                        }),
                        place: competition.place,
                        descr: competition.descr,
                        scheme: competition.scheme,

                        queues: bincode::deserialize(&data.queues).unwrap(),
                        part_list: bincode::deserialize(&data.participants).unwrap(),
                    };
                    Ok(Response::new(declaration))
                }
                _ => Err(Status::not_found("Not found")),
            }
        } else {
            Err(Status::internal("Database error"))
        }
    }

    async fn remake_staff_passwords(
        &self,
        request: tonic::Request<generic::Id>,
    ) -> Result<tonic::Response<PasswordPackage>, Status> {
        let (meta, ext, req) = request.into_parts();
        let coid = req.id;

        if let Some(ext) = ext.get::<TokenClaim>() {
            if let Ok(mut conn) = self.db_con.lock().await.get().await {
                match comp_mng::get_competition(&mut conn, coid).await {
                    Ok(x) => {
                        let organisation = x.organisation; //declaration.related_organisation_id;
                        match has_ability_to_modify(
                            &mut conn,
                            ext.user_id,
                            organisation,
                            &ext.perms,
                        )
                        .await
                        {
                            true => Err(Status::unimplemented("Unimplemented method")),
                            false => Err(Status::permission_denied("Permission Denied")),
                        }
                    }
                    Err(e) => Err(Status::not_found("Not found")),
                }
            } else {
                Err(Status::internal("Database error"))
            }
        } else {
            Err(Status::permission_denied("Auth failed"))
        }

        /*{
            let result = match JudgeScheme::try_from(declaration.scheme) {
                Ok(JudgeScheme::SixSixTwo) => {
                    vec![6, 6, 2]
                }
                Ok(JudgeScheme::FourFourTwo) => {
                    vec![4, 4, 2]
                }
                _ => {
                    vec![]
                }
            };
            //println!("> Scheme: {:?}", result);
            let argon2 = Argon2::default();
            let mut j_users = vec![];
            let mut j_perms = vec![];
            for qid in 1..declaration.queues.len()+1 {
                for (i, &qsize) in result.iter().enumerate() {
                    for u_num in 0..qsize {
                        let name = format!(
                                "o{}c{}q{}g{}u{}{}",
                                declaration.related_organisation_id,
                                comp_id,
                                qid,
                                i,
                                u_num,
                                Alphanumeric.sample_string(&mut rand::thread_rng(), 5)
                            );
                        let pwd: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
                        j_users.push(UserModel {
                            login: name,
                            hash: argon2.hash_password(pwd.as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string(),
                        });
                        j_perms.push((qid as i32, i as i32));
                    }
                }
            }
            let secretary = UserModel {
                login: format!(
                    "o{}c{}sec{}",
                    declaration.related_organisation_id,
                    comp_id,
                    Alphanumeric.sample_string(&mut rand::thread_rng(), 7)
                ),
                hash: argon2.hash_password(Alphanumeric.sample_string(&mut rand::thread_rng(), 20).as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string(),
            };
            let supervisor = UserModel {
                login: format!(
                    "o{}c{}sup{}",
                    declaration.related_organisation_id,
                    comp_id,
                    Alphanumeric.sample_string(&mut rand::thread_rng(), 7)
                ),
                hash: argon2.hash_password(Alphanumeric.sample_string(&mut rand::thread_rng(), 20).as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string(),
            };
            //println!("> Before J. insert");
            let inserted_judges = insert_users(conn, &j_users).await?;
            create_com_staff(conn, &inserted_judges.iter().map(
                |u| {
                    CompStaffLink {
                        uid: u.id,
                        oid: comp_id,
                    }
                }
            ).collect()).await?;
            // TODO: Optimize this pls!
            for (user, perm_qualifier) in inserted_judges.iter().zip(j_perms) {
                setup_org_perms_to_user(
                    conn,
                    user.id,
                    declaration.related_organisation_id,
                    &vec![
                        Permissions::Judge(comp_id, perm_qualifier.0, perm_qualifier.1)
                    ],
                ).await?;
            }
            //println!("> Perms done!");
            let inserted_sups = insert_users(conn, &vec![secretary, supervisor]).await?;
            create_com_staff(conn, &inserted_sups.iter().map(
                |u| {
                    CompStaffLink {
                        uid: u.id,
                        oid: comp_id,
                    }
                }
            ).collect()).await?;
        }*/
    }
}
