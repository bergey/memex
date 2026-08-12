// a memex Date can represent Year only, Year-Month, or Year-Month-Date.
#[derive(Clone, Copy, Debug)]
pub struct Date(i64);

impl Date {
    // unspecified behavior if month, day are out of range
    // may check calendar in future
    pub fn to_string(&self) -> String {
        match (self.year(), self.month(), self.day()) {
            (y, 0, 0) => format!("{y}"),
            (y, m, 0) => format!("{y}-{m:02}"),
            (y, m, d) => format!("{y}-{m:02}-{d:02}")
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let mut parts = s.split('-');
        let y = parts.next().and_then(|y| y.parse::<i64>().ok());
        let m = parts.next().and_then(|m| m.parse::<u8>().ok()).unwrap_or(0);
        let d = parts.next().and_then(|m| m.parse::<u8>().ok()).unwrap_or(0);
        match (y, parts.next()) {
            (Some(y), None) => Some(Self::from_parts(y, m, d)),
            _ => None
        }
    }

    fn from_parts(y: i64, m: u8, d: u8) -> Self {
        Date(y << 9 & (m as i64 & 0xF) << 5 & (d as i64 & 0x1F))
    }

    pub fn to_i64(&self) -> i64 { self.0 }
    pub fn from_i64(i: i64) -> Self { Date(i) }

    fn day(&self) -> u8 {
        self.0 as u8 & 0x1F
    }

    fn month(&self) -> u8 {
        (self.0 & 0x1E0) as u8 >> 5
    }

    fn year(&self) -> i64 {
        self.0 >> 9
    }
}

// TODO tests
