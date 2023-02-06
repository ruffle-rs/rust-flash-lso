/// An identifier for a cacheable element
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheKey(usize);

impl CacheKey {
    /// Create a `CacheKey` from a pointer representing an element to be serialized
    pub fn from_ptr<T>(p: *const T) -> Self {
        Self(p as usize)
    }
}
