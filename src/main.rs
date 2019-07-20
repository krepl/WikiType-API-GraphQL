extern crate log;

#[macro_use]
extern crate serde_derive;

use wikitype_api::graphql::{Context, Mutation, Query, Schema};
use wikitype_api::openid_connect::{get_google_oauth2_certificate_der, IdToken};

use dotenv::dotenv;
use warp::{http::Response, Filter};

fn schema() -> Schema {
    Schema::new(Query, Mutation)
}

#[derive(Debug, Serialize, Deserialize)]
struct Blah {}

// TODO: Improve the endpoint structure / implementation.
// NOTE: The current implementation is copied verbatim from the following juniper example.
// https://github.com/graphql-rust/juniper/blob/master/juniper_warp/examples/warp_server.rs
fn main() {
    dotenv().ok();

    // Set the RUST_LOG environment variable if not already set.
    if let Err(_) = std::env::var("RUST_LOG") {
        // Enable error-logging AND the warp_server logging target.
        std::env::set_var("RUST_LOG", "error,warp_server");
    }
    env_logger::init();

    let log = warp::log("warp_server");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });

    log::info!("Listening on 127.0.0.1:8080");

    let app_state = warp::any().map(move || Context::new());

    let graphql_context_extractor = warp::any()
        .and(warp::header::optional::<String>("authorization"))
        .and(app_state)
        .map(|auth_header: Option<String>, app_state: Context| {
            if let Some(id_token) = auth_header {
                let prefix = "Bearer ";
                if id_token.starts_with(prefix) {
                    let id_token = &id_token[prefix.len()..];
                    let id_token: biscuit::jws::Compact<
                        biscuit::ClaimsSet<IdToken>,
                        biscuit::jws::Header<Blah>,
                    > = biscuit::jws::Compact::new_encoded(id_token);
                    let token_header = id_token.unverified_header().unwrap();
                    println!("Header: {:#?}", token_header);
                    let google_key = get_google_oauth2_certificate_der();
                    let signature_algorithm = token_header.registered.algorithm;
                    let id_token = id_token
                        .into_decoded(&google_key, signature_algorithm)
                        .unwrap();
                    // TODO: id_token.validate
                    let (_, id_token_claims): (_, biscuit::ClaimsSet<_>) =
                        id_token.unwrap_decoded();
                    println!("Claims: {:#?}", id_token_claims);
                }
            }
            app_state
        })
        .boxed();

    let graphql_filter = juniper_warp::make_graphql_filter(schema(), graphql_context_extractor);

    warp::serve(
        warp::get2()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql"))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log),
    )
    .run(([127, 0, 0, 1], 8080));
}
