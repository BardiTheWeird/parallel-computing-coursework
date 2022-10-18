use std::{collections::HashSet, path::{Path, PathBuf}};

use chashmap::CHashMap;

use crate::word_stemming::stem_words;

pub struct InvertedIndex {
    hashmap: CHashMap<PathBuf, HashSet<String>>
}

pub struct QueryResult<'a> {
    key: &'a PathBuf,
    rank: usize,
}

impl InvertedIndex {
    pub fn insert(&mut self, key: PathBuf, words: HashSet<String>) {
        let words = stem_words(words.into_iter());
        self.hashmap.insert(key, words);
    }

    pub fn query(&self, words: HashSet<String>) -> Vec<QueryResult> {
        todo!()
    }

    pub fn new() -> Self {
        Self { hashmap: CHashMap::new() }
    }
}