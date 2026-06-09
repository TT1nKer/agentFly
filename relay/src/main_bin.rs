use agent_relay::routes;

#[tokio::main]
async fn main() {
    println!("agent-relay starting...");

    let app = routes::create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Relay listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await.unwrap();
}
