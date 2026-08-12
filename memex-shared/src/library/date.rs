// a memex Date can represent Year only, Year-Month, or Year-Month-Date.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Date(i64);

impl Date {
    // unspecified behavior if month, day are out of range
    // may check calendar in future
    pub fn to_string(&self) -> String {
        match (self.year(), self.month(), self.day()) {
            (0, _, _) => "".to_owned(),
            (y, 0, 0) => format!("{y}"),
            (y, m, 0) => format!("{y}-{m:02}"),
            (y, m, d) => format!("{y}-{m:02}-{d:02}")
        }
    }

    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        let mut parts = s.as_ref().split('-');
        let y = parts.next().and_then(|y| y.parse::<i64>().ok());
        let m = parts.next().and_then(|m| m.parse::<u8>().ok()).unwrap_or(0);
        let d = parts.next().and_then(|m| m.parse::<u8>().ok()).unwrap_or(0);
        match (y, parts.next()) {
            (Some(y), None) => Self::from_parts(y, m, d),
            _ => Self::from_parts(0 ,0 ,0)
        }
    }

    fn from_parts(y: i64, m: u8, d: u8) -> Self {
        Date(y << 9 | (m as i64 & 0xF) << 5 | (d as i64 & 0x1F))
    }

    pub fn to_i64(&self) -> i64 { self.0 }
    pub fn from_i64(i: i64) -> Self { Date(i) }

    fn day(&self) -> u8 {
        self.0 as u8 & 0x1F
    }

    fn month(&self) -> u8 {
        (self.0 >> 5) as u8 & 0xF
    }

    fn year(&self) -> i64 {
        self.0 >> 9
    }
}

impl Default for Date {
    fn default() -> Self {
        Date::from_i64(0)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use proptest::prelude::*;

    fn test_from_str(s: &str, y: i64, m: u8, d: u8) {
        assert_eq!(Date::from_str(s), Date::from_parts(y, m, d))
    }

    #[test]
    fn from_y() {
        test_from_str("100", 100, 0, 0);
    }

    #[test]
    fn from_y_m() {
        test_from_str("2026-08", 2026, 8, 0)
    }

    #[test]
    fn from_y_m_d() {
        test_from_str("2026-08-12", 2026, 8, 12);
    }

    #[test]
    fn to_y_m_d() {
        assert_eq!("2026-08-12", Date::from_parts(2026,8,12).to_string());
    }

    #[test]
    fn string_round_trip() {
        let to_from_str = |s: &str| {
            assert_eq!(s, Date::from_str(s).to_string());
        };

        for s in [
            "2026-08-12",
            "2026-08",
            "2026",
            "1950",
            "900",
            "12345",
            "12345-10-01",
            "2026-08-01"
        ] {
            to_from_str(s);
        }
    }

    proptest! {
        #[test]
        // fn delete_any_record(n in 1..=10usize, i in 0..10000usize) {
        fn from_to_string(y in 1..3000i64, m in 0..12u8, d in 0..31u8) {
            let date = Date::from_parts(y, m, d);
            let s = date.to_string();
            let parsed = Date::from_str(s);
            assert_eq!(parsed, date);
        }
    }
}
