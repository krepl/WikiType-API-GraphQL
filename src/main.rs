extern crate log;

use wikitype_api::graphql::{Context, Mutation, Query, Schema};

use dotenv::dotenv;
use warp::{http::Response, Filter};

fn schema() -> Schema {
    Schema::new(Query, Mutation)
}

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

    let state = warp::any().map(move || Context::new());
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

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
