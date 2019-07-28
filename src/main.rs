extern crate log;

use wikitype_api::graphql::{Context, Mutation, Query, Schema};
use wikitype_api::openid_connect::get_google_oidc_client;

use dotenv::dotenv;
use std::net::SocketAddr;
use warp::filters::log::{Info, Log};
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

fn main() {
    initialize_environment_variables_from_dotenv_file();
    let logging = initialize_logging();
    let service;
    if cfg!(debug_assertions) {
        service = make_graphql_and_graphiql_service().with(logging).boxed();
    } else {
        service = make_graphql_service().with(logging).boxed();
    }
    let socket_address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    listen_and_serve(service, socket_address);
}

fn initialize_environment_variables_from_dotenv_file() {
    dotenv().ok();
}

fn listen_and_serve(service: BoxedFilter<(impl Reply + 'static,)>, socket_address: SocketAddr) {
    log::info!("Listening on {}", socket_address);
    warp::serve(service).run(socket_address);
}

fn initialize_logging() -> Log<impl Fn(Info) + Copy> {
    let default_logging_level = "error";
    let warp_server_logging_target = "warp_server";
    let default_logging_directives = [default_logging_level, warp_server_logging_target].join(",");
    set_environment_variable_if_not_defined("RUST_LOG", &default_logging_directives);
    env_logger::init();
    warp::log(warp_server_logging_target)
}

fn set_environment_variable_if_not_defined(variable_name: &str, value: &str) {
    if !environment_variable_is_defined(variable_name) {
        std::env::set_var("RUST_LOG", value);
    }
}

fn environment_variable_is_defined(variable_name: &str) -> bool {
    match std::env::var(variable_name) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn make_graphql_service() -> BoxedFilter<(impl Reply,)> {
    warp::post2()
        .and(warp::path("graphql"))
        .and(make_graphql_filter())
        .boxed()
}

fn make_graphql_and_graphiql_service() -> BoxedFilter<(impl Reply,)> {
    let graphql_filter = warp::post2()
        .and(warp::path("graphql"))
        .and(make_graphql_filter());
    let graphiql_filter = warp::get2()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql"));
    graphql_filter.or(graphiql_filter).boxed()
}

fn make_graphql_filter() -> BoxedFilter<(impl Reply,)> {
    let graphql_context_extractor = make_graphql_context_extractor();
    let schema = Schema::new(Query, Mutation);
    juniper_warp::make_graphql_filter(schema, graphql_context_extractor)
}

fn make_graphql_context_extractor() -> BoxedFilter<(wikitype_api::graphql::Context,)> {
    let context = Context::new();
    let app_state = warp::any().map(move || context.clone());
    warp::any()
        .and(warp::header::optional::<String>("authorization"))
        .and(app_state)
        .map(get_user_from_bearer_token)
        .boxed()
}

// TODO: DELETE ME
#[derive(Debug)]
pub struct MyClaims {
    pub iss: url::Url,
    pub sub: String,
    pub aud: biscuit::SingleOrMultiple<String>,
    pub exp: i64,
    pub iat: i64,
    pub auth_time: Option<i64>,
    pub nonce: Option<String>,
    pub acr: Option<String>,
    pub amr: Option<Vec<String>>,
    pub azp: Option<String>,
}

impl MyClaims {
    pub fn from_claims(token: &oidc::token::Claims) -> MyClaims {
        MyClaims {
            iss: token.iss.clone(),
            sub: token.sub.clone(),
            aud: token.aud.clone(),
            exp: token.exp.clone(),
            iat: token.iat.clone(),
            auth_time: token.auth_time.clone(),
            nonce: token.nonce.clone(),
            acr: token.acr.clone(),
            amr: token.amr.clone(),
            azp: token.azp.clone(),
        }
    }
}

// TODO: add error-handling logic
fn get_user_from_bearer_token(authorization_header: Option<String>, app_state: Context) -> Context {
    if let Some(id_token) = authorization_header {
        let prefix = "Bearer ";
        if id_token.starts_with(prefix) {
            let id_token = &id_token[prefix.len()..];
            let mut id_token: oidc::token::Jws<oidc::token::Claims, biscuit::Empty> =
                oidc::token::Jws::new_encoded(id_token);
            let google_client = get_google_oidc_client();
            google_client.decode_token(&mut id_token).unwrap();
            remove_nonce_from_id_token_if_present(&mut id_token);
            google_client.validate_token(&id_token, None, None).unwrap();
            let token = id_token.payload().unwrap();
            println!("{:#?}", MyClaims::from_claims(&token));
        }
    }
    app_state
}

// NOTE: The nonce needs to be removed because otherwise token validation will fail since we don't
// have an original nonce to which we can compare the token's nonce.
fn remove_nonce_from_id_token_if_present(
    id_token: &mut oidc::token::Jws<oidc::token::Claims, biscuit::Empty>,
) {
    if let &mut oidc::token::Jws::Decoded {
        header: _,
        ref mut payload,
    } = id_token
    {
        payload.nonce = None;
    }
}
