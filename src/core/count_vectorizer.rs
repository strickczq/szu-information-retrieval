use crate::core::{cs_helper, Vocabulary};
use jieba_rs::Jieba;
use rust_stemmers::{Algorithm, Stemmer};
use sprs::{CsMat, CsMatView};

pub struct CountVectorizer {
    /// 词汇表
    vocab: Vocabulary,
    /// 词频 shape(nd, nt)
    data: CsMat<usize>,
    /// 词干提取器
    stemmer: Stemmer,
    /// 分词器
    jieba: Jieba,
}

impl CountVectorizer {
    pub fn new() -> Self {
        Self {
            vocab: Vocabulary::default(),
            data: CsMat::zero((0, 0)),
            stemmer: Stemmer::create(Algorithm::English),
            jieba: Jieba::new(),
        }
    }

    pub fn tokenize(&self, x: &[String]) -> Vec<Vec<String>> {
        x.iter()
            .map(|s| {
                let mut tokens = Vec::new();

                for token in self.jieba.cut_for_search(s, true) {
                    if token.chars().all(|c| c.is_whitespace()) {
                        continue; // 跳过空白字符
                    }

                    if token.chars().all(|c| c.is_ascii()) {
                        let token = token.to_lowercase(); // 转小写
                        tokens.push(self.stemmer.stem(&token).into_owned());
                    } else {
                        tokens.push(token.to_owned());
                    }
                }

                tokens
            })
            .collect()
    }

    /// 训练
    pub fn fit(&mut self, x: &[String]) {
        tracing::info!("[CountVectorizer] 开始训练 {} 个文档", x.len());
        let start_time = std::time::Instant::now();

        // 分词
        tracing::info!("[CountVectorizer] (1/3) 分词");
        let tokenized = self.tokenize(x);

        // 构建词汇表
        tracing::info!("[CountVectorizer] (2/3) 构建词汇表");
        self.vocab = Vocabulary::default(); // 清空词汇表
        for doc in &tokenized {
            for token in doc {
                self.vocab.insert(token);
            }
        }

        // 构建词频矩阵
        tracing::info!("[CountVectorizer] (3/3) 构建词频矩阵");
        self.data = self.count(tokenized);

        tracing::info!(
            "[CountVectorizer] 训练完成，用时 {:?}，词汇表大小 {}",
            start_time.elapsed(),
            self.vocab.len()
        );
    }

    pub fn get_data(&self) -> CsMatView<usize> {
        self.data.view()
    }

    /// 转换
    pub fn transform(&self, inputs: &[String]) -> CsMat<usize> {
        let tokenized = self.tokenize(inputs);
        self.count(tokenized)
    }

    fn count(&self, tokenized: Vec<Vec<String>>) -> CsMat<usize> {
        let mut vecs = vec![];
        for doc in tokenized {
            // 这个文档的词项id列表（会重复）
            let mut tokens = doc
                .into_iter()
                .filter_map(|token| self.vocab.id(token))
                .collect::<Vec<_>>();

            // 从小到大排序
            tokens.sort();

            // 计算词频
            let vec = cs_helper::cs_vec_count(self.vocab.len(), &tokens);
            vecs.push(vec);
        }

        cs_helper::cs_mat_from_cs_vecs(&vecs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jieba() {
        let jieba = Jieba::new();
        let s = "我来到北京清华大学";
        let words = jieba.cut(s, false);
        assert_eq!(words, vec!["我", "来到", "北京", "清华大学"]);
    }

    #[test]
    fn test_stemmer() {
        let stemmer = Stemmer::create(Algorithm::English);
        let s = "fruitlessly";
        let stemmed = stemmer.stem(s).into_owned();
        assert_eq!(stemmed, "fruitless");
    }

    #[test]
    fn test_tokenizer() {
        let cv = CountVectorizer::new();
        let s = vec!["我a b你".to_owned()];
        assert_eq!(cv.tokenize(&s), vec![vec!["我", "a", "b", "你"]]);
    }

    #[test]
    fn test_count_vectorizer() {
        let mut cv = CountVectorizer::new();
        let x = vec!["a b c", "a b", "a"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();
        cv.fit(&x);
        let data = cv.get_data();
        assert_eq!(data.indptr().as_slice().unwrap(), &[0, 3, 5, 6]);
        assert_eq!(data.indices(), &[0, 1, 2, 0, 1, 0]);
        assert_eq!(data.data(), &[1, 1, 1, 1, 1, 1]);
    }
}
