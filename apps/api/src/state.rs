use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub web_origin: String,
}

impl AppState {
    pub fn new(pool: PgPool, web_origin: String) -> Self {
        Self { pool, web_origin }
    }
}
