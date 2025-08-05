mod error;
use axum::extract::Request;
use axum::{
    extract::Extension,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use deadpool_postgres::{Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::NoTls;
// use tokio_postgres::{Client, NoTls};
use axum::response::{IntoResponse, Response};
use error::Result;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// #[tokio::main]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("axum_tracing_example=debug,tower_http=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // // connect to Postgres
    // let (client, connection) = tokio_postgres::connect(
    //     "host=localhost user=admin password=admin dbname=app_dev",
    //     NoTls,
    // )
    // .await
    // .expect("Failed to connect to Postgres");

    // // spawn the connection task
    // tokio::spawn(async move {
    //     if let Err(e) = connection.await {
    //         eprintln!("DB connection error: {}", e);
    //     }
    // });

    // let shared_client = Arc::new(client);

    // Set up the Deadpool config
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("admin".to_string());
    cfg.password = Some("admin".to_string());
    cfg.dbname = Some("app_dev".to_string());
    cfg.keepalives = Some(true);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Clean,
    });
    cfg.pool = Some(PoolConfig::new(60));

    // Create connection pool
    let pool: Pool = cfg.create_pool(None, NoTls).expect("Could not create pool");
    // This sets the maximum size (like MaxPoolSize = 100)
    // pool.resize(60); // Min = 0, Max = 100 (matches Npgsql default)
    for _ in 0..5 {
        let _ = pool.get().await.unwrap();
    }
    assert_eq!(pool.status().max_size, 60);
    let shared_client = Arc::new(pool);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        // .layer(Extension(shared_client))
        .layer(Extension(shared_client))
        // .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
        .fallback(handler_404);

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

async fn handler_404(request: Request) -> impl IntoResponse {
    let path = request.uri().path();
    (
        StatusCode::NOT_FOUND,
        Json(format!("nothing to see here {}", path)),
    )
}

#[axum::debug_handler]
async fn root(Extension(pool): Extension<Arc<Pool>>) -> Result<Response> {
    let db = pool.get().await?;
    let row = db
        .query_one("SELECT message FROM greetings LIMIT 1", &[])
        .await?;

    let message: String = row.get("message");
    Ok((StatusCode::OK, message).into_response())
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
