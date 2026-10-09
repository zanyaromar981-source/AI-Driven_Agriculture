use chrono::{DateTime, Utc};
use getset::Getters;

use crate::features::app_config::domain::{AppConfigError, AppVersion, HelpPhone, NoticeText};

/// The parts of the app that can be switched off from the dashboard without
/// a release. The app reads them at start; they hide a screen, they do not
/// close its routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    pub add_farm: bool,
    pub walk_mode: bool,
    pub satellite: bool,
    pub doctor: bool,
    pub reports: bool,
    pub alwa: bool,
    pub plan: bool,
    pub push: bool,
}

/// The limits the app applies on the phone before it uploads anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Limits {
    farms_per_phone: u32,
    max_farm_dunam: u32,
    min_corners: u32,
    max_corners: u32,
    gps_meters: u32,
}

impl Limits {
    /// Three corners is the fewest that enclose any ground.
    const FEWEST_CORNERS: u32 = 3;

    pub fn new(
        farms_per_phone: u32,
        max_farm_dunam: u32,
        min_corners: u32,
        max_corners: u32,
        gps_meters: u32,
    ) -> Result<Self, AppConfigError> {
        let bad = |detail: &str| Err(AppConfigError::BadLimits(detail.to_string()));

        if farms_per_phone == 0 {
            return bad("farms_per_phone must be at least 1");
        }

        if max_farm_dunam == 0 {
            return bad("max_farm_dunam must be at least 1");
        }

        if min_corners < Self::FEWEST_CORNERS {
            return bad("min_corners must be at least 3");
        }

        if max_corners < min_corners {
            return bad("max_corners may not be below min_corners");
        }

        if gps_meters == 0 {
            return bad("gps_meters must be at least 1");
        }

        Ok(Self {
            farms_per_phone,
            max_farm_dunam,
            min_corners,
            max_corners,
            gps_meters,
        })
    }
}

/// Everything staff set on the App control page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSettings {
    pub latest_version: AppVersion,
    pub min_version: AppVersion,
    pub update_message_ku: Option<NoticeText>,
    pub update_message_en: Option<NoticeText>,
    pub maintenance: bool,
    pub maintenance_message_ku: Option<NoticeText>,
    pub maintenance_message_en: Option<NoticeText>,
    pub maintenance_from: Option<DateTime<Utc>>,
    pub maintenance_until: Option<DateTime<Utc>>,
    pub announcement_on: bool,
    pub announcement_ku: Option<NoticeText>,
    pub announcement_en: Option<NoticeText>,
    pub features: Features,
    pub limits: Limits,
    pub help_phone: Option<HelpPhone>,
    pub public_farm_totals: bool,
}

/// What the farmer app reads at start. There is exactly one.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct AppConfig {
    settings: AppSettings,
    /// The staff member who last saved it. `None` for the seeded row.
    updated_by: Option<i32>,
    updated_at: DateTime<Utc>,
}

impl AppConfig {
    /// The settings as staff saved them, once they agree with each other.
    pub fn new(
        settings: AppSettings,
        updated_by: i32,
        now: DateTime<Utc>,
    ) -> Result<Self, AppConfigError> {
        // An app told to update to a version that is not out yet could
        // never be used again.
        if settings.min_version > settings.latest_version {
            return Err(AppConfigError::MinAboveLatest {
                min: settings.min_version.to_string(),
                latest: settings.latest_version.to_string(),
            });
        }

        if let (Some(from), Some(until)) = (settings.maintenance_from, settings.maintenance_until)
            && from >= until
        {
            return Err(AppConfigError::BadMaintenanceWindow);
        }

        // A switch that shows a text needs the text, in Sorani at least:
        // the app would show an empty banner.
        if settings.maintenance && settings.maintenance_message_ku.is_none() {
            return Err(AppConfigError::MissingText("maintenance_message_ku"));
        }

        if settings.announcement_on && settings.announcement_ku.is_none() {
            return Err(AppConfigError::MissingText("announcement_ku"));
        }

        Ok(Self {
            settings,
            updated_by: Some(updated_by),
            updated_at: now,
        })
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        settings: AppSettings,
        updated_by: Option<i32>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            settings,
            updated_by,
            updated_at,
        }
    }
}

/// How many farmers use one version of the app.
#[derive(Clone, Copy, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct VersionUsage {
    version: AppVersion,
    farmers: u64,
    /// The part of all counted farmers on this version, from 0 to 1.
    share: f64,
}

/// Turns farmers counted per version into the list staff read: newest
/// version first, each with its share of all the farmers counted. The share
/// is rounded to four decimals, so the shares may add up to a hair off 1.
pub fn usage_of(mut counts: Vec<(AppVersion, u64)>) -> Vec<VersionUsage> {
    counts.sort_by_key(|(version, _)| std::cmp::Reverse(*version));

    let total: u64 = counts.iter().map(|(_, farmers)| farmers).sum();

    counts
        .into_iter()
        .filter(|(_, farmers)| *farmers > 0)
        .map(|(version, farmers)| VersionUsage {
            version,
            farmers,
            share: ((farmers as f64 / total as f64) * 10_000.0).round() / 10_000.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0)
            .single()
            .expect("time")
    }

    fn version(value: &str) -> AppVersion {
        AppVersion::new(value).expect("version")
    }

    fn notice(value: &str) -> NoticeText {
        NoticeText::new(value.to_string()).expect("text")
    }

    fn settings() -> AppSettings {
        AppSettings {
            latest_version: version("1.2.0"),
            min_version: version("1.0.0"),
            update_message_ku: None,
            update_message_en: None,
            maintenance: false,
            maintenance_message_ku: None,
            maintenance_message_en: None,
            maintenance_from: None,
            maintenance_until: None,
            announcement_on: false,
            announcement_ku: None,
            announcement_en: None,
            features: Features {
                add_farm: true,
                walk_mode: true,
                satellite: true,
                doctor: true,
                reports: false,
                alwa: true,
                plan: false,
                push: false,
            },
            limits: Limits::new(20, 2_000, 3, 50, 10).expect("limits"),
            help_phone: None,
            public_farm_totals: true,
        }
    }

    #[test]
    fn settings_that_agree_are_saved_with_who_saved_them_and_when() {
        let config = AppConfig::new(settings(), 9, now()).expect("config");

        assert_eq!(*config.updated_by(), Some(9));
        assert_eq!(*config.updated_at(), now());
        assert_eq!(config.settings(), &settings());
    }

    #[test]
    fn the_oldest_allowed_version_may_equal_the_latest() {
        let result = AppConfig::new(
            AppSettings {
                min_version: version("1.2.0"),
                ..settings()
            },
            9,
            now(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn the_oldest_allowed_version_may_not_be_above_the_latest() {
        let result = AppConfig::new(
            AppSettings {
                latest_version: version("1.9.0"),
                min_version: version("1.10.0"),
                ..settings()
            },
            9,
            now(),
        );

        assert!(
            matches!(result, Err(AppConfigError::MinAboveLatest { .. })),
            "1.10.0 is above 1.9.0 although it sorts below it as text"
        );
    }

    #[test]
    fn a_maintenance_window_must_end_after_it_starts() {
        let window = |from: DateTime<Utc>, until: DateTime<Utc>| {
            AppConfig::new(
                AppSettings {
                    maintenance_from: Some(from),
                    maintenance_until: Some(until),
                    ..settings()
                },
                9,
                now(),
            )
        };

        assert!(window(now(), now() + Duration::hours(2)).is_ok());
        assert!(matches!(
            window(now(), now()),
            Err(AppConfigError::BadMaintenanceWindow)
        ));
        assert!(matches!(
            window(now(), now() - Duration::hours(1)),
            Err(AppConfigError::BadMaintenanceWindow)
        ));
    }

    #[test]
    fn a_window_with_one_end_only_is_open_ended() {
        let result = AppConfig::new(
            AppSettings {
                maintenance_until: Some(now()),
                ..settings()
            },
            9,
            now(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn maintenance_needs_its_sorani_text() {
        let on = |text: Option<NoticeText>| {
            AppConfig::new(
                AppSettings {
                    maintenance: true,
                    maintenance_message_ku: text,
                    ..settings()
                },
                9,
                now(),
            )
        };

        assert!(matches!(
            on(None),
            Err(AppConfigError::MissingText("maintenance_message_ku"))
        ));
        assert!(on(Some(notice("چاکسازی"))).is_ok());
    }

    #[test]
    fn an_announcement_needs_its_sorani_text() {
        let on = |text: Option<NoticeText>| {
            AppConfig::new(
                AppSettings {
                    announcement_on: true,
                    announcement_ku: text,
                    ..settings()
                },
                9,
                now(),
            )
        };

        assert!(matches!(
            on(None),
            Err(AppConfigError::MissingText("announcement_ku"))
        ));
        assert!(on(Some(notice("ئاگاداری"))).is_ok());
    }

    #[test]
    fn a_text_may_stay_while_its_switch_is_off() {
        let result = AppConfig::new(
            AppSettings {
                announcement_ku: Some(notice("ئاگاداری")),
                ..settings()
            },
            9,
            now(),
        );

        assert!(
            result.is_ok(),
            "staff prepare a text before switching it on"
        );
    }

    #[test]
    fn limits_must_make_sense() {
        assert!(Limits::new(20, 2_000, 3, 50, 10).is_ok());
        assert!(Limits::new(1, 1, 3, 3, 1).is_ok());

        for (farms, dunam, min, max, gps) in [
            (0, 2_000, 3, 50, 10),
            (20, 0, 3, 50, 10),
            (20, 2_000, 2, 50, 10),
            (20, 2_000, 10, 9, 10),
            (20, 2_000, 3, 50, 0),
        ] {
            assert!(
                matches!(
                    Limits::new(farms, dunam, min, max, gps),
                    Err(AppConfigError::BadLimits(_))
                ),
                "{farms} {dunam} {min} {max} {gps} must be refused"
            );
        }
    }

    #[test]
    fn usage_lists_the_newest_version_first_by_number() {
        let usage = usage_of(vec![
            (version("1.9.0"), 1),
            (version("1.10.0"), 2),
            (version("1.0.3"), 1),
        ]);

        assert_eq!(
            usage
                .iter()
                .map(|one| one.version().to_string())
                .collect::<Vec<_>>(),
            vec!["1.10.0", "1.9.0", "1.0.3"]
        );
    }

    #[test]
    fn the_share_is_the_part_of_all_farmers_counted() {
        let usage = usage_of(vec![(version("1.0.0"), 1), (version("1.1.0"), 3)]);

        assert_eq!(*usage[0].farmers(), 3);
        assert_eq!(*usage[0].share(), 0.75);
        assert_eq!(*usage[1].share(), 0.25);
    }

    #[test]
    fn a_share_is_rounded_to_four_decimals() {
        let usage = usage_of(vec![(version("1.0.0"), 1), (version("1.1.0"), 2)]);

        assert_eq!(*usage[0].share(), 0.6667);
        assert_eq!(*usage[1].share(), 0.3333);
    }

    #[test]
    fn nothing_seen_is_an_empty_list_not_a_division_by_zero() {
        assert!(usage_of(vec![]).is_empty());
        assert!(usage_of(vec![(version("1.0.0"), 0)]).is_empty());
    }
}
