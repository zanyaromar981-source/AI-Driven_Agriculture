use crate::features::workers::domain::WorkerError;

const MAX_LENGTH: usize = 200;

/// A few words on what work the person does, in their own language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerNote(String);

impl WorkerNote {
    /// A blank note is no note: the form in the app sends an empty box as
    /// an empty text.
    pub fn new(value: String) -> Result<Option<Self>, WorkerError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Ok(None);
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(WorkerError::NoteTooLong(MAX_LENGTH));
        }

        Ok(Some(Self(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&WorkerNote> for String {
    fn from(value: &WorkerNote) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(WorkerNote::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(WorkerNote::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn a_blank_note_is_no_note() {
        assert_eq!(WorkerNote::new("  ".to_string()).expect("none"), None);
    }

    #[test]
    fn surrounding_whitespace_is_stripped() {
        assert_eq!(
            WorkerNote::new(" harvest, pruning ".to_string())
                .expect("note")
                .expect("some")
                .as_str(),
            "harvest, pruning"
        );
    }
}
