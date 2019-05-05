// SPDX-License-Identifier: AGPL-3.0-only

#![allow(clippy::non_ascii_literal)]

use failure::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// The _LCC Enhancement for the Sortation of Books_ classification system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LESBClassification {
    /// General Works -- Cookbooks.
    AC,
    /// Socio-Political Science and History -- Biographies, Autobiographies, Interviews.
    HB,
    /// Socio-Political Science and History -- General History and Analysis.
    HG,
    /// Socio-Political Science and History -- Media Analysis and Theory.
    HM,
    /// Socio-Political Science and History -- Religion and Religious Texts.
    HR,
    /// Socio-Political Science and History -- People's Theory, History, Analysis.
    HX,
    /// Law -- Theory and Analysis of Games.
    KA,
    /// Law -- Role-Playing Game Rulebooks.
    KG,
    /// Literature -- Fiction.
    LF,
    /// Literature -- Historical Fiction.
    LH,
    /// Literature -- General Literature, Collections, Anthologies.
    LL,
    /// Literature -- Nonfiction.
    LN,
    /// Literature -- Poetry.
    LP,
    /// Literature -- Speculative Fiction.
    LS,
    /// Literature -- Smut.
    LX,
    /// Audio/Visual Art -- Film.
    NF,
    /// Audio/Visual Art -- Video Games.
    NG,
    /// Audio/Visual Art -- Information Design Theory.
    NI,
    /// Audio/Visual Art -- Information Design Reference.
    NJ,
    /// Audio/Visual Art -- Books About Music.
    NM,
    /// Audio/Visual Art -- Recorded Music.
    NR,
    /// Audio/Visual Art -- Books About Visual Art.
    NV,
    /// Audio/Visual Art -- Books Of Art.
    NBookEmoji,
    /// Language -- Dictionaries.
    PD,
    /// Language -- Grammar and Style.
    PG,
    /// Physical Sciences -- Astronomy.
    QA,
    /// Physical Sciences -- Botany.
    QB,
    /// Physical Sciences -- Physics.
    QP,
    /// Physical Sciences -- General Physical Sciences.
    QS,
    /// Physical Sciences -- Zoology.
    QZ,
    /// Relationships and Sex -- Sex Education.
    RE,
    /// Relationships and Sex -- Professional Relationships.
    RF,
    /// Relationships and Sex -- Kink and BDSM.
    RK,
    /// Relationships and Sex -- Polyamory.
    RP,
    /// Witchcraft, Maths, Computers -- Applied Witchcraft and Daemonology.
    WA,
    /// Witchcraft, Maths, Computers -- Electronics.
    WE,
    /// Witchcraft, Maths, Computers -- Mathematics.
    WM,
    /// Witchcraft, Maths, Computers -- Programming and General Computing.
    WP,
    /// Witchcraft, Maths, Computers -- Computer Systems and Security.
    WS,
    /// Witchcraft, Maths, Computers -- Witchcraft, Grimoires, Magical Reference.
    WW,
    /// Witchcraft, Maths, Computers -- Forbidden Knowledge.
    WX,
    /// Miscellaneous -- Quine Has No Self Control and Hoards Random Printed Material.
    XQ,
}

impl LESBClassification {
    pub(crate) fn description(self) -> &'static str {
        use LESBClassification::*;

        match self {
            AC => "Cookbooks",
            HB => "Biographies, Autobiographies, Interviews",
            HG => "General History and Analysis",
            HM => "Media Analysis and Theory",
            HR => "Religion and Religious Texts",
            HX => "People's Theory, History, Analysis",
            KA => "Theory and Analysis of Games",
            KG => "Role-Playing Game Rulebooks",
            LF => "Fiction",
            LH => "Historical Fiction",
            LL => "General Literature, Collections, Anthologies",
            LN => "Nonfiction",
            LP => "Poetry",
            LS => "Speculative Fiction",
            LX => "Smut",
            NF => "Film",
            NG => "Video Games",
            NI => "Information Design Theory",
            NJ => "Information Design Reference",
            NM => "Books About Music",
            NR => "Recorded Music",
            NV => "Books About Visual Art",
            NBookEmoji => "Books Of Art",
            PD => "Dictionaries",
            PG => "Grammar and Style",
            QA => "Astronomy",
            QB => "Botany",
            QP => "Physics",
            QS => "General Physical Sciences",
            QZ => "Zoology",
            RE => "Sex Education",
            RF => "Professional Relationships",
            RK => "Kink and BDSM",
            RP => "Polyamory",
            WA => "Applied Witchcraft and Daemonology",
            WE => "Electronics",
            WM => "Mathematics",
            WP => "Programming and General Computing",
            WS => "Computer Systems and Security",
            WW => "Witchcraft, Grimoires, Magical Reference",
            WX => "Forbidden Knowledge",
            XQ => "Quine Has No Self Control and Hoards Random Printed Material",
        }
    }

    pub(crate) fn category(self) -> LESBCategory {
        use LESBCategory::*;
        use LESBClassification::*;

        match self {
            AC => A,
            HB | HG | HM | HR | HX => H,
            KA | KG => K,
            LF | LH | LL | LN | LP | LS | LX => L,
            NF | NG | NI | NJ | NM | NR | NV | NBookEmoji => N,
            PD | PG => P,
            QA | QB | QP | QS | QZ => Q,
            RE | RF | RK | RP => R,
            WA | WE | WM | WP | WS | WW | WX => W,
            XQ => X,
        }
    }
}

impl fmt::Display for LESBClassification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LESBClassification::*;

        write!(
            f,
            "{}",
            match self {
                AC => "AC",
                HB => "HB",
                HG => "HG",
                HM => "HM",
                HR => "HR",
                HX => "HX",
                KA => "KA",
                KG => "KG",
                LF => "LF",
                LH => "LH",
                LL => "LL",
                LN => "LN",
                LP => "LP",
                LS => "LS",
                LX => "LX",
                NF => "NF",
                NG => "NG",
                NI => "NI",
                NJ => "NJ",
                NM => "NM",
                NR => "NR",
                NV => "NV",
                NBookEmoji => "N📖",
                PD => "PD",
                PG => "PG",
                QA => "QA",
                QB => "QB",
                QP => "QP",
                QS => "QS",
                QZ => "QZ",
                RE => "RE",
                RF => "RF",
                RK => "RK",
                RP => "RP",
                WA => "WA",
                WE => "WE",
                WM => "WM",
                WP => "WP",
                WS => "WS",
                WW => "WW",
                WX => "WX",
                XQ => "XQ",
            }
        )
    }
}

impl FromStr for LESBClassification {
    type Err = Error;

    fn from_str(s: &str) -> Result<LESBClassification, Error> {
        use LESBClassification::*;

        match s {
            "AC" => Ok(AC),
            "HB" => Ok(HB),
            "HG" => Ok(HG),
            "HM" => Ok(HM),
            "HR" => Ok(HR),
            "HX" => Ok(HX),
            "KA" => Ok(KA),
            "KG" => Ok(KG),
            "LF" => Ok(LF),
            "LH" => Ok(LH),
            "LL" => Ok(LL),
            "LN" => Ok(LN),
            "LP" => Ok(LP),
            "LS" => Ok(LS),
            "LX" => Ok(LX),
            "NF" => Ok(NF),
            "NG" => Ok(NG),
            "NI" => Ok(NI),
            "NJ" => Ok(NJ),
            "NM" => Ok(NM),
            "NR" => Ok(NR),
            "NV" => Ok(NV),
            "N📖" => Ok(NBookEmoji),
            "PD" => Ok(PD),
            "PG" => Ok(PG),
            "QA" => Ok(QA),
            "QB" => Ok(QB),
            "QP" => Ok(QP),
            "QS" => Ok(QS),
            "QZ" => Ok(QZ),
            "RE" => Ok(RE),
            "RF" => Ok(RF),
            "RK" => Ok(RK),
            "RP" => Ok(RP),
            "WA" => Ok(WA),
            "WE" => Ok(WE),
            "WM" => Ok(WM),
            "WP" => Ok(WP),
            "WS" => Ok(WS),
            "WW" => Ok(WW),
            "WX" => Ok(WX),
            "XQ" => Ok(XQ),
            _ => Err(failure::err_msg(format!(
                "Unknown LESB classification {:?}",
                s
            ))),
        }
    }
}

impl Serialize for LESBClassification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for LESBClassification {
    fn deserialize<D>(deserializer: D) -> Result<LESBClassification, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// Categories of classifications within the _LESB_.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LESBCategory {
    /// General Works.
    A,
    /// Socio-Political Science and History.
    H,
    /// Law.
    K,
    /// Literature.
    L,
    /// Audio/Visual Art.
    N,
    /// Language.
    P,
    /// Physical Sciences.
    Q,
    /// Relationships and Sex.
    R,
    /// Witchcraft, Maths, Computers.
    W,
    /// Miscellaneous.
    X,
}

impl LESBCategory {
    pub(crate) fn description(self) -> &'static str {
        use LESBCategory::*;

        match self {
            A => "General Works",
            H => "Socio-Political Science and History",
            K => "Law",
            L => "Literature",
            N => "Audio/Visual Art",
            P => "Language",
            Q => "Physical Sciences",
            R => "Relationships and Sex",
            W => "Witchcraft, Maths, Computers",
            X => "Miscellaneous",
        }
    }
}

impl fmt::Display for LESBCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LESBCategory::*;

        write!(
            f,
            "{}",
            match self {
                A => "A",
                H => "H",
                K => "K",
                L => "L",
                N => "N",
                P => "P",
                Q => "Q",
                R => "R",
                W => "W",
                X => "X",
            }
        )
    }
}

impl FromStr for LESBCategory {
    type Err = Error;

    fn from_str(s: &str) -> Result<LESBCategory, Error> {
        use LESBCategory::*;

        match s {
            "A" => Ok(A),
            "H" => Ok(H),
            "K" => Ok(K),
            "L" => Ok(L),
            "N" => Ok(N),
            "P" => Ok(P),
            "Q" => Ok(Q),
            "R" => Ok(R),
            "W" => Ok(W),
            "X" => Ok(X),
            _ => Err(failure::err_msg(format!("Unknown LESB category {:?}", s))),
        }
    }
}

impl Serialize for LESBCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for LESBCategory {
    fn deserialize<D>(deserializer: D) -> Result<LESBCategory, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
