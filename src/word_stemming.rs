use std::collections::HashSet;

pub fn stem_words(words: impl Iterator<Item = String>) -> HashSet<String> {
    words.map(|s| word_to_stem(s)).collect()
}

fn word_to_stem(s: String) -> String {
    s
}