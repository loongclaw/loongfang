//! Run with
//!
//! ```not_rust
//! sqlx database create
//! sqlx migrate run --source ./examples
//! # With default features
//! cargo run --example demo
//! # Or without features
//! cargo run --example demo --no-default-features
//! ```
//!
//! Test with curl:
//!
//! ```not_rust
//! curl 127.0.0.1:8000
//! curl -X POST -H 'Content-Type:application/json' -d '{"username":"loongfang"}' 127.0.0.1:8000/users
//! ```

mod handler {
    use axum::extract::Json;
    use loongfang::{AppResult, validation::ValidatedJson};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[cfg(feature = "postgres")]
    use loongfang::postgres;

    #[cfg(feature = "redis")]
    use ::redis::AsyncCommands;

    #[cfg(feature = "redis")]
    use loongfang::redis;

    #[derive(Deserialize, Validate)]
    pub struct CreateUser {
        #[validate(length(min = 1, message = "Can not be empty"))]
        pub username: String,
    }

    #[derive(Serialize)]
    pub struct User {
        pub id: i64,
        pub username: String,
    }

    pub async fn root() -> AppResult<String> {
        #[cfg(feature = "redis")]
        {
            let mut con = redis::conn().await?;
            let _: () = con
                .set_ex("greeting", "Hello, Loongfang with Redis!", 10)
                .await?;
            let result: String = con.get("greeting").await?;
            Ok(result)
        }
        #[cfg(not(feature = "redis"))]
        Ok("Hello, Loongfang without Redis!".to_string())
    }

    pub async fn create_user(
        ValidatedJson(payload): ValidatedJson<CreateUser>,
    ) -> AppResult<Json<User>> {
        #[cfg(feature = "postgres")]
        {
            let user = sqlx::query_as!(
                User,
                r#"insert into users (username) values ($1) returning id, username"#,
                payload.username
            )
            .fetch_one(postgres::conn())
            .await?;
            Ok(Json(user))
        }
        #[cfg(not(feature = "postgres"))]
        {
            let user = User {
                id: 9527,
                username: payload.username,
            };
            Ok(Json(user))
        }
    }
}

mod route {
    use crate::handler;
    use axum::{
        Router,
        routing::{get, post},
    };
    use loongfang::middleware::{compression, cors, request_id, trace, trace_body};
    use tower::ServiceBuilder;

    pub fn init() -> Router {
        Router::new()
            .route("/", get(handler::root))
            .route("/users", post(handler::create_user))
            .layer(
                ServiceBuilder::new()
                    .layer(compression::compression())
                    .layer(request_id::set_request_id())
                    .layer(request_id::propagate_request_id())
                    .layer(trace::trace())
                    .layer(cors::cors())
                    .layer(trace_body::trace_body()),
            )
    }
}

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let _worker_guard = loongfang::bootstrap::Application::default("config.toml")?
        .with_router(route::init)
        .before_run(|| {
            tokio::spawn(async move {
                println!("Running pre-run initialization tasks...");
                Ok(())
            })
        })
        .run()
        .await?;
    Ok(())
}
