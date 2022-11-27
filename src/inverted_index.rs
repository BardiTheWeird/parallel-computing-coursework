use std::{collections::{HashSet, HashMap}, sync::Arc};

use chashmap::CHashMap;
use log::debug;
use serde::{Serialize, Deserialize};

use crate::{word_filtering::scan_for_unique_words};

#[derive(Debug)]
pub struct InvertedIndex {
    hashmap: CHashMap<String, HashSet<Arc<String>>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    document: String,
    rank: usize,
}

impl InvertedIndex {
    pub fn insert(&self, document: String, words: HashSet<String>) {
        let stems = Self::words_to_stems(words);
        let document = Arc::new(document);

        let insert = ||
            vec![Arc::clone(&document)].into_iter().collect();
        let update = |old: &mut HashSet<Arc<String>>| {
            old.insert(Arc::clone(&document)); };
        
        for stem in stems {
            self.hashmap.upsert(stem, &insert, update);
        }
    }

    pub fn query(&self, query: &str) -> Vec<QueryResult> {
        debug!("processing inverse_index query `{}`", query);
        let words = match scan_for_unique_words(query) {
            Some(w) => w,
            None => return vec![],
        };
        debug!("words found in `{}`: {:?}", query, words);
        let stems = Self::words_to_stems(words);
        debug!("stems found in `{}`: {:?}", query, stems);

        let mut v: Vec<QueryResult> = stems.into_iter()
            .filter_map(|s| {
                self.hashmap.get(&s)
                    .and_then(|v| Some((*v).clone()))
            }).fold(HashMap::<String, usize>::new(), |mut accum, item| {
                for stem in item {
                    accum.entry(stem.to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                accum
            }).into_iter().map(|(key, rank)| QueryResult{document: key, rank})
            .collect();
        v.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
        v
    }

    pub fn new() -> Self {
        Self { hashmap: CHashMap::new() }
    }

    fn words_to_stems(words: HashSet<String>) -> HashSet<String> {
        words.into_iter()
            .map(|w| w.to_lowercase())
            .map(|w| porter_stemmer::stem(&w))
            .collect()
    }
}