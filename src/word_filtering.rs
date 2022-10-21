use std::{io::{self, Read}, collections::HashSet};

pub fn reader_to_words<'a>(mut reader: impl Read) -> io::Result<HashSet<String>> {
    let mut words_set = HashSet::new();
    let mut word_left: Option<String> = None;

    let mut buffer =  [0 as u8; 264];
    let mut read_start = 0;

    loop {
        let bytes_read = reader.read(&mut buffer[read_start..])?;
        if bytes_read == 0 {
            if read_start > 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected end of UTF8 input"));
            }
            break;
        }

        let (string_read, bytes_left) = bytes_to_str(&buffer[..bytes_read])?;

        match scan_for_words_from_reader(string_read) {
            ScanForWordsResult::Words(WordsWithAlphanumericRuns{
                mut words, leading_run, trailing_run}) => 
            {
                if let Some(word) = word_left {
                    match leading_run {
                        true => words[0].insert_str(0, &word),
                        false => { words_set.insert(word); },
                    }
                    word_left = None;
                }

                for word in words {
                    words_set.insert(word);
                }

                if let Some(run) = trailing_run {
                    word_left = Some(run.to_owned())
                }
            },
            ScanForWordsResult::SingleAlphanumericRun => word_left = match word_left {
                Some(mut word) => {
                    word.push_str(string_read);
                    Some(word)
                },
                None => Some(string_read.to_owned()),
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
    SingleAlphanumericRun,
    Words(WordsWithAlphanumericRuns<'a>),
}

#[derive(Debug)]
struct WordsWithAlphanumericRuns<'a> {
    words: Vec<String>,
    leading_run: bool,
    trailing_run: Option<&'a str>
}

/// Scans `s` for runs of alphanumeric characters and returns them in `HashSet<String>`
pub fn scan_for_unique_words(s: &str) -> Option<HashSet<String>> {
    if s.is_empty() {
        return None;
    }

    let mut words = HashSet::new();

    let mut chars = s.char_indices();
    let mut word_start = match is_word_char(chars.next().unwrap().1) {
        true => Some(0),
        false => None
    };

    for (i, c) in chars {
        match (word_start, is_word_char(c)) {
            (Some(i_start), false) => {
                words.insert(s[i_start..i].to_owned());
                word_start = None;
            },
            (None, true) => word_start = Some(i),
            _ => {}
        }
    }
    if let Some(i) = word_start {
        words.insert(s[i..].to_owned());
    }
    Some(words)
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
/// `scan_for_words` finds the breakpoints (indices at which `A` turns into `N` or vice versa), 
/// and then it splits `s` accordingly.
/// 
/// If `s` is of the form of `[N]`, `None` is returned.
/// 
/// If `s` is of the form of `(AN){m}[A]`, `Words` is returned with leading_run = true
/// 
/// If `s` is of the form of `[N](AN){m}A`, `Words` is returned with trailing_run = Some(A)
/// 
/// If `s` is of the form of `A`, `SingleAlphanumericRun` is returned.
fn scan_for_words_from_reader<'a, 'b>(s: &'a str) -> ScanForWordsResult {
    if s.is_empty() {
        return ScanForWordsResult::NoWords
    }
    let mut words = vec![];

    let mut chars = s.char_indices();
    let mut word_start = match is_word_char(chars.next().unwrap().1) {
        true => Some(0),
        false => None
    };
    let leading_run = word_start.is_some();

    for (i, c) in chars {
        match (word_start, is_word_char(c)) {
            (Some(i_start), false) => {
                words.push(s[i_start..i].to_owned());
                word_start = None;
            },
            (None, true) => word_start = Some(i),
            _ => {}
        }
    }

    match words.is_empty() {
        true => match word_start {
            Some(_) => ScanForWordsResult::SingleAlphanumericRun,
            None => ScanForWordsResult::NoWords,
        },
        false => ScanForWordsResult::Words(WordsWithAlphanumericRuns{ 
                words, leading_run, trailing_run: match word_start {
                Some(i) => Some(&s[i..]),
                _ => None,
            } 
        })
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\''
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
    use stringreader::StringReader;

    use super::*;

    #[test]
    fn test_reader_to_words() {
        struct ReaderToWordsTestCase<'a> {
            content: &'a str,
            words: HashSet<String>,
        }

        let test_cases = vec![
            ReaderToWordsTestCase {
                content: "Yah. I know. It has the name \"Sinatra\" in the title, so how bad can it be? Well, it's bad, trust me! I rented this thinking it was some movie I missed in the theaters. It's not. It's some garbage \"movie\" made by the folks at Showtime (cable station). Geez, these cable stations make a few bucks they think they can make whatever garbage movies they want! It's not good. I am as big a Sinatra fan as any sane man, but this movie was just dumb. Boring. Dull. Unfunny. Uninteresting. The only redeeming quality is that (assuming they did stick to the facts) you do learn about what happened to the captors of Frank Jr. Otherwise it's just a stupid film.",
                words: vec_to_owned(vec!["in", "can", "they", "did", "Boring", "movies", "Sinatra", "was", "sane", "but", "has", "assuming", "fan", "Well", "stick", "learn", "thinking", "so", "movie", "me", "quality", "that", "think", "bucks", "stations", "any", "man", "It's", "about", "Frank", "It", "happened", "Jr", "made", "a", "the", "as", "do", "is", "film", "Yah", "of", "be", "you", "not", "missed", "redeeming", "I", "it", "by", "at", "Geez", "few", "The", "title", "it's", "some", "Uninteresting", "only", "good", "am", "folks", "how", "facts", "name", "stupid", "this", "dumb", "know", "bad", "theaters", "whatever", "just", "what", "big", "garbage", "these", "captors", "to", "cable", "Showtime", "trust", "Unfunny", "rented", "Otherwise", "station", "Dull", "want", "make"]).into_iter().collect(),
            },
            ReaderToWordsTestCase {
                content: "I cannot understand...Simbu does not have any originality. ..He acts and copies Rajini's style, then copies Kamal's style .. then copies Vijay's style and then last but not least copies his dad's style. Does it mean he is more like a mimicry hero. ..who has no originality and just copies everyone...He wants to be like everyone and never wants to be like Simbu.....It is so annoying and boring to see the same crap. .. If the same portrait of Rajini, Kamal and Vijay are going to be always there, why see a dummy instead of a real one. Maybe Simbu should accept himself and act his own style instead of having no originality and copying everyone.<br /><br />Renu",
                words: vec_to_owned(vec!["his", "He", "acts", "but", "instead", "then", "wants", "portrait", "always", "any", "is", "originality", "mimicry", "not", "boring", "never", "why", "should", "accept", "himself", "annoying", "style", "the", "crap", "mean", "it", "no", "copies", "Does", "Rajini", "understand", "cannot", "Vijay\'s", "has", "are", "last", "Kamal\'s", "and", "hero", "does", "Kamal", "real", "Rajini\'s", "going", "he", "see", "same", "Renu", "having", "dummy", "Maybe", "a", "Simbu", "dad\'s", "like", "one", "just", "br", "to", "copying", "Vijay", "more", "everyone", "have", "It", "If", "least", "be", "act", "own", "so", "I", "of", "there", "who"]).into_iter().collect(),
            },
            ReaderToWordsTestCase {
                content: "Wow!I am quite disappointed that this could not compete with his recent sh*t called 'Kaalai'(The Bull-It was even more pathetic) Vallavan-has no story,screenplay and direction.It only has a good original score by Yuvan Shankar Raja.If he hadn't been there,the movie would have fallen flat.Even when he is there the movie still falls flat.<br /><br />A youngster falls in love with a girl 3 years older than him,and thinks its no mistake.He even has sex with her.After knowing that he is a young boy the girl gets annoyed and refuses to marry him.Now another female who had a crush on this guy in high school enters and kidnaps a friend of this guy.How he convinces the older girl and marries her,deceiving the crush is the story.Okay,we can't call this a story but unfortunately this is what we have as a story.Now there is cheap and vulgar scenes which is more or less pornography,comedy which makes you weep,acting which makes you puke and horrifying punch dialogues to stab you over and over.Simbu is there to give you goosebumps!I wish I could rate this movie in minus infinity,sadly IMDb allows me rate 1 star as the lowest.<br /><br />Are you nuts?Are you a dumbo?Are you a bozo?Watch this movie.(I did because some nutcases recommended me this movie) <br /><br />Watch a tom and jerry episode,funny videos or even old photo albums.Try to miss this movie by all means.Otherwise you will repent on doing a life-time mistake.",
                words: vec_to_owned(vec!["nutcases", "episode", "but", "jerry", "enters", "did", "some", "tom", "crush", "would", "Bull", "marry", "only", "call", "lowest", "gets", "am", "goosebumps", "was", "star", "which", "time", "had", "dumbo", "annoyed", "youngster", "score", "less", "direction", "boy", "story", "3", "high", "scenes", "called", "minus", "deceiving", "If", "have", "vulgar", "of", "IMDb", "means", "who", "years", "been", "albums", "will", "refuses", "videos", "her", "me", "convinces", "movie", "school", "doing", "mistake", "flat", "1", "young", "disappointed", "over", "compete", "or", "sadly", "photo", "recent", "stab", "makes", "t", "still", "falls", "puke", "rate", "to", "It", "Yuvan", "Shankar", "in", "friend", "pathetic", "I", "even", "there", "this", "A", "dialogues", "punch", "weep", "is", "marries", "pornography", "wish", "girl", "can\'t", "Wow", "hadn\'t", "another", "\'Kaalai\'", "Otherwise", "repent", "has", "screenplay", "and", "cheap", "quite", "with", "allows", "Even", "he", "we", "infinity", "fallen", "old", "a", "its", "Simbu", "After", "comedy", "thinks", "sex", "older", "horrifying", "Are", "recommended", "Watch", "his", "when", "knowing", "He", "funny", "life", "original", "than", "guy", "love", "him", "not", "by", "good", "kidnaps", "the", "unfortunately", "Okay", "no", "Try", "Vallavan", "as", "all", "what", "Now", "because", "could", "you", "acting", "on", "nuts", "sh", "Raja", "How", "br", "female", "that", "more", "The", "give", "miss", "bozo"]).into_iter().collect(),
            },
            ReaderToWordsTestCase {
                content: "oh god..please save the people who has seen this \"comical acting, fatuous direction, futile story, insane dialogues,etc\" which gave a non-stop unbelievable maligning experience till the end..people watching this will go into an irrecoverable coma..it was an harrowing experience..<br /><br />the director-what is he trying to make, it looks as though he was completely out of his mind..the viewers will not condone this piece of work..unfortunately this movie has the same director, story-writer, and also the leading actor..at least this time the blame goes to single person..simbhu takes the sole responsibility for making life difficult for viewers..he has given importance in exposing extreme vulgarity..<br /><br />music-this is the only good thing about this movie..it has some good numbers and good background score..but it doesn\'t make this rubbish watchable..<br /><br />nayanthara and reemasen - i pity them both..i feel that they would have done much better for any other movie..they were doing skin show in this entire movie..reemasen\'s acting was terrible..nayanthara has and always done glamorous roles..she does it so as to hide her acting inabilities..<br /><br />bottom line-what more to say..if you want to be peaceful please avoid this like dog sh**..if you want to terrorize yourself you are most welcome to watch this A-hole at the highest level..<br /><br />actually i would like to give it zero on 10..<br /><br />but the rating is 1/10",
                words: vec_to_owned(vec!["peaceful", "condone", "but", "single", "some", "entire", "people", "viewers", "would", "only", "director", "much", "goes", "10", "responsibility", "watch", "mind", "other", "were", "importance", "insane", "completely", "was", "does", "which", "time", "pity", "dog", "unbelievable", "score", "rating", "direction", "story", "about", "blame", "have", "seen", "go", "background", "of", "sole", "better", "hide", "who", "welcome", "actually", "will", "making", "any", "her", "trying", "roles", "also", "an", "movie", "doing", "doesn\'t", "given", "reemasen", "1", "it", "exposing", "zero", "i", "into", "are", "she", "glamorous", "hole", "same", "piece", "extreme", "bottom", "god", "save", "maligning", "to", "out", "at", "please", "in", "dialogues", "this", "A", "avoid", "terrorize", "numbers", "simbhu", "is", "watchable", "till", "looks", "difficult", "make", "yourself", "comical", "level", "thing", "has", "and", "for", "if", "feel", "he", "experience", "show", "a", "work", "most", "nayanthara", "like", "gave", "least", "rubbish", "life", "his", "always", "say", "fatuous", "watching", "not", "harrowing", "vulgarity", "coma", "good", "leading", "non", "the", "stop", "futile", "unfortunately", "inabilities", "takes", "writer", "they", "oh", "as", "what", "line", "actor", "skin", "irrecoverable", "you", "want", "acting", "on", "terrible", "sh", "them", "br", "end", "done", "music", "that", "reemasen\'s", "more", "be", "give", "both", "etc", "though", "person", "so", "highest"]).into_iter().collect(),
            }
        ];

        for case in test_cases {
            let words = reader_to_words(StringReader::new(case.content));
            assert!(words.is_ok(), "Result is not OK; case: {}", case.content);
            let words = words.unwrap();
            assert_eq!(words, case.words, "Word sets do not match; case: {}
            expected: {:?}
            actual: {:?}
            in expected, but not actual: {:?}
            in actual, but not expected: {:?}", case.content, case.words, 
            words, case.words.difference(&words), words.difference(&case.words))
        }
    }

    #[test]
    fn test_scan_words() {
        use ScanForWordsResult::*;

        impl PartialEq for ScanForWordsResult<'_> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::Words(l), Self::Words(r)) => 
                        l.leading_run == r.leading_run
                        && l.trailing_run == r.trailing_run
                        && vec_compare(&l.words, &r.words),
                    _ => core::mem::discriminant(self) == core::mem::discriminant(other),
                }
            }
        }

        struct ScanForWordsTestCase<'a> {
            string: &'a str,
            expected_result: ScanForWordsResult<'a>,
        }

        let test_cases = vec![
            ScanForWordsTestCase {
                string: "アニャ likes peanuts, ワクワク!",
                expected_result: Words(WordsWithAlphanumericRuns { 
                    words: vec_to_owned(vec!["アニャ", "likes", "peanuts", "ワクワク"]), 
                    leading_run: true, 
                    trailing_run: None,
                })
            },
            ScanForWordsTestCase {
                string: "!アニャ likes peanuts, ワクワク!",
                expected_result: Words(WordsWithAlphanumericRuns { 
                    words: vec_to_owned(vec!["アニャ", "likes", "peanuts", "ワクワク"]), 
                    leading_run: false, 
                    trailing_run: None,
                })
            },
            ScanForWordsTestCase {
                string: "アニャ likes peanuts, ワクワク",
                expected_result: Words(WordsWithAlphanumericRuns { 
                    words: vec_to_owned(vec!["アニャ", "likes", "peanuts"]), 
                    leading_run: true, 
                    trailing_run: Some("ワクワク"),
                })
            },
            ScanForWordsTestCase {
                string: "ワクワク",
                expected_result: SingleAlphanumericRun,
            },
            ScanForWordsTestCase {
                string: "!,==<>",
                expected_result: NoWords,
            },
            ScanForWordsTestCase {
                string: "let's play!",
                expected_result: Words(WordsWithAlphanumericRuns { 
                    words: vec_to_owned(vec!["let's", "play"]), 
                    leading_run: true, 
                    trailing_run: None,
                })
            },
        ];

        for case in test_cases {
            let res = scan_for_words_from_reader(&case.string);

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