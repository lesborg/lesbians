// SPDX-License-Identifier: AGPL-3.0-only

use lazy_static::lazy_static;

pub(crate) fn isbn10_to_isbn13(isbn10: &str) -> Option<String> {
    let isbn10 = isbn10.replace('-', "").into_bytes();
    if isbn10.len() != 10
        || !isbn10.iter().take(9).all(|b| b.is_ascii_digit())
        || !(isbn10[9].is_ascii_digit() || isbn10[9] == b'X')
    {
        return None;
    }

    let mut isbn13: [u8; 13] = [b'0'; 13];
    (&mut isbn13[0..3]).copy_from_slice(b"978");
    (&mut isbn13[3..12]).copy_from_slice(&isbn10[0..9]);

    let sum: u8 = isbn13
        .iter()
        .take(12)
        .enumerate()
        .map(|(i, b)| (b - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    isbn13[12] = (10 - (sum % 10)) + b'0';

    Some(String::from_utf8(isbn13.to_vec()).unwrap())
}

pub(crate) fn isbn13_to_isbn10(isbn13: &str) -> Option<String> {
    let isbn13 = isbn13.replace('-', "").into_bytes();
    if isbn13.len() != 13 || !isbn13.iter().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let mut isbn10: [u8; 10] = [b'0'; 10];
    (&mut isbn10[0..9]).copy_from_slice(&isbn13[3..12]);

    let mut sum = 0u16;
    let mut acc = 0u16;
    for i in 0..9 {
        acc += u16::from(isbn10[i] - b'0');
        sum += acc;
    }
    sum += acc;
    isbn10[9] = match 11 - (sum % 11) as u8 {
        b @ 0...9 => b + b'0',
        10 => b'X',
        _ => unreachable!(),
    };

    Some(String::from_utf8(isbn10.to_vec()).unwrap())
}

#[cfg(test)]
mod tests {
    use super::{isbn10_to_isbn13, isbn13_to_isbn10};

    #[test]
    fn test_isbn10_to_isbn13() {
        assert_eq!(
            isbn10_to_isbn13("0306406152"),
            Some("9780306406157".to_owned())
        );
        assert_eq!(
            isbn10_to_isbn13("080442957X"),
            Some("9780804429573".to_owned())
        );
    }

    #[test]
    fn test_isbn13_to_isbn10() {
        assert_eq!(
            isbn13_to_isbn10("9780306406157"),
            Some("0306406152".to_owned())
        );
        assert_eq!(
            isbn13_to_isbn10("9780804429573"),
            Some("080442957X".to_owned())
        );
    }
}
