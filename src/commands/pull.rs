use anyhow::Result;
use std::path::PathBuf;
use clap::Args;
use component2oci::Auth;
use crate::commands::{CommonOptions, ensure_dir};

/// Pulls a root component and component layers from an OCI registry
#[derive(Args)]
pub struct PullCommand {
    /// The common command options.
    #[clap(flatten)]
    pub common: CommonOptions,

    /// The URI to the image registry and image. For example: registry/namespace/name:v1.0.0
    #[clap(long, value_name = "ref")]
    pub reference: String,

    /// The path to the root component.
    #[clap(long, default_value = ".")]
    pub output_directory: String,

    /// If true, https will be used. If false, http will be used.
    #[clap(long, short)]
    pub insecure: bool,
}

impl PullCommand {
    /// The command to pull a component from a registry.
    pub async fn exec(self) -> Result<String> {
        ensure_dir(self.output_directory.clone())?;
        ensure_dir(self.common.cache_dir.clone())?;

        let client = component2oci::client::Client::new(self.insecure, Auth::Anonymous).await;
        let content_path = client.pull(self.reference,  PathBuf::from(self.output_directory)).await?;
        println!("pulled: {}", content_path);
        Ok(content_path)
    }
}
