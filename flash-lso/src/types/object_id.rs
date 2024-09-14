/// A locally unique identifier for an Amf3 object
///
/// See the comment on `Value::Amf3ObjectReference` for details
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ObjectId(pub i64);

impl ObjectId {
    /// An invalid object id
    ///
    /// Use this when the id of an object can't be known or will never need to be referenced (e.g. amf0)
    /// Unlike valid object id's, multiple objects with an `INVALID` id are explicitly allowed
    /// Attempting to write a reference to an invalid object id is illegal and may error
    /// Attempting to write an object with an invalid object id is allowed, but it cannot be referenced later
    pub const INVALID: Self = ObjectId(-1);
}
