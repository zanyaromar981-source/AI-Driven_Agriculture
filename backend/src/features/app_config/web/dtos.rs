use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::app_config::{
    app::AppError,
    domain::{
        AppConfig, AppSettings, AppVersion, Features, HelpPhone, Limits, NoticeText, VersionUsage,
    },
};

/// The parts of the app staff can switch off.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AppFeaturesDto {
    pub add_farm: bool,
    pub walk_mode: bool,
    pub satellite: bool,
    pub doctor: bool,
    pub reports: bool,
    pub alwa: bool,
    pub plan: bool,
    pub push: bool,
}

impl From<AppFeaturesDto> for Features {
    fn from(value: AppFeaturesDto) -> Self {
        Self {
            add_farm: value.add_farm,
            walk_mode: value.walk_mode,
            satellite: value.satellite,
            doctor: value.doctor,
            reports: value.reports,
            alwa: value.alwa,
            plan: value.plan,
            push: value.push,
        }
    }
}

impl From<&Features> for AppFeaturesDto {
    fn from(value: &Features) -> Self {
        Self {
            add_farm: value.add_farm,
            walk_mode: value.walk_mode,
            satellite: value.satellite,
            doctor: value.doctor,
            reports: value.reports,
            alwa: value.alwa,
            plan: value.plan,
            push: value.push,
        }
    }
}

/// The limits the app applies on the phone. They are what the app is told,
/// not what the server enforces: the server's own limits are set elsewhere.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AppLimitsDto {
    pub farms_per_phone: u32,
    pub max_farm_dunam: u32,
    pub min_corners: u32,
    pub max_corners: u32,
    pub gps_meters: u32,
}

impl From<&Limits> for AppLimitsDto {
    fn from(value: &Limits) -> Self {
        Self {
            farms_per_phone: *value.farms_per_phone(),
            max_farm_dunam: *value.max_farm_dunam(),
            min_corners: *value.min_corners(),
            max_corners: *value.max_corners(),
            gps_meters: *value.gps_meters(),
        }
    }
}

/// What the farmer app reads at start.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct AppConfigResponse {
    /// The newest version of the app, as `major.minor.patch`.
    pub latest_version: String,
    /// The oldest version still allowed. An older app gets `426
    /// update_required` on every other route.
    pub min_version: String,
    pub update_message_ku: Option<String>,
    pub update_message_en: Option<String>,
    pub maintenance: bool,
    pub maintenance_message_ku: Option<String>,
    pub maintenance_message_en: Option<String>,
    pub maintenance_from: Option<DateTime<Utc>>,
    pub maintenance_until: Option<DateTime<Utc>>,
    pub announcement_on: bool,
    pub announcement_ku: Option<String>,
    pub announcement_en: Option<String>,
    pub features: AppFeaturesDto,
    pub limits: AppLimitsDto,
    pub help_phone: Option<String>,
    pub public_farm_totals: bool,
}

impl From<&AppSettings> for AppConfigResponse {
    fn from(settings: &AppSettings) -> Self {
        let text = |text: &Option<NoticeText>| text.as_ref().map(String::from);

        Self {
            latest_version: settings.latest_version.to_string(),
            min_version: settings.min_version.to_string(),
            update_message_ku: text(&settings.update_message_ku),
            update_message_en: text(&settings.update_message_en),
            maintenance: settings.maintenance,
            maintenance_message_ku: text(&settings.maintenance_message_ku),
            maintenance_message_en: text(&settings.maintenance_message_en),
            maintenance_from: settings.maintenance_from,
            maintenance_until: settings.maintenance_until,
            announcement_on: settings.announcement_on,
            announcement_ku: text(&settings.announcement_ku),
            announcement_en: text(&settings.announcement_en),
            features: (&settings.features).into(),
            limits: (&settings.limits).into(),
            help_phone: settings.help_phone.as_ref().map(String::from),
            public_farm_totals: settings.public_farm_totals,
        }
    }
}

impl From<&AppConfig> for AppConfigResponse {
    fn from(config: &AppConfig) -> Self {
        config.settings().into()
    }
}

/// The config as staff holding an `app` permission see it: the same fields,
/// with who saved it last and when.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardAppConfigResponse {
    #[serde(flatten)]
    pub config: AppConfigResponse,
    /// The id of the staff member who saved it last. `null` until someone
    /// does: the first values come with the server.
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<&AppConfig> for DashboardAppConfigResponse {
    fn from(config: &AppConfig) -> Self {
        Self {
            config: config.into(),
            updated_by: config.updated_by().map(|staff_id| staff_id.to_string()),
            updated_at: *config.updated_at(),
        }
    }
}

/// The whole config. `PUT` replaces every field: one left out of an
/// optional text clears it.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateAppConfigParams {
    /// `major.minor.patch`, for example `1.10.0`.
    pub latest_version: String,
    /// `major.minor.patch`. May not be above `latest_version`.
    pub min_version: String,
    pub update_message_ku: Option<String>,
    pub update_message_en: Option<String>,
    pub maintenance: bool,
    /// Needed while `maintenance` is on.
    pub maintenance_message_ku: Option<String>,
    pub maintenance_message_en: Option<String>,
    pub maintenance_from: Option<DateTime<Utc>>,
    pub maintenance_until: Option<DateTime<Utc>>,
    pub announcement_on: bool,
    /// Needed while `announcement_on` is on.
    pub announcement_ku: Option<String>,
    pub announcement_en: Option<String>,
    pub features: AppFeaturesDto,
    pub limits: AppLimitsDto,
    pub help_phone: Option<String>,
    pub public_farm_totals: bool,
}

/// `null` and a blank text both mean "no text".
fn notice(text: Option<String>) -> Result<Option<NoticeText>, AppError> {
    Ok(text
        .filter(|text| !text.trim().is_empty())
        .map(NoticeText::new)
        .transpose()?)
}

impl UpdateAppConfigParams {
    pub fn into_input(self) -> Result<AppSettings, AppError> {
        Ok(AppSettings {
            latest_version: AppVersion::new(&self.latest_version)?,
            min_version: AppVersion::new(&self.min_version)?,
            update_message_ku: notice(self.update_message_ku)?,
            update_message_en: notice(self.update_message_en)?,
            maintenance: self.maintenance,
            maintenance_message_ku: notice(self.maintenance_message_ku)?,
            maintenance_message_en: notice(self.maintenance_message_en)?,
            maintenance_from: self.maintenance_from,
            maintenance_until: self.maintenance_until,
            announcement_on: self.announcement_on,
            announcement_ku: notice(self.announcement_ku)?,
            announcement_en: notice(self.announcement_en)?,
            features: self.features.into(),
            limits: Limits::new(
                self.limits.farms_per_phone,
                self.limits.max_farm_dunam,
                self.limits.min_corners,
                self.limits.max_corners,
                self.limits.gps_meters,
            )?,
            help_phone: self
                .help_phone
                .filter(|phone| !phone.trim().is_empty())
                .map(HelpPhone::new)
                .transpose()?,
            public_farm_totals: self.public_farm_totals,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSchema)]
pub struct AppVersionUsageResponse {
    pub version: String,
    /// Farmers whose most recent request in the last 30 days came from this
    /// version.
    pub farmers: u64,
    /// The part of all counted farmers on this version, from 0 to 1.
    pub share: f64,
}

impl From<&VersionUsage> for AppVersionUsageResponse {
    fn from(usage: &VersionUsage) -> Self {
        Self {
            version: usage.version().to_string(),
            farmers: *usage.farmers(),
            share: *usage.share(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSchema)]
pub struct AppVersionsResponse {
    /// Newest version first.
    pub versions: Vec<AppVersionUsageResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::{
        app::testing::{a_time, settings, version},
        domain::{AppConfigError, usage_of},
    };

    fn params() -> UpdateAppConfigParams {
        serde_json::from_value(serde_json::json!({
            "latest_version": "1.10.0",
            "min_version": "1.9.0",
            "update_message_ku": "نوێی بکەرەوە",
            "update_message_en": "Please update",
            "maintenance": false,
            "maintenance_message_ku": null,
            "maintenance_message_en": null,
            "maintenance_from": null,
            "maintenance_until": null,
            "announcement_on": true,
            "announcement_ku": "ئاگاداری",
            "announcement_en": "",
            "features": {
                "add_farm": true, "walk_mode": true, "satellite": true, "doctor": true,
                "reports": false, "alwa": true, "plan": false, "push": false
            },
            "limits": {
                "farms_per_phone": 20, "max_farm_dunam": 2000, "min_corners": 3,
                "max_corners": 50, "gps_meters": 10
            },
            "help_phone": "+964 750 123 4567",
            "public_farm_totals": true
        }))
        .expect("params")
    }

    #[test]
    fn a_full_body_becomes_settings() {
        let settings = params().into_input().expect("settings");

        assert_eq!(settings.latest_version, version("1.10.0"));
        assert_eq!(settings.min_version, version("1.9.0"));
        assert!(settings.announcement_on);
        assert_eq!(
            settings.help_phone.as_ref().map(HelpPhone::as_str),
            Some("+9647501234567")
        );
        assert_eq!(*settings.limits.farms_per_phone(), 20);
    }

    #[test]
    fn a_blank_text_is_no_text() {
        let settings = params().into_input().expect("settings");

        assert_eq!(settings.announcement_en, None);
    }

    #[test]
    fn a_version_that_is_not_three_numbers_is_refused() {
        let result = UpdateAppConfigParams {
            min_version: "1.9".to_string(),
            ..params()
        }
        .into_input();

        assert!(matches!(
            result,
            Err(AppError::AppConfig(AppConfigError::BadVersion(_)))
        ));
    }

    #[test]
    fn limits_that_make_no_sense_are_refused() {
        let result = UpdateAppConfigParams {
            limits: AppLimitsDto {
                min_corners: 10,
                max_corners: 5,
                ..params().limits
            },
            ..params()
        }
        .into_input();

        assert!(matches!(
            result,
            Err(AppError::AppConfig(AppConfigError::BadLimits(_)))
        ));
    }

    #[test]
    fn a_body_missing_a_switch_or_carrying_an_unknown_field_cannot_be_read() {
        let mut missing = serde_json::to_value(params()).expect("json");
        missing["features"]
            .as_object_mut()
            .expect("features")
            .remove("push");

        let mut unknown = serde_json::to_value(params()).expect("json");
        unknown["updated_by"] = serde_json::json!("1");

        assert!(serde_json::from_value::<UpdateAppConfigParams>(missing).is_err());
        assert!(serde_json::from_value::<UpdateAppConfigParams>(unknown).is_err());
    }

    #[test]
    fn the_app_reads_every_field_the_request_lists() {
        let json = serde_json::to_value(AppConfigResponse::from(&settings())).expect("json");

        for field in [
            "latest_version",
            "min_version",
            "update_message_ku",
            "update_message_en",
            "maintenance",
            "maintenance_message_ku",
            "maintenance_message_en",
            "maintenance_from",
            "maintenance_until",
            "announcement_on",
            "announcement_ku",
            "announcement_en",
            "features",
            "limits",
            "help_phone",
            "public_farm_totals",
        ] {
            assert!(json.get(field).is_some(), "{field} is missing");
        }

        assert_eq!(json["features"]["plan"], false);
        assert_eq!(json["limits"]["max_corners"], 50);
        assert!(
            json.get("updated_by").is_none(),
            "who saved the config is for staff only"
        );
    }

    #[test]
    fn staff_see_the_same_fields_flat_with_who_saved_them() {
        let config = AppConfig::new(settings(), 9, a_time()).expect("config");

        let json = serde_json::to_value(DashboardAppConfigResponse::from(&config)).expect("json");

        assert_eq!(json["min_version"], "1.0.0");
        assert_eq!(json["updated_by"], "9");
        assert!(json.get("config").is_none(), "the fields are not nested");
    }

    #[test]
    fn a_version_in_use_carries_its_farmers_and_share() {
        let usage = usage_of(vec![(version("1.10.0"), 3), (version("1.9.0"), 1)]);

        let response = AppVersionsResponse {
            versions: usage.iter().map(AppVersionUsageResponse::from).collect(),
        };

        assert_eq!(
            serde_json::to_value(&response).expect("json"),
            serde_json::json!({"versions": [
                {"version": "1.10.0", "farmers": 3, "share": 0.75},
                {"version": "1.9.0", "farmers": 1, "share": 0.25}
            ]})
        );
    }
}
