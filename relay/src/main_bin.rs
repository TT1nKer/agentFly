use agent_relay::db;
use agent_relay::routes;

#[tokio::main]
async fn main() {
    println!("agent-relay starting...");

    let db = db::RelayDb::open("relay.db").expect("Failed to open relay database");
    println!("Database initialized.");

    let app = routes::create_router(db);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Relay listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await.unwrap();
}
