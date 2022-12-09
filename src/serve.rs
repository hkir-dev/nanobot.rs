use crate::{get, sql};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Params {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    // TODO: this is a hack to allow for one PostgREST-style column filter
    pub table: Option<String>,
}

#[tokio::main]
pub async fn main() -> Result<String, String> {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/:table", get(table));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    let hello = String::from("Hello, world!");
    Ok(hello)
}

async fn root() -> impl IntoResponse {
    tracing::info!("request root");
    Redirect::permanent("/table")
}

async fn table(Path(table): Path<String>, params: Query<Params>) -> impl IntoResponse {
    tracing::info!("request table {:?} {:?}", table, params.0);
    let select = sql::Select {
        table,
        limit: params.limit.unwrap_or_default(),
        offset: params.offset.unwrap_or_default(),
        // TODO: restore filters
        ..Default::default()
    };
    match get::get_rows(".nanobot.db", &select, "page", "html").await {
        Ok(html) => (StatusCode::FOUND, Html(html)),
        Err(_) => (StatusCode::NOT_FOUND, Html("404 Not Found".to_string())),
    }
}
