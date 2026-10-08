use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum DamError {
    #[error(
        "A volume of {volume_bn_m3} billion m3 is more than the dam can hold ({capacity_bn_m3} billion m3)"
    )]
    VolumeOverCapacity {
        volume_bn_m3: f64,
        capacity_bn_m3: f64,
    },

    #[error("The history range ends before it starts")]
    RangeEndsBeforeItStarts,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
