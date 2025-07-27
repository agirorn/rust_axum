use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("axum_tracing_example=debug,tower_http=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // connect to Postgres
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=admin password=admin dbname=app_dev",
        NoTls,
    )
    .await
    .expect("Failed to connect to Postgres");

    // spawn the connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("DB connection error: {}", e);
        }
    });

    let shared_client = Arc::new(client);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .layer(Extension(shared_client.clone()))
        .layer(TraceLayer::new_for_http());

    // Define address and log it
    let port = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(port).await.unwrap();
    info!("🚀 Server listening on http://{}", port);
    axum::serve(listener, app).await.unwrap();
}

// // basic handler that responds with a static string
// async fn root() -> &'static str {
//     "Hello, World!"
// }

async fn root(Extension(db): Extension<Arc<Client>>) -> Result<String, (StatusCode, String)> {
    let row = db
        .query_one("SELECT message FROM greetings LIMIT 1", &[])
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error: {}", e),
            )
        })?;

    let message: String = row.get("message");
    Ok(message)
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
