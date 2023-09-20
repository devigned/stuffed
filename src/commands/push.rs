use std::fs;
use anyhow::Result;
use clap::Args;
use std::path::Path;
use component2oci::Auth;
use crate::commands::{CommonOptions, digest, ensure_dir};

/// Pushes a root component and component layers to an OCI registry
#[derive(Args)]
pub struct PushCommand {
    /// The common command options.
    #[clap(flatten)]
    pub common: CommonOptions,

    /// The URI to the image registry and image. For example: http://registry/namespace/name:latest
    #[clap(long, value_name = "ref")]
    pub reference: String,

    /// The path to the root component.
    #[clap(long, value_name = "root")]
    pub root_path: String,

    /// The additional components needed by the root component.
    #[clap(long, value_name = "deps")]
    pub component_paths: Option<Vec<String>>,

    /// If true, https will be used. If false, http will be used.
    #[clap(long, short)]
    pub insecure: bool,
}

impl PushCommand {
    /// The command to push a component to a registry.
    pub async fn exec(self) -> Result<String> {
        let root_path = Path::new(&self.root_path);
        if !root_path.exists() {
            return Err(anyhow::anyhow!("root path does not exist"));
        }

        ensure_dir(self.common.cache_dir.clone())?;
        let root_bits = fs::read(root_path)?;
        let digest = digest(root_bits.as_slice());
        let digest_string = hex::encode(digest);
        let client = component2oci::client::Client::new(self.insecure, Auth::Anonymous).await;
        let _ = client.push(self.reference.clone(), root_bits, digest_string.clone()).await?;
        println!("pushed: {} with digest: {}", self.reference.clone(), digest_string);
        Ok(self.reference)
    }
}