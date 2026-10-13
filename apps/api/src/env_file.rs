use std::env;
use std::fs;
use std::path::PathBuf;

fn same_file(a: &PathBuf, b: &PathBuf) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Load repo-root `.env`, then `apps/api/.env`, then cwd `.env` when it is a different file.
pub fn load_dotenv() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf());
    let root_env = root.as_ref().map(|r| r.join(".env"));
    let api_env = manifest.join(".env");

    if let Some(ref path) = root_env {
        let _ = dotenvy::from_path(path);
    }

    let _ = dotenvy::from_path_override(api_env);

    if let (Ok(cwd), Some(ref root_path)) = (env::current_dir(), root_env.as_ref()) {
        let cwd_env = cwd.join(".env");
        if cwd_env.exists() && !same_file(&cwd_env, root_path) {
            let _ = dotenvy::from_path_override(cwd_env);
        }
    }
}
