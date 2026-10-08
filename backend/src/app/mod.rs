mod errors;
pub use errors::*;

use getset::Getters;

use crate::shared::Phone;

#[derive(Clone, Copy, Debug, Getters)]
#[getset(get = "pub")]
pub struct Pagination {
    page: u64,
    rows_per_page: u64,
}

impl Pagination {
    pub fn new(page: u64, rows_per_page: u64) -> Self {
        Self {
            page: page.max(1),
            rows_per_page: rows_per_page.clamp(1, 100),
        }
    }

    pub fn skip(&self) -> u64 {
        (self.page - 1) * self.rows_per_page
    }

    pub fn is_first_page(&self) -> bool {
        self.page == 1
    }
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct AuthContext {
    user: User,
    jwt: String,
}

impl AuthContext {
    pub fn new(user: User, jwt: String) -> Self {
        Self { user, jwt }
    }
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct User {
    pub phone: Phone,
}

impl User {
    pub fn new(phone: Phone) -> Self {
        Self { phone }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_below_one_is_pulled_up_to_the_first() {
        assert_eq!(*Pagination::new(0, 20).page(), 1);
        assert_eq!(Pagination::new(0, 20).skip(), 0);
    }

    #[test]
    fn rows_per_page_is_clamped_to_a_sane_window() {
        assert_eq!(*Pagination::new(1, 0).rows_per_page(), 1);
        assert_eq!(*Pagination::new(1, 5000).rows_per_page(), 100);
        assert_eq!(*Pagination::new(1, 20).rows_per_page(), 20);
    }

    #[test]
    fn skip_counts_whole_pages_before_the_current_one() {
        assert_eq!(Pagination::new(1, 20).skip(), 0);
        assert_eq!(Pagination::new(2, 20).skip(), 20);
        assert_eq!(Pagination::new(5, 20).skip(), 80);
    }

    #[test]
    fn skip_uses_the_clamped_rows_per_page_not_the_requested_one() {
        assert_eq!(
            Pagination::new(3, 5000).skip(),
            200,
            "an over-large page size must not be able to skip past the clamp"
        );
    }

    #[test]
    fn only_the_first_page_carries_the_total_count() {
        assert!(Pagination::new(1, 20).is_first_page());
        assert!(Pagination::new(0, 20).is_first_page());
        assert!(!Pagination::new(2, 20).is_first_page());
    }
}
