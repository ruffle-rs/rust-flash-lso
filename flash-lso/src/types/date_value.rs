/// The data contained within a Value of type Date
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct DateValue {
    /// Number of seconds since the epoch
    pub time: f64,

    /// The timezone identifier or UTC if missing
    pub timezone_or_utc: Option<u16>,
}

impl PartialEq for DateValue {
    fn eq(&self, other: &Self) -> bool {
        // We consider two NaN times to be equal
        let time_eq = (self.time.is_nan() && other.time.is_nan()) || (self.time == other.time);
        let tz_eq = self.timezone_or_utc == other.timezone_or_utc;
        time_eq && tz_eq
    }
}
