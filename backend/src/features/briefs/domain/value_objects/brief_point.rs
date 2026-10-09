use getset::Getters;

use crate::features::briefs::domain::{PointLevel, PointText};

/// One thing a brief wants the reader to notice, in both languages.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct BriefPoint {
    level: PointLevel,
    text_en: PointText,
    text_ku: PointText,
}

impl BriefPoint {
    pub fn new(level: PointLevel, text_en: PointText, text_ku: PointText) -> Self {
        Self {
            level,
            text_en,
            text_ku,
        }
    }
}
