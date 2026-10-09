use crate::features::briefs::domain::{BriefError, ZoneSlug};

use super::text;

const REGION: &str = "region";
const MAX_LENGTH: usize = 40;

/// What a brief is about: the whole region, or one district named by its
/// zone slug.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BriefScope(String);

impl BriefScope {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::slug(value, "A brief scope", MAX_LENGTH)?))
    }

    pub fn region() -> Self {
        Self(REGION.to_string())
    }

    pub fn zone(slug: &ZoneSlug) -> Self {
        Self(slug.into())
    }

    pub fn is_region(&self) -> bool {
        self.0 == REGION
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&BriefScope> for String {
    fn from(value: &BriefScope) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_literal_region_is_the_scope_of_the_whole_region() {
        let scope = BriefScope::new("region".to_string()).expect("scope");

        assert!(scope.is_region());
        assert_eq!(scope, BriefScope::region());
    }

    #[test]
    fn a_zone_slug_is_the_scope_of_that_district() {
        let slug = ZoneSlug::new("chamchamal".to_string()).expect("slug");
        let scope = BriefScope::zone(&slug);

        assert!(!scope.is_region());
        assert_eq!(scope.as_str(), "chamchamal");
        assert_eq!(
            scope,
            BriefScope::new("chamchamal".to_string()).expect("scope")
        );
    }

    #[test]
    fn anything_that_is_not_shaped_like_a_slug_is_refused() {
        for bad in ["", "Region", "all zones", "-kalar", &"a".repeat(41)] {
            assert!(BriefScope::new(bad.to_string()).is_err(), "{bad}");
        }
    }
}
