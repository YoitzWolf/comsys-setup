// utf-8
use tonic::Request;
use crate::context::{Context, ContextAction, GlobalContext};
use crate::grpc::auth::{auth_result, AuthRequest, DropTokenRequest, GetAuthTokenRequest, Token, TokenType};

/// Подгрузка AccessToken по логи-паролю + автоматический запрос на AuthToken
pub fn get_access_shortcut(ctx: GlobalContext, login: String, password: String) {
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
                if resp.is_ok() {
                    get_auth_shortcut(ctx);
                }
            }
        }
    );
}


/// Запрашивает получение Auth токена и устанавливает данные о нем в контекст
pub fn get_auth_shortcut(ctx: GlobalContext) {
    wasm_bindgen_futures::spawn_local(async move {
        let mut grpc_client = Context::get_auth_grpc_client();
        let req = Request::new(
            GetAuthTokenRequest { access_token: None }
        ); // set Cookie by browser
        let resp_auth = grpc_client.get_auth(
            req
        ).await;
        //web_sys::console::log_1(&format!("{:?}", resp_auth).into());
        if let Ok(rauth) = resp_auth {
            let inner = rauth.into_inner();

            if let Some(auth_result::Result::Token(token)) = inner.result {
                ctx.dispatch(ContextAction::SetupAuth(token));
            } else {
                web_sys::console::log_1(&"No Auth token in response!".to_string().into());
            }
        } else {
            web_sys::console::log_1(&"Auth request error".to_string().into());
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