use command_center_api::config::Config;
use command_center_api::state::AppState;
use command_center_api::{app, test_support};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "command_center_api=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env().map_err(|e| {
        eprintln!("config error: {e}");
        e
    })?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    test_support::migrate(&pool).await;

    let state = AppState::new(pool, config.web_origin.clone());
    let router = app(state);
    let addr = config.bind_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{addr}");
    tracing::info!("openapi ui at http://{addr}/swagger-ui/");

    axum::serve(listener, router).await?;
    Ok(())
}
