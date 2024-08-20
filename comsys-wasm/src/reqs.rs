use std::error::Error;
// utf-8
use tonic::{IntoRequest, Request, Response, Status};
use yew::{Callback, UseReducerHandle};
use crate::components::comps::{CompetitionDeclarationWrapper, CompetitionDeclarationWrapperContextAction};
use crate::context::{Context, ContextAction, GlobalContext};
use crate::grpc::auth::{auth_result, AuthFailError, AuthRequest, AuthResult, DropTokenRequest, GetAuthTokenRequest, RegisterRequest, Token, TokenType};
use crate::grpc::generic;

/// Подгрузка AccessToken по логи-паролю + автоматический запрос на AuthToken
pub fn get_access_shortcut(
    ctx: GlobalContext, login: String, password: String,
    on_finish: Callback<Result<(), Status>, ()>
) {
    wasm_bindgen_futures::spawn_local(
        {
            async move {
                let mut grpc_client = Context::get_auth_grpc_client();
                let resp = grpc_client.authorize(
                    AuthRequest {
                        login,
                        password,
                    }
                ).await;
               match resp {
                   Ok(resp) => { get_auth_shortcut(ctx, on_finish) },
                   Err(status) => { on_finish.emit(Err(status)) }

               }
            }
        }
    );
}

pub fn registration_shortcut(
    ctx: GlobalContext, login: String, password: String, supervisor_code: String,
    on_finish: Callback<Result<(String, String), Status>, ()>
) {
    wasm_bindgen_futures::spawn_local(
        {
            async move {
                let mut grpc_client = Context::get_auth_grpc_client();
                let resp = grpc_client.registration(
                    RegisterRequest {
                        auth_req: Some(
                            AuthRequest {
                                login: login.clone(),
                                password: password.clone(),
                            }
                        ),
                        supervisor_code: Some(supervisor_code)
                    }
                ).await;
               match resp {
                   Ok(resp) => {on_finish.emit(Ok((login, password))) },
                   Err(status) => { on_finish.emit(Err(status)) }

               }
            }
        }
    );
}


/// Запрашивает получение Auth токена и устанавливает данные о нем в контекст
pub fn get_auth_shortcut(ctx: GlobalContext, on_finish: Callback<Result<(), Status>, ()>) {
    wasm_bindgen_futures::spawn_local(async move {
        let mut grpc_client = Context::get_auth_grpc_client();
        let req = Request::new(
            GetAuthTokenRequest { access_token: None }
        ); // set Cookie by browser
        let resp_auth = grpc_client.get_auth(
            req
        ).await;
        //web_sys::console::log_1(&format!("{:?}", resp_auth).into());

        match resp_auth {
            Ok(rauth) => {
                let inner = rauth.into_inner();
                match inner.result {
                    Some(auth_result::Result::Token(token)) => {
                        ctx.dispatch(ContextAction::SetupAuth(token));
                        on_finish.emit(Ok(()));
                    },
                    Some(auth_result::Result::Error(ec)) => {
                        web_sys::console::log_1(&"No Auth token in response!".to_string().into());
                        on_finish.emit(Err(Status::unknown(AuthFailError::try_from(ec).unwrap().as_str_name().to_string())))
                    },
                    None => {
                        web_sys::console::log_1(&"No Auth token in response!".to_string().into());
                        on_finish.emit(Err(Status::unknown("Неизвестная ошибка сервиса!".to_string())));
                    }
                }
            }
            Err(status) => {
                web_sys::console::log_1(&"Auth request error".to_string().into());
                on_finish.emit(Err(status));
            }
        }
    });
}

/// Уведомляет сервер о желании выйти из аккаунта и очищает контекст при полодительном ответе
pub fn drop_me_shortcut(ctx: GlobalContext) {
    wasm_bindgen_futures::spawn_local(async move {
        let mut grpc_client = Context::get_auth_grpc_client();
        let req = Request::new(
            DropTokenRequest {
                auth_token: ctx.user.get_token().clone(),
                to_drop: Some(Token { value: "".to_string(), token_type: Some(TokenType::Access as i32) })
                // не имеет значения, так как сервер выбирает Access Token по сопоставлению с Auth.
                // или сервер сам с кукис работает :3
            }
        );
        let resp_drop = grpc_client.drop_token(
            req
        ).await;
        //web_sys::console::log_1(&format!("{:?}", resp_drop).into());
        if let Ok(dropped) = resp_drop {
            let inner = dropped.into_inner();

            if inner.is_done {
                ctx.dispatch(ContextAction::DropAuth);
            } else {
                web_sys::console::log_1(&"Access drop not done!".to_string().into());
            }

        } else {
            web_sys::console::log_1(&"Drop Access request error".to_string().into());
        }
    });
}

pub fn get_competition_decl_shortcut(ctx: UseReducerHandle<Context>, compstate: UseReducerHandle<CompetitionDeclarationWrapper>, cid:i32) {
    wasm_bindgen_futures::spawn_local(
        {
            let ctx = ctx.clone();
            let compstate = compstate.clone();
            async move {
                if ctx.user.get_token().is_some() {
                    let mut client = Context::get_comp_grpc_client(&ctx);
                    web_sys::console::log_1(&"Loading Declaration..".to_string().into());
                    match (client.get_comp_declaration(
                        generic::Id{id: cid}.into_request()
                    ).await) {
                        Ok(resp) => {
                            compstate.dispatch(CompetitionDeclarationWrapperContextAction::Setup(resp.into_parts().1.to_owned()));
                        }
                        Err(e) => {
                            web_sys::console::log_1(&"Unable to request data!".into());
                        }
                    };
                }
            }
        }
    );
}