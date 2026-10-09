use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{features::versions::domain::DataVersion, infra::http::API_VERSION};

/// The current version of each kind of data. A site compares these with the
/// numbers it remembers and fetches again only what went up.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct VersionsResponse {
    /// Changes when the shape of any answer changes: throw the cache away.
    pub api: String,
    pub versions: BTreeMap<String, i64>,
    pub server_time: DateTime<Utc>,
}

impl From<&[DataVersion]> for VersionsResponse {
    fn from(versions: &[DataVersion]) -> Self {
        Self {
            api: API_VERSION.to_string(),
            versions: versions
                .iter()
                .map(|version| (version.topic().as_str().to_string(), *version.version()))
                .collect(),
            server_time: Utc::now(),
        }
    }
}
