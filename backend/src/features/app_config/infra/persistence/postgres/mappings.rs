use sea_orm::ActiveValue::Set;

use crate::{
    app::AppError as GlobalAppError,
    features::app_config::{
        app::AppError,
        domain::{AppConfig, AppSettings, AppVersion, Features, HelpPhone, Limits, NoticeText},
        infra::persistence::postgres::entities::app_config,
    },
};

/// The one row has this id. The table's check constraint allows no other.
pub const THE_ROW: i16 = 1;

fn notice(text: Option<String>) -> Result<Option<NoticeText>, AppError> {
    Ok(text.map(NoticeText::new).transpose()?)
}

fn stored_limit(value: i32) -> Result<u32, AppError> {
    u32::try_from(value).map_err(|_| {
        GlobalAppError::MissingValue("A stored app limit is negative".to_string()).into()
    })
}

fn limit(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

impl TryFrom<app_config::Model> for AppConfig {
    type Error = AppError;

    fn try_from(model: app_config::Model) -> Result<Self, Self::Error> {
        let settings = AppSettings {
            latest_version: AppVersion::new(&model.latest_version)?,
            min_version: AppVersion::new(&model.min_version)?,
            update_message_ku: notice(model.update_message_ku)?,
            update_message_en: notice(model.update_message_en)?,
            maintenance: model.maintenance,
            maintenance_message_ku: notice(model.maintenance_message_ku)?,
            maintenance_message_en: notice(model.maintenance_message_en)?,
            maintenance_from: model.maintenance_from.map(|at| at.and_utc()),
            maintenance_until: model.maintenance_until.map(|at| at.and_utc()),
            announcement_on: model.announcement_on,
            announcement_ku: notice(model.announcement_ku)?,
            announcement_en: notice(model.announcement_en)?,
            features: Features {
                add_farm: model.feature_add_farm,
                walk_mode: model.feature_walk_mode,
                satellite: model.feature_satellite,
                doctor: model.feature_doctor,
                reports: model.feature_reports,
                alwa: model.feature_alwa,
                plan: model.feature_plan,
                push: model.feature_push,
            },
            limits: Limits::new(
                stored_limit(model.limit_farms_per_phone)?,
                stored_limit(model.limit_max_farm_dunam)?,
                stored_limit(model.limit_min_corners)?,
                stored_limit(model.limit_max_corners)?,
                stored_limit(model.limit_gps_meters)?,
            )?,
            help_phone: model.help_phone.map(HelpPhone::new).transpose()?,
            public_farm_totals: model.public_farm_totals,
        };

        Ok(AppConfig::rehydrate(
            settings,
            model.updated_by,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&AppConfig> for app_config::ActiveModel {
    fn from(config: &AppConfig) -> Self {
        let settings = config.settings();
        let text = |text: &Option<NoticeText>| text.as_ref().map(String::from);

        app_config::ActiveModel {
            id: Set(THE_ROW),
            latest_version: Set(settings.latest_version.to_string()),
            min_version: Set(settings.min_version.to_string()),
            update_message_ku: Set(text(&settings.update_message_ku)),
            update_message_en: Set(text(&settings.update_message_en)),
            maintenance: Set(settings.maintenance),
            maintenance_message_ku: Set(text(&settings.maintenance_message_ku)),
            maintenance_message_en: Set(text(&settings.maintenance_message_en)),
            maintenance_from: Set(settings.maintenance_from.map(|at| at.naive_utc())),
            maintenance_until: Set(settings.maintenance_until.map(|at| at.naive_utc())),
            announcement_on: Set(settings.announcement_on),
            announcement_ku: Set(text(&settings.announcement_ku)),
            announcement_en: Set(text(&settings.announcement_en)),
            feature_add_farm: Set(settings.features.add_farm),
            feature_walk_mode: Set(settings.features.walk_mode),
            feature_satellite: Set(settings.features.satellite),
            feature_doctor: Set(settings.features.doctor),
            feature_reports: Set(settings.features.reports),
            feature_alwa: Set(settings.features.alwa),
            feature_plan: Set(settings.features.plan),
            feature_push: Set(settings.features.push),
            limit_farms_per_phone: Set(limit(*settings.limits.farms_per_phone())),
            limit_max_farm_dunam: Set(limit(*settings.limits.max_farm_dunam())),
            limit_min_corners: Set(limit(*settings.limits.min_corners())),
            limit_max_corners: Set(limit(*settings.limits.max_corners())),
            limit_gps_meters: Set(limit(*settings.limits.gps_meters())),
            help_phone: Set(settings.help_phone.as_ref().map(String::from)),
            public_farm_totals: Set(settings.public_farm_totals),
            updated_by: Set(*config.updated_by()),
            updated_at: Set(config.updated_at().naive_utc()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(hour: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 9)
            .and_then(|day| day.and_hms_opt(hour, 0, 0))
            .expect("time")
    }

    /// The row as the migration seeds it.
    fn the_seeded_row() -> app_config::Model {
        app_config::Model {
            id: THE_ROW,
            latest_version: "1.0.0".to_string(),
            min_version: "1.0.0".to_string(),
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
            feature_add_farm: true,
            feature_walk_mode: true,
            feature_satellite: true,
            feature_doctor: true,
            feature_reports: false,
            feature_alwa: true,
            feature_plan: false,
            feature_push: false,
            limit_farms_per_phone: 20,
            limit_max_farm_dunam: 2_000,
            limit_min_corners: 3,
            limit_max_corners: 50,
            limit_gps_meters: 10,
            help_phone: None,
            public_farm_totals: true,
            updated_by: None,
            updated_at: at(8),
        }
    }

    #[test]
    fn the_seeded_row_is_a_valid_config() {
        let config = AppConfig::try_from(the_seeded_row()).expect("config");

        assert_eq!(config.settings().min_version.to_string(), "1.0.0");
        assert!(config.settings().features.doctor);
        assert!(!config.settings().features.push);
        assert_eq!(*config.settings().limits.max_corners(), 50);
        assert_eq!(*config.updated_by(), None);
    }

    #[test]
    fn a_config_survives_the_trip_to_a_row_and_back() {
        let model = app_config::Model {
            latest_version: "1.10.0".to_string(),
            min_version: "1.9.0".to_string(),
            maintenance: true,
            maintenance_message_ku: Some("چاکسازی".to_string()),
            maintenance_from: Some(at(10)),
            maintenance_until: Some(at(12)),
            help_phone: Some("+9647501234567".to_string()),
            public_farm_totals: false,
            updated_by: Some(9),
            ..the_seeded_row()
        };

        let config = AppConfig::try_from(model.clone()).expect("config");
        let active = app_config::ActiveModel::from(&config);

        assert_eq!(active.id, Set(THE_ROW));
        assert_eq!(active.latest_version, Set(model.latest_version));
        assert_eq!(active.min_version, Set(model.min_version));
        assert_eq!(
            active.maintenance_message_ku,
            Set(model.maintenance_message_ku)
        );
        assert_eq!(active.maintenance_from, Set(model.maintenance_from));
        assert_eq!(active.maintenance_until, Set(model.maintenance_until));
        assert_eq!(active.feature_reports, Set(false));
        assert_eq!(active.limit_farms_per_phone, Set(20));
        assert_eq!(active.help_phone, Set(model.help_phone));
        assert_eq!(active.public_farm_totals, Set(false));
        assert_eq!(active.updated_by, Set(Some(9)));
    }

    #[test]
    fn a_row_with_a_version_that_cannot_be_read_is_an_error_not_a_guess() {
        let model = app_config::Model {
            min_version: "one".to_string(),
            ..the_seeded_row()
        };

        assert!(AppConfig::try_from(model).is_err());
    }

    #[test]
    fn a_row_with_a_negative_limit_is_an_error() {
        let model = app_config::Model {
            limit_gps_meters: -1,
            ..the_seeded_row()
        };

        assert!(AppConfig::try_from(model).is_err());
    }
}
