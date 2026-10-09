mod display_name;
mod history_days;
mod idempotency_key;
mod market_name;
mod market_slug;
mod note;
mod price_per_kg;
mod price_source;
mod quantity_kg;
mod zone_slug;

pub use display_name::DisplayName;
pub use history_days::HistoryDays;
pub use idempotency_key::IdempotencyKey;
pub use market_name::MarketName;
pub use market_slug::MarketSlug;
pub use note::Note;
pub use price_per_kg::PricePerKg;
pub use price_source::PriceSource;
pub use quantity_kg::QuantityKg;
pub use zone_slug::ZoneSlug;

const MAX_SLUG_LENGTH: usize = 40;

/// Markets and zones are both named by a slug: lower-case letters and
/// hyphens, for example `sulaymaniyah` or `dashti-hawler`.
fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SLUG_LENGTH
        && value
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
}
