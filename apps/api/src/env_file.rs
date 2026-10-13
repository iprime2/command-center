use std::env;
use std::path::PathBuf;

/// Load repo-root `.env`, then `apps/api/.env`, then cwd `.env` (each layer overrides).
pub fn load_dotenv() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    if let Some(root) = manifest.parent().and_then(|p| p.parent()) {
        let _ = dotenvy::from_path(root.join(".env"));
    }

    let _ = dotenvy::from_path_override(manifest.join(".env"));

    if let Ok(cwd_env) = env::current_dir().map(|d| d.join(".env")) {
        if cwd_env.exists() {
            let _ = dotenvy::from_path_override(cwd_env);
        }
    }
}
