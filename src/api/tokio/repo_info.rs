use crate::RepoType;

use super::{Api, ApiError, ApiRepo, HfBadResponse};

/// todo docs
#[derive(Debug)]
pub enum RepoInfo {
    /// Model Variant
    Model(ModelInfo),
    // TODO add dataset and space info
}

impl RepoInfo {
    /// todo docs
    pub fn sha(&self) -> Option<&str> {
        match self {
            RepoInfo::Model(m) => m.sha.as_deref(),
        }
    }
}

impl From<ModelInfo> for RepoInfo {
    fn from(value: ModelInfo) -> Self {
        Self::Model(value)
    }
}

impl ApiRepo {
    /// Get the info object for a given repo.
    pub async fn repo_info(&self) -> Result<RepoInfo, ApiError> {
        //println!("Repo info {:?}", self.repo);

        match self.repo.repo_type {
            RepoType::Model => Ok(self
                .api
                .model_info(&self.repo.repo_id, Some(&self.repo.revision))
                .await?
                .into()),
            RepoType::Dataset =>Ok(self
                .api
                .dataset_info(&self.repo.repo_id, Some(&self.repo.revision))
                .await?
                .into()),
            RepoType::Space => todo!(),
        }
    }

    /// Checks if this repository exists on the Hugging Face Hub.
    pub async fn exists(&self) -> bool {
        match self.repo_info().await {
            Ok(_) => true,
            // no access, but it exists
            Err(ApiError::GatedRepoError(_)) => true,
            Err(ApiError::RepositoryNotFoundError(_)) => false,
            Err(_) => false,
        }
    }

    /// Checks if this repository exists and is writable on the Hugging Face Hub.
    pub async fn is_writable(&self) -> bool {
        if !self.exists().await {
            return false;
        }
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/x-ndjson".parse().unwrap());

        let url = format!(
            "{}/api/{}s/{}/commit/{}",
            self.api.endpoint,
            self.repo.repo_type.to_string(),
            self.repo.url(),
            self.repo.revision
        );

        let res: Result<StatusCode, ApiError> = (async {
            Ok(self
                .api
                .client
                .post(&url)
                .headers(headers)
                .send()
                .await
                .map_err(ApiError::from)?
                .status())
        })
        .await;
        if let Ok(status) = res {
            if status == StatusCode::FORBIDDEN {
                return false;
            }
        }
        true
    }
}

impl Api {
    /// Get info on one specific model on huggingface.co
    ///
    /// Model can be private if you pass an acceptable token or are logged in.
    ///
    /// Args:
    ///     repo_id (`str`):
    ///         A namespace (user or an organization) and a repo name separated
    ///         by a `/`.
    ///     revision (`str`, *optional*):
    ///         The revision of the model repository from which to get the
    ///         information.
    async fn model_info(
        &self,
        repo_id: &str,
        revision: Option<&str>,
    ) -> Result<ModelInfo, ApiError> {
        let url = if let Some(revision) = revision {
            format!(
                "{}/api/models/{repo_id}/revision/{}",
                self.endpoint,
                urlencoding::encode(revision)
            )
        } else {
            format!("{}/api/models/{repo_id}", self.endpoint)
        };

        // TODO add params for security status, blobs, expand, etc.

        let model_info: ModelInfo = self
            .client
            .get(url)
            .send()
            .await?
            .maybe_hf_err()
            .await?
            .json()
            .await?;

        Ok(model_info)
    }

    async fn dataset_info(
        &self,
        repo_id: &str,
        revision: Option<&str>,
    ) -> Result<ModelInfo, ApiError> {
        let url = if let Some(revision) = revision {
            format!(
                "{}/api/datasets/{repo_id}/revision/{}",
                self.endpoint,
                urlencoding::encode(revision)
            )
        } else {
            format!("{}/api/datasets/{repo_id}", self.endpoint)
        };

        // TODO add params for security status, blobs, expand, etc.

        let model_info: ModelInfo = self
            .client
            .get(url)
            .send()
            .await?
            .maybe_hf_err()
            .await?
            .json()
            .await?;

        Ok(model_info)
    }
}

use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
/// todo docs
pub struct ModelInfo {
    /// todo docs
    #[serde(default)]
    pub _id: Option<String>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "modelId")]
    pub model_id: Option<String>,

    /// todo docs
    pub id: String,

    /// todo docs
    #[serde(default)]
    pub author: Option<String>,

    /// todo docs
    #[serde(default)]
    pub sha: Option<String>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "createdAt", alias = "created_at")]
    pub created_at: Option<String>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "lastModified", alias = "last_modified")]
    pub last_modified: Option<String>,

    /// todo docs
    #[serde(default)]
    pub private: Option<bool>,

    /// todo docs
    #[serde(default)]
    pub disabled: Option<bool>,

    /// todo docs
    #[serde(default)]
    pub downloads: Option<i32>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "downloadsAllTime")]
    pub downloads_all_time: Option<i32>,

    /// todo docs
    #[serde(default)]
    pub gated: Option<GatedStatus>,

    /// todo docs
    #[serde(default)]
    pub gguf: Option<HashMap<String, serde_json::Value>>,

    /// todo docs
    #[serde(default)]
    pub inference: Option<InferenceStatus>,

    /// todo docs
    #[serde(default)]
    pub likes: Option<i32>,

    /// todo docs
    #[serde(default)]
    pub library_name: Option<String>,

    /// todo docs
    #[serde(default)]
    pub tags: Option<Vec<String>>,

    /// todo docs
    #[serde(default)]
    pub pipeline_tag: Option<String>,

    /// todo docs
    #[serde(default)]
    pub mask_token: Option<String>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "cardData", alias = "card_data")]
    pub card_data: Option<ModelCardData>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "widgetData")]
    pub widget_data: Option<serde_json::Value>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "model-index", alias = "model_index")]
    pub model_index: Option<HashMap<String, serde_json::Value>>,

    /// todo docs
    #[serde(default)]
    pub config: Option<HashMap<String, serde_json::Value>>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "transformersInfo", alias = "transformers_info")]
    pub transformers_info: Option<TransformersInfo>,

    /// todo docs
    #[serde(default)]
    #[serde(alias = "trendingScore")]
    pub trending_score: Option<i32>,

    /// todo docs
    #[serde(default)]
    pub siblings: Option<Vec<RepoSibling>>,

    /// todo docs
    #[serde(default)]
    pub spaces: Option<Vec<String>>,

    /// todo docs
    #[serde(default)]
    pub safetensors: Option<SafeTensorsInfo>,
}

/// todo docs
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GatedStatus {
    /// todo docs
    Auto,
    /// todo docs
    Manual,
    /// todo docs
    False,
}

impl<'de> Deserialize<'de> for GatedStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GatedStatusVisitor;

        impl<'de> serde::de::Visitor<'de> for GatedStatusVisitor {
            type Value = GatedStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string \"auto\", \"manual\", or boolean false")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !value {
                    Ok(GatedStatus::False)
                } else {
                    Err(E::custom("expected false"))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "auto" => Ok(GatedStatus::Auto),
                    "manual" => Ok(GatedStatus::Manual),
                    _ => Err(E::custom("invalid value")),
                }
            }
        }

        deserializer.deserialize_any(GatedStatusVisitor)
    }
}

/// todo docs
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InferenceStatus {
    /// todo docs
    Warm,
    /// todo docs
    Cold,
    /// todo docs
    Frozen,
}

/// todo docs
#[derive(Debug, Serialize, Deserialize)]
pub struct RepoSibling {
    /// todo docs
    pub rfilename: String,

    /// todo docs
    #[serde(default)]
    pub size: Option<i64>,

    /// todo docs
    #[serde(alias = "blobId")]
    #[serde(default)]
    pub blob_id: Option<String>,

    /// todo docs
    #[serde(default)]
    pub lfs: Option<BlobLfsInfo>,
}

/// todo docs
#[derive(Debug, Serialize, Deserialize)]
pub struct BlobLfsInfo {
    /// todo docs
    pub size: i64,
    /// todo docs
    pub sha256: String,

    /// todo docs
    #[serde(alias = "pointerSize")]
    pub pointer_size: i64,
}

/// todo docs
#[derive(Debug, Serialize, Deserialize)]
pub struct SafeTensorsInfo {
    /// todo docs
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    /// todo docs
    pub total: i64,
}

/// todo docs
// Note: You'll need to implement ModelCardData and TransformersInfo structs separately
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelCardData {
    // Add fields as needed
}

/// todo docs
#[derive(Debug, Serialize, Deserialize)]
pub struct TransformersInfo {
    // Add fields as needed
}
