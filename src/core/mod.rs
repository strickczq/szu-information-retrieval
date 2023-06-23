mod count_vectorizer;
pub mod cs_helper;
mod inverted_index;
mod tfidf_vectorizer;
mod vocabulary;

use crate::dataset::Dataset;
pub use count_vectorizer::*;
pub use inverted_index::*;
pub use tfidf_vectorizer::*;
pub use vocabulary::*;

pub struct Core {
    pub count_vectorizer: CountVectorizer,
    pub index: InvertedIndex,
    pub tfidf_vectorizer: TfidfVectorizer,
}

impl Core {
    pub fn new(dataset: &Dataset) -> anyhow::Result<Self> {
        // 可以搜索的内容
        let text_for_search = dataset
            .docs
            .iter()
            .map(|doc| {
                format!(
                    "{}\n{}\n{}",
                    doc.title,
                    doc.text,
                    doc.attachments
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>();

        let mut count_vectorizer = CountVectorizer::new();
        count_vectorizer.fit(&text_for_search);

        let index = InvertedIndex::build(count_vectorizer.get_data());

        let mut tfidf_vectorizer = TfidfVectorizer::new();
        tfidf_vectorizer.fit(count_vectorizer.get_data());

        Ok(Self {
            count_vectorizer,
            index,
            tfidf_vectorizer,
        })
    }
}
