// SPDX-License-Identifier: AGPL-3.0-only

use failure::{ensure, Error};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PartialDate(u16, Option<(u8, Option<u8>)>);

impl PartialDate {
    pub(crate) fn year(&self) -> u16 {
        self.0
    }

    pub(crate) fn month(&self) -> Option<u8> {
        self.1.map(|(month, _)| month)
    }

    pub(crate) fn day(&self) -> Option<u8> {
        self.1.and_then(|(_, opt_day)| opt_day)
    }
}

impl fmt::Display for PartialDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)?;
        if let Some((month, opt_day)) = self.1 {
            write!(f, "-{}", month)?;
            if let Some(day) = opt_day {
                write!(f, "-{}", day)?;
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
