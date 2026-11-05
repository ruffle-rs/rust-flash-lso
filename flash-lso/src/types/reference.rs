/// A reference to an existing value
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference(pub(crate) u16);

impl Reference {
    /// Extract the reference index from this `Reference`
    pub fn index(self) -> u16 {
        self.0
    }
}
