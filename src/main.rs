use anyhow::Result;

pub mod azure;
pub mod azureai;
pub mod routes;
pub mod tests;
pub mod utils;
pub mod tools;
pub mod local;

use tracing::Level;
use warp;
use warp::Filter;

const MAXIMUM_UPLOAD_SIZE: u64 = 100;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    tokio::fs::create_dir_all("./upload_path").await?;

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["Content-Type", "User-Agent", "Authorization"])
        .build();

    let home_route = warp::path::end().and_then(routes::home);

    let app_root = warp::path!("api");

    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .and_then(routes::ws_handler);

    let create_table = warp::path("create_table")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(routes::create_new_table);

    let delete_table = warp::path("delete_table")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(routes::delete_table);

    let search_bases = warp::path("search_bases")
        .and(warp::post())
        .and_then(routes::get_all_search_base);

    let handle_upload = warp::path("handle_upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(1024 * 1024 * MAXIMUM_UPLOAD_SIZE))
        .and_then(routes::handle_upload);

    let app_routes = home_route
        .or(app_root
            .and_then(routes::home)
            .or(websocket_route)
            .or(create_table)
            .or(delete_table)
            .or(search_bases)
            .or(handle_upload))
        .with(cors);

    warp::serve(app_routes).run(([0, 0, 0, 0], 8000)).await;

    Ok(())
}
