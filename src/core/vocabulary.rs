use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Vocabulary {
    pub set: HashSet<String>,
    pub id_map: HashMap<String, usize>,
    pub term_map: HashMap<usize, String>,
}

impl Vocabulary {
    #[inline]
    pub fn insert(&mut self, token: impl ToString) {
        let word = token.to_string();
        if self.set.insert(word.clone()) {
            let id = self.set.len() - 1;
            self.id_map.insert(word.clone(), id);
            self.term_map.insert(id, word);
        }
    }

    #[inline]
    pub fn id(&self, word: impl AsRef<str>) -> Option<usize> {
        let word = word.as_ref();
        self.id_map.get(word).copied()
    }

    #[inline]
    pub fn word(&self, id: usize) -> &str {
        &self.term_map[&id]
    }

    /// 词汇量
    #[inline]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}
