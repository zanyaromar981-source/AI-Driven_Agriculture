use crate::features::farmers::domain::Farmer;

/// A farmer as Ministry staff see one: the stored farmer and how many farms
/// they have. The number comes from the farms feature.
#[derive(Clone, Debug)]
pub struct FarmerRecord {
    pub farmer: Farmer,
    pub farms_count: u64,
}
