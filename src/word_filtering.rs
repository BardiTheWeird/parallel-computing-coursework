use std::{io::{self, Read}, fs::File, collections::HashSet};

pub fn file_to_words<'a>(mut file: File) -> io::Result<HashSet<String>> {
    let mut words_set = HashSet::new();
    let mut word_left: Option<String> = None;

    let mut buffer =  [0 as u8; 264];
    let mut read_start = 0;

    loop {
        let bytes_read = file.read(&mut buffer[read_start..])?;
        if bytes_read == 0 {
            if read_start > 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected end of UTF8 input"));
            }
            break;
        }

        let (string_read, bytes_left) = bytes_to_str(&buffer[..bytes_read])?;

        match scan_for_words(string_read) {
            ScanForWordsResult::Words(mut words) => {
                if let Some(word) = word_left {
                    words[0].insert_str(0, &word);
                }
                for s in words {
                    words_set.insert(s);
                }
                word_left = None;
            },
            ScanForWordsResult::WordsTrailingAlphanumericRun((mut words, run)) => {
                if let Some(word) = word_left {
                    words[0].insert_str(0, &word);
                }
                for s in words {
                    words_set.insert(s);
                }
                word_left = Some(run.to_owned());
            },
            ScanForWordsResult::EverythingAlphanumericRun(run) => word_left = match word_left {
                Some(mut word) => {
                    word.push_str(run);
                    Some(word)
                },
                None => Some(run.to_owned()),
            },
            ScanForWordsResult::NoWords => {},
        }

        read_start = match bytes_left {
            None => 0,
            Some(bytes_left) => {
                let content_len = read_start + bytes_read;
                buffer.copy_within((content_len-bytes_left)..content_len, 0);
                bytes_left
            },
        };
    };
    
    if let Some(word) = word_left {
        words_set.insert(word);
    }

    return Ok(words_set);
}

#[derive(Debug)]
enum ScanForWordsResult<'a> {
    NoWords,
    EverythingAlphanumericRun(&'a str),
    Words(Vec<String>),
    WordsTrailingAlphanumericRun((Vec<String>, &'a str))
}

/// Searches `s` for runs of alphanumeric characters and appends them to `words_vec`
/// 
/// Let `a` be an alphanumeric character.
/// Let `n` be a non-alphanumeric character.
/// Thus any string can be represented as `"aaaannaanna"` or similar.
/// 
/// Let `A` be a run of `a`.
/// Let `N` be a run of `n`.
/// Thus any string can be represented as `[N](AN){m}[A]`
/// where `[X]` denotes that `X` is optional, `(X){m}` denotes that `X` is repeated `m` times
/// 
/// `scan_for_words` first finds the breakpoints (indices at which `A` turns into `N` or vice versa).
/// Then it splits `s` accordingly.
/// 
/// If `s` is in the form of `[N](AN){m}A`, `Some(A)` is returned.
/// Otherwise `None` is returned
fn scan_for_words<'a, 'b>(s: &'a str) -> ScanForWordsResult {
    let char_indices = s.char_indices()
        .map(|(i, c)| (i, c.is_alphanumeric() || c == '\''));
    
    let right = char_indices.clone().skip(1);
    let char_pairs = char_indices.zip(right);
    // (i, false) means that an alphanumeric run ends at i (non-inclusive)
    // (i, true)  means that an alphanumeric run starts at i (inclusive)
    let mut break_points = char_pairs
        .filter(|((_, c1), (_, c2))| c1 != c2)
        .map(|(_, t)| t)
        .peekable();
    
    let mut ranges: Box<dyn Iterator<Item = (usize, bool)>>;
    if let Some((_, starts_run)) = break_points.peek() {
        ranges = if !starts_run {
            Box::new(std::iter::once((0, true)).chain(break_points))
        }
        else {
            Box::new(break_points)
        }
    } else {
        let first_char = s.chars().next();
        if let Some(c) = first_char {
            if c.is_alphanumeric() {
                return ScanForWordsResult::EverythingAlphanumericRun(s);
            }
        }
        return ScanForWordsResult::NoWords
    }

    let mut words = vec![];
    loop {
        let (break_point_1, _) = match (*ranges).next() {
            Some(x) => x,
            None => return ScanForWordsResult::Words(words),
        };
        let (break_point_2, _) = match (*ranges).next() {
            Some(x) => x,
            None => return ScanForWordsResult::WordsTrailingAlphanumericRun((words, &s[break_point_1..]))
        };
        words.push(String::from(&s[break_point_1..break_point_2]));
    }
}

/// In-place converts a UTF8 formatted string into `&str`.
/// 
/// If a UTF8 character was cut off in the end,
/// returns the length left in the second element of a tuple
/// 
/// If there is an invalid UTF8 character, returns an io::Error
fn bytes_to_str(bytes: &[u8]) -> io::Result<(&str, Option<usize>)> {
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok((s, None)),
        Err(utf8_error) => {
            if let Some(_) = utf8_error.error_len() {
                Err(io::Error::new(io::ErrorKind::InvalidInput, 
                    "string contains invalid UTF8"))
            } else {
                let valid_up_to = utf8_error.valid_up_to();
                let s = std::str::from_utf8(&bytes[..valid_up_to]).unwrap();
                Ok((s, Some(bytes.len() - valid_up_to)))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    impl PartialEq for ScanForWordsResult<'_> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::EverythingAlphanumericRun(l0), Self::EverythingAlphanumericRun(r0)) => l0 == r0,
                (Self::Words(l0), Self::Words(r0)) => vec_compare(l0, r0),
                (Self::WordsTrailingAlphanumericRun((l0, l1)), 
                    Self::WordsTrailingAlphanumericRun((r0, r1))) => vec_compare(l0, r0) && l1 == r1,
                _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }

    #[test]
    fn test_scan_words() {
        use ScanForWordsResult::*;
        struct ScanForWordsTestCase<'a> {
            string: &'a str,
            expected_result: ScanForWordsResult<'a>,
        }

        let test_cases = vec![
            ScanForWordsTestCase {
                string: "アニャ likes peanuts, ワクワク!",
                expected_result: Words(vec_to_owned(vec!["アニャ", "likes", "peanuts", "ワクワク"])),
            },
            ScanForWordsTestCase {
                string: "!アニャ likes peanuts, ワクワク!",
                expected_result: Words(vec_to_owned(vec!["アニャ", "likes", "peanuts", "ワクワク"])),
            },
            ScanForWordsTestCase {
                string: "アニャ likes peanuts, ワクワク",
                expected_result: WordsTrailingAlphanumericRun(
                    (vec_to_owned(vec!["アニャ", "likes", "peanuts"]), "ワクワク")),
            },
            ScanForWordsTestCase {
                string: "ワクワク",
                expected_result: EverythingAlphanumericRun("ワクワク"),
            },
            ScanForWordsTestCase {
                string: "!,==<>",
                expected_result: NoWords,
            },
            ScanForWordsTestCase {
                string: "let's play!",
                expected_result: Words(vec_to_owned(vec!["let's", "play"])),
            },
        ];

        for case in test_cases {
            let res = scan_for_words(&case.string);

            assert_eq!(res, case.expected_result, "case `{}`:
            expected: {:?}
            actual: {:?}", case.string, case.expected_result, res);
        }
    }

    fn vec_compare<T>(v1: &Vec<T>, v2: &Vec<T>) -> bool
    where
        T: std::cmp::PartialEq 
    {
        v1.len() == v2.len() &&
            v1.iter().zip(v2).all(|(a, b)| a == b)
    }

    fn vec_to_owned(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|&s| s.to_owned()).collect()
    }
}