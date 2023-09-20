use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use oci_distribution::config::{Architecture, Config as DistConfig, ConfigFile, Os};
use oci_distribution::{
    client,
    client::{ClientProtocol, Config, ImageLayer},
    manifest::OciImageManifest,
    secrets::RegistryAuth,
    Reference,
};
use serde_json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tokio::task::block_in_place;
use crate::Auth;

const COMPONENT_ARTIFACT_TYPE: &str = "application/vnd.bytecodealliance.component.v1+wasm";
const WASM_LAYER_MEDIA_TYPE: &str = "application/vnd.bytecodealliance.wasm.component.layer.v0+wasm";
// const COMPONENT_COMPOSE_MANIFEST_MEDIA_TYPE: &str = "application/vnd.bytecodealliance.component.compose.v0+yaml";

/// Client for interacting with an OCI registry
pub struct Client {
    oci_client: Arc<RwLock<oci_distribution::Client>>,
    auth: RegistryAuth,
}

impl Client {
    /// Create a new instance of an OCI client for storing components.
    pub async fn new(
        insecure: bool,
        auth: Auth,
    ) -> Self {
        let client = oci_distribution::Client::new(Self::build_config(insecure));
        Self {
            oci_client: Arc::new(RwLock::new(client)),
            auth,
        }
    }

    pub async fn pull(
        &self,
        reference: impl AsRef<str>,
        output_dir: PathBuf,
    ) -> Result<String> {
        let reference: Reference = reference
            .as_ref()
            .parse()
            .with_context(|| format!("cannot parse reference {}", reference.as_ref()))?;

        // TODO: fix the higher-level lifetime error that occurs when not using block_in_place and
        // block_on.
        let result = block_in_place(|| {
            Handle::current().block_on(async move {
                let mut oci = self.oci_client.write().await;
                oci.pull(&reference, &self.auth, vec![WASM_LAYER_MEDIA_TYPE])
                    .await
            })
        });

        let image = result?;

        // TODO (dj): We should make this work with many layers, not just 1 layer.
        let layer = image
            .layers.first().unwrap();
        let content_path = Self::output_path(output_dir, layer.sha256_digest());
        let mut file = File::create(content_path.clone())
            .await?;
        file.write_all(&layer.data)
            .await?;
        Ok(content_path.to_string_lossy().to_string())
    }

    /// Push a component to an OCI registry.
    pub async fn push(
        &self,
        reference: impl AsRef<str>,
        file_contents: Vec<u8>,
        digest: String,
    ) -> Result<String> {
        let reference: Reference = reference
            .as_ref()
            .parse()
            .with_context(|| format!("cannot parse reference {}", reference.as_ref()))?;

        let entrypoint = format!("/{}", digest);
        let config = ConfigFile {
            architecture: Architecture::Wasm,
            os: Os::Wasi,
            config: Some(DistConfig {
                // use the sha256 hash as the file name for the entrypoint
                entrypoint: vec![entrypoint],
                ..Default::default()
            }),
            ..Default::default()
        };
        let config_data =
            serde_json::to_vec(&config)?;
        let oci_config = Config::oci_v1(config_data, None);
        let mut layers = Vec::new();
        let wasm_layer = Self::wasm_layer(file_contents)
            .await
            .context("cannot create wasm layer")?;
        layers.insert(0, wasm_layer);
        let mut manifest = OciImageManifest::build(&layers, &oci_config, None);
        manifest.artifact_type = Some(COMPONENT_ARTIFACT_TYPE.to_string());

        // TODO: fix the higher-level lifetime error that occurs when not using block_in_place and
        // block_on.
        let result = block_in_place(|| {
            Handle::current().block_on(async move {
                tracing::log::trace!("Pushing component to {:?}", reference);
                let mut oci = self.oci_client.write().await;
                oci.push(&reference, &layers, oci_config, &self.auth, Some(manifest))
                    .await
            })
        });

        result
            .map(|push_response| push_response.manifest_url)
            .context("cannot push component to the registry")
    }

    pub async fn content_exists(
        &self,
        reference: impl AsRef<str>,
    ) -> Result<bool> {
        let reference: Reference = reference
            .as_ref()
            .parse()
            .with_context(|| format!("cannot parse reference {}", reference.as_ref()))
            .unwrap();

        let mut oci = self.oci_client.write().await;
        match oci.fetch_manifest_digest(&reference, &self.auth).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Create a new wasm layer based on a file.
    async fn wasm_layer(file_contents: Vec<u8>) -> Result<ImageLayer> {
        tracing::log::trace!("Reading wasm component of length {:?}", file_contents.len());

        Ok(ImageLayer::new(
            file_contents,
            WASM_LAYER_MEDIA_TYPE.to_string(),
            None,
        ))
    }

    /// Returns the path to the content file for a given content address.
    fn output_path(output_dir: PathBuf, digest: String) -> PathBuf {
        output_dir.join(Self::content_file_name(digest))
    }

    /// Returns the file name for a given content address replacing colons with dashes.
    fn content_file_name(digest: String) -> String {
        digest.replace(':', "-")
    }

    /// Build the OCI client configuration given the insecure option.
    fn build_config(insecure: bool) -> client::ClientConfig {
        let protocol = if insecure {
            ClientProtocol::Http
        } else {
            ClientProtocol::Https
        };

        client::ClientConfig {
            protocol,
            ..Default::default()
        }
    }
}