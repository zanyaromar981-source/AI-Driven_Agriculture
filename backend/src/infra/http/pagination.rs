use serde::Deserialize;
use utoipa::IntoParams;

use crate::app::Pagination;

fn default_page() -> u64 {
    1
}

fn default_rows_per_page() -> u64 {
    100
}

#[derive(Deserialize, Debug, Clone, Copy, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationQueryDto {
    /// Page number, starting from 1.
    #[param(example = 1)]
    #[serde(default = "default_page")]
    pub page: u64,

    /// Rows returned per page. Clamped to 100.
    #[param(example = 20)]
    #[serde(default = "default_rows_per_page")]
    pub rows_per_page: u64,
}

impl From<&PaginationQueryDto> for Pagination {
    fn from(value: &PaginationQueryDto) -> Self {
        Self::new(value.page, value.rows_per_page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dto(json: serde_json::Value) -> PaginationQueryDto {
        serde_json::from_value(json).expect("pagination query deserializes")
    }

    #[test]
    fn omitted_parameters_fall_back_to_the_first_page() {
        let query = dto(serde_json::json!({}));

        assert_eq!(query.page, 1);
        assert_eq!(query.rows_per_page, 100);
    }

    #[test]
    fn the_wire_name_is_snake_case() {
        let query = dto(serde_json::json!({ "page": 3, "rows_per_page": 25 }));

        assert_eq!(query.page, 3);
        assert_eq!(query.rows_per_page, 25);
    }

    #[test]
    fn an_over_large_page_size_is_clamped_on_the_way_into_the_app() {
        let pagination = Pagination::from(&dto(
            serde_json::json!({ "page": 2, "rows_per_page": 5000 }),
        ));

        assert_eq!(
            *pagination.rows_per_page(),
            100,
            "a caller must not be able to ask the repository for 5000 rows"
        );
        assert!(!pagination.is_first_page());
    }
}
