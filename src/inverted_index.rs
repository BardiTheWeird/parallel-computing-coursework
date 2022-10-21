use std::{collections::{HashSet, HashMap}, path::{Path, PathBuf}, rc::Rc};

use chashmap::CHashMap;
use log::debug;
use serde::{Serialize, Deserialize};

use crate::{word_stemming::stem_words, word_filtering::scan_for_unique_words};

#[derive(Debug)]
pub struct InvertedIndex {
    hashmap: CHashMap<String, HashSet<Rc<String>>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    document: String,
    rank: usize,
}

impl InvertedIndex {
    pub fn insert(&mut self, document: String, words: HashSet<String>) {
        let words = stem_words(words.into_iter());
        let document = Rc::new(document);

        let insert = ||
            vec![Rc::clone(&document)].into_iter().collect();
        let update = |old: &mut HashSet<Rc<String>>| {
            old.insert(Rc::clone(&document)); };
        
        for word in words {
            self.hashmap.upsert(word, &insert, update);
        }
    }

    pub fn query(&self, query: &str) -> Vec<QueryResult> {
        debug!("processing inverse_index query `{}`", query);
        let words = match scan_for_unique_words(query) {
            Some(w) => w,
            None => return vec![],
        };
        debug!("words found in `{}`: {:?}", query, words);

        let mut v: Vec<QueryResult> = words.into_iter()
            .filter_map(|s| {
                self.hashmap.get(&s)
                    .and_then(|v| Some((*v).clone()))
            }).fold(HashMap::<String, usize>::new(), |mut accum, item| {
                for word in item {
                    accum.entry(word.to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                accum
            }).into_iter().map(|(key, rank)| QueryResult{document: key, rank})
            .collect();
        v.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
        debug!("result for query `{}`: {:?}", query, &v);
        v
    }

    pub fn new() -> Self {
        Self { hashmap: CHashMap::new() }
    }
}