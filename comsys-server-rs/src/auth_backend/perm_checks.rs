use std::collections::HashMap;

use diesel_async::AsyncPgConnection;

use crate::{auth_backend::prelude::Permissions, db_mng::{comp_mng::get_competition, org_mng::is_owner_of}};

pub struct PermissionComparator(pub Vec<Permissions>);

impl PermissionComparator {
    pub fn contains(&self, perm: &Permissions) -> bool {
        match perm {
            Permissions::Create => self.0.contains(&perm),
            Permissions::Watch(_) => {
                todo!()
            },
            Permissions::Moderator(acc) => {
                self.0.iter().any(
                    |x| {
                        match x {
                            Permissions::Moderator(acc_2) => {
                                acc_2.contains(&acc)
                            },
                            _ => false
                        }
                    }
                )
            },
            Permissions::Administrate => self.0.contains(&perm),
            Permissions::Judge(_, _, _) => self.0.contains(&perm),
            Permissions::Secretary(_) => self.0.contains(&perm),
            Permissions::Arbitor(_, _) => self.0.contains(&perm),
        }
    }
}
pub async fn has_ability_to_create(conn: &mut AsyncPgConnection, uid: i32, oid: i32, perms: &Vec<(i32, Vec<Permissions>)>) -> bool {
    // TODO!
    is_owner_of(conn, uid, oid).await.is_ok() // TODO
}

pub async fn has_ability_to_modify(conn: &mut AsyncPgConnection, uid: i32, coid: i32, perms: &Vec<(i32, Vec<Permissions>)>) -> bool {
    // TODO!
    let competition_result = get_competition(conn, coid).await;

    if let Ok(declaration) = competition_result{
        if perms.iter().filter(|(oid, perms)| {
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
                Permissions::Secretary(_coid) => coid.eq(_coid),
                _ => false,
            }
          }).collect::<Vec<_>>().len() > 0 
        {
            true 
        } else {
            is_owner_of(conn, uid, declaration.organisation).await.is_ok()
        }
    } else {
        false
    }
}

pub async fn has_ability_to_see_view(conn: &mut AsyncPgConnection, uid: i32, coid: i32, perms: &Vec<(i32, Vec<Permissions>)>) -> bool {
    // TODO!
    let competition_result = get_competition(conn, coid).await;

    if let Ok(declaration) = competition_result{
        if perms.iter().filter(|(oid, perms)| {
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
                Permissions::Secretary(_coid) | Permissions::Arbitor(_coid, _)=> coid.eq(_coid),
                Permissions::Judge(_coid, _, _) => coid.eq(_coid),
                _ => false,
            }
          }).collect::<Vec<_>>().len() > 0 
        {
            true 
        } else {
            is_owner_of(conn, uid, declaration.organisation).await.is_ok()
        }
    } else {
        false
    }
}


/*pub async fn clear_perms(perms: &Vec<(i32, Vec<Permissions>)>) -> Vec<(i32, Vec<Permissions>)> {

    let mut cleared = HashMap::new();

    for i in perms.iter() {
        for p in i.1 {
            match p {
                Permissions::Watch(access) => todo!(),
                Permissions::Create => todo!(),
                Permissions::Moderator(access) => todo!(),
                Permissions::Administrate => todo!(),
                Permissions::Judge(coid, queue, markt) => todo!(),
                Permissions::Secretary(coid) => todo!(),
                Permissions::Supervisor(coid) => todo!(),
            }
        }
    }

    todo!()
}*/