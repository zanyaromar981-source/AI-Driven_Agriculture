use getset::Getters;

use crate::features::briefs::domain::{SourceTitle, SourceUrl};

/// A page the nightly job read while writing a brief.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct BriefSource {
    title: SourceTitle,
    url: SourceUrl,
}

impl BriefSource {
    pub fn new(title: SourceTitle, url: SourceUrl) -> Self {
        Self { title, url }
    }
}
