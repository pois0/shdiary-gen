use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
    pub const fn new(year: u32, month: u32, day: u32) -> Option<Self> {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => {
                if day < 1 || day > 31 {
                    return None;
                }
            }
            4 | 6 | 9 | 11 => {
                if day < 1 || day > 30 {
                    return None;
                }
            }
            2 => {
                let is_leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
                if day < 1 || day > (if is_leap { 29 } else { 28 }) {
                    return None;
                }
            }
            _ => return None,
        }
        Some(Self { year, month, day })
    }

    pub const fn year(&self) -> u32 {
        self.year
    }

    pub const fn month(&self) -> u32 {
        self.month
    }

    pub const fn day(&self) -> u32 {
        self.day
    }

    pub const fn weekday_ja(&self) -> &str {
        let month = if self.month <= 2 {
            self.month + 12
        } else {
            self.month
        };

        let c = self.year / 100;
        let y = self.year % 100;
        let d = ((self.day + 26 * (month + 1) / 10 + y + y / 4 + (5 * c + c / 4) + 5) % 7) + 1;
        match d {
            1 => "月",
            2 => "火",
            3 => "水",
            4 => "木",
            5 => "金",
            6 => "土",
            7 => "日",
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.year, self.month, self.day)
    }
}
