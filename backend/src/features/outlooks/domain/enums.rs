use crate::{features::outlooks::domain::OutlookError, shared::DomainError};

/// What the next growing season is expected to be like in a zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Outlook {
    Good,
    Normal,
    Bad,
}

impl Outlook {
    pub const ALL: [Outlook; 3] = [Outlook::Good, Outlook::Normal, Outlook::Bad];

    /// Lower comes first on the dashboard: the zones that need attention
    /// lead the list.
    pub fn attention_order(self) -> u8 {
        match self {
            Outlook::Bad => 0,
            Outlook::Normal => 1,
            Outlook::Good => 2,
        }
    }
}

impl From<Outlook> for String {
    fn from(value: Outlook) -> Self {
        match value {
            Outlook::Good => "good".to_string(),
            Outlook::Normal => "normal".to_string(),
            Outlook::Bad => "bad".to_string(),
        }
    }
}

impl TryFrom<&str> for Outlook {
    type Error = OutlookError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "good" => Ok(Outlook::Good),
            "normal" => Ok(Outlook::Normal),
            "bad" => Ok(Outlook::Bad),
            _ => Err(DomainError::InvalidValue(format!("Invalid outlook: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_outlook_round_trips() {
        for outlook in Outlook::ALL {
            let stored = String::from(outlook);
            let parsed = Outlook::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{outlook:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, outlook, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn the_stored_form_is_the_lower_case_code_the_dashboard_uses() {
        assert_eq!(String::from(Outlook::Good), "good");
        assert_eq!(String::from(Outlook::Normal), "normal");
        assert_eq!(String::from(Outlook::Bad), "bad");
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(Outlook::try_from("great").is_err());
        assert!(Outlook::try_from("").is_err());
        assert!(Outlook::try_from("Bad").is_err());
    }

    #[test]
    fn bad_leads_and_good_trails() {
        assert!(Outlook::Bad.attention_order() < Outlook::Normal.attention_order());
        assert!(Outlook::Normal.attention_order() < Outlook::Good.attention_order());
    }
}
