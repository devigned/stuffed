mod push;
mod pull;

use anyhow::Result;
use std::fs;
use std::path::Path;
use clap::Args;
use sha2::{Sha256, Digest};

pub use self::push::*;
pub use self::pull::*;

/// Common options for commands.
#[derive(Args)]
pub struct CommonOptions {
    /// The temporary directory where to cache pulled images.
    #[clap(long, short, value_name = "KEY_NAME", default_value = "~/.stuffed/")]
    pub cache_dir: String,
}

fn digest(content_bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(content_bytes);
    hasher.finalize().to_vec()
}

fn ensure_dir(dir_path: String) -> Result<()> {
    let path = Path::new(&dir_path);
    if !path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())
}