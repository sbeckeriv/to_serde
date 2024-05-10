use axum::{
    response::Html,
    routing::{get, get_service},
    Router,
};
use std::env;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Set up the routes
    let app = Router::new().route("/", get(serve_index)).nest(
        "/pkg",
        get_service(ServeDir::new("./pkg")).handle_error(handle_error),
    );
    async fn serve_index() -> Html<String> {
        let index_html = tokio::fs::read_to_string("examples/index.html")
            .await
            .unwrap_or_else(|_| "Failed to read index.html".to_string());
        Html(index_html)
    }

    // Run the server
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: std::io::Error) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong...",
    )
}
