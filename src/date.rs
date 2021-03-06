// SPDX-License-Identifier: AGPL-3.0-only

use failure::{ensure, Error};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PartialDate(pub u16, pub Option<(u8, Option<u8>)>);

impl PartialDate {
    pub(crate) fn year(self) -> u16 {
        self.0
    }
}

impl fmt::Display for PartialDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}", self.0)?;
        if let Some((month, opt_day)) = self.1 {
            write!(f, "-{:02}", month)?;
            if let Some(day) = opt_day {
                write!(f, "-{:02}", day)?;
            }
        }
        Ok(())
    }
}

impl FromStr for PartialDate {
    type Err = Error;

    fn from_str(s: &str) -> Result<PartialDate, Error> {
        let mut iter = s.split('-');
        let mut date = PartialDate(
            iter.next()
                .ok_or_else(|| failure::err_msg("empty string"))?
                .parse()?,
            None,
        );
        if let Some(month_str) = iter.next() {
            let month = month_str.parse()?;
            let day = match iter.next() {
                Some(day_str) => Some(day_str.parse()?),
                None => None,
            };
            date.1 = Some((month, day));
        }
        ensure!(iter.next().is_none(), "too many date components");
        Ok(date)
    }
}

impl Serialize for PartialDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for PartialDate {
    fn deserialize<D>(deserializer: D) -> Result<PartialDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::PartialDate;
    use std::str::FromStr;

    #[test]
    fn test() {
        let date = PartialDate(6969, None);
        assert_eq!(PartialDate::from_str("6969").unwrap(), date);
        assert_eq!("6969", date.to_string());

        let date = PartialDate(6969, Some((4, None)));
        assert_eq!(PartialDate::from_str("6969-04").unwrap(), date);
        assert_eq!("6969-04", date.to_string());

        let date = PartialDate(6969, Some((4, Some(20))));
        assert_eq!(PartialDate::from_str("6969-04-20").unwrap(), date);
        assert_eq!("6969-04-20", date.to_string());
    }
}
