use chrono::{DateTime, Utc};
use getset::Getters;

/// A kind of data a website caches as one piece. Its version goes up by one
/// every time anything of that kind is written, so a site that remembers the
/// number can tell, with one small call, what it must fetch again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Topic {
    Zones,
    SubZones,
    Dams,
    Fires,
    Outlooks,
    Water,
    AlwaPrices,
    AlwaListings,
    Crops,
    Rules,
    Briefs,
    AppConfig,
    Farmers,
    Farms,
    Messages,
    Jobs,
    StaffRoles,
}

impl Topic {
    pub const ALL: [Topic; 17] = [
        Topic::Zones,
        Topic::SubZones,
        Topic::Dams,
        Topic::Fires,
        Topic::Outlooks,
        Topic::Water,
        Topic::AlwaPrices,
        Topic::AlwaListings,
        Topic::Crops,
        Topic::Rules,
        Topic::Briefs,
        Topic::AppConfig,
        Topic::Farmers,
        Topic::Farms,
        Topic::Messages,
        Topic::Jobs,
        Topic::StaffRoles,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::Zones => "zones",
            Topic::SubZones => "sub_zones",
            Topic::Dams => "dams",
            Topic::Fires => "fires",
            Topic::Outlooks => "outlooks",
            Topic::Water => "water",
            Topic::AlwaPrices => "alwa_prices",
            Topic::AlwaListings => "alwa_listings",
            Topic::Crops => "crops",
            Topic::Rules => "rules",
            Topic::Briefs => "briefs",
            Topic::AppConfig => "app_config",
            Topic::Farmers => "farmers",
            Topic::Farms => "farms",
            Topic::Messages => "messages",
            Topic::Jobs => "jobs",
            Topic::StaffRoles => "staff_roles",
        }
    }

    pub fn parse(value: &str) -> Option<Topic> {
        Topic::ALL.into_iter().find(|topic| topic.as_str() == value)
    }

    /// Private topics hold people's data or staff matters. Even the fact
    /// that they changed is shown to signed-in staff only.
    pub fn is_private(&self) -> bool {
        matches!(
            self,
            Topic::Farmers | Topic::Farms | Topic::Messages | Topic::Jobs | Topic::StaffRoles
        )
    }
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct DataVersion {
    topic: Topic,
    version: i64,
    changed_at: DateTime<Utc>,
}

impl DataVersion {
    /// Reconstruct from persisted state.
    pub fn rehydrate(topic: Topic, version: i64, changed_at: DateTime<Utc>) -> Self {
        Self {
            topic,
            version,
            changed_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_topic_round_trips_through_its_stored_name() {
        for topic in Topic::ALL {
            assert_eq!(Topic::parse(topic.as_str()), Some(topic));
        }
    }

    #[test]
    fn an_unknown_name_is_no_topic() {
        assert_eq!(Topic::parse("everything"), None);
        assert_eq!(Topic::parse("Zones"), None);
    }

    #[test]
    fn topics_about_people_and_staff_are_private() {
        for topic in [
            Topic::Farmers,
            Topic::Farms,
            Topic::Messages,
            Topic::Jobs,
            Topic::StaffRoles,
        ] {
            assert!(
                topic.is_private(),
                "{topic:?} must not be shown without sign-in"
            );
        }

        assert!(!Topic::Fires.is_private());
        assert!(!Topic::Zones.is_private());
    }
}
