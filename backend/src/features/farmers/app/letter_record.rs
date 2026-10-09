use crate::features::farmers::domain::{FarmHolding, Farmer, FarmerName, Letter, LetterTotals};

/// Everything a printed support letter shows, as it was when the letter was
/// issued.
#[derive(Clone, Debug)]
pub struct IssuedLetter {
    pub letter: Letter,
    pub farmer: Farmer,
    pub farms: Vec<FarmHolding>,
    pub totals: LetterTotals,
    /// The name of the staff member who issued it.
    pub issued_by_name: String,
}

/// A stored letter, for checking one later, with the farmer's name as it is
/// now. The name is `None` when the farmer has none or is gone.
#[derive(Clone, Debug)]
pub struct LetterRecord {
    pub letter: Letter,
    pub farmer_name: Option<FarmerName>,
}
