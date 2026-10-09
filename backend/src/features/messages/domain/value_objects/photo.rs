use crate::features::messages::domain::{MessageError, PhotoType};

/// A photo the farmer attached to a message, before it is stored.
#[derive(Clone, PartialEq, Eq)]
pub struct Photo {
    kind: PhotoType,
    bytes: Vec<u8>,
}

impl Photo {
    pub const MAX_MB: usize = 4;
    pub const MAX_BYTES: usize = Self::MAX_MB * 1024 * 1024;

    /// The bytes must really be the type the farmer's phone declared: the
    /// photo is later served to staff browsers under that type, and a file
    /// that is something else must never be.
    pub fn new(kind: PhotoType, bytes: Vec<u8>) -> Result<Self, MessageError> {
        if bytes.len() > Self::MAX_BYTES {
            return Err(MessageError::PhotoTooLarge(Self::MAX_MB));
        }

        if !bytes.starts_with(kind.signature()) {
            return Err(MessageError::PhotoNotAnImage);
        }

        Ok(Self { kind, bytes })
    }

    pub fn kind(&self) -> PhotoType {
        self.kind
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// A photo is megabytes of bytes; its size says all a log needs.
impl std::fmt::Debug for Photo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Photo")
            .field("kind", &self.kind)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg_of(size: usize) -> Vec<u8> {
        let mut bytes = PhotoType::Jpeg.signature().to_vec();
        bytes.resize(size, 0);
        bytes
    }

    #[test]
    fn accepts_a_jpeg_and_a_png() {
        assert!(Photo::new(PhotoType::Jpeg, jpeg_of(1_000)).is_ok());
        assert!(Photo::new(PhotoType::Png, PhotoType::Png.signature().to_vec()).is_ok());
    }

    #[test]
    fn the_size_limit_is_inclusive() {
        assert!(Photo::new(PhotoType::Jpeg, jpeg_of(Photo::MAX_BYTES)).is_ok());
        assert!(matches!(
            Photo::new(PhotoType::Jpeg, jpeg_of(Photo::MAX_BYTES + 1)),
            Err(MessageError::PhotoTooLarge(4))
        ));
    }

    #[test]
    fn bytes_that_are_not_the_declared_type_are_refused() {
        assert!(
            matches!(
                Photo::new(PhotoType::Jpeg, b"hello, not a picture".to_vec()),
                Err(MessageError::PhotoNotAnImage)
            ),
            "a text file labelled image/jpeg must not be stored as a picture"
        );
        assert!(
            matches!(
                Photo::new(PhotoType::Png, jpeg_of(100)),
                Err(MessageError::PhotoNotAnImage)
            ),
            "a JPEG labelled as a PNG is mislabelled"
        );
    }

    #[test]
    fn an_empty_file_is_not_a_photo() {
        assert!(matches!(
            Photo::new(PhotoType::Jpeg, Vec::new()),
            Err(MessageError::PhotoNotAnImage)
        ));
    }

    #[test]
    fn debug_output_gives_the_size_not_the_bytes() {
        let printed = format!(
            "{:?}",
            Photo::new(PhotoType::Jpeg, jpeg_of(2_048)).expect("p")
        );

        assert!(printed.contains("2048"), "{printed}");
        assert!(!printed.contains("255"), "{printed}");
    }
}
