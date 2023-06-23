use sprs::{CsMat, CsMatView};

#[derive(Debug)]
pub struct TfidfVectorizer {
    /// 逆文档频率, shape(nt,)
    idf: Vec<f64>,
    /// tf-idf, shape(nd,nt)
    tf_idf: CsMat<f64>,
}

impl TfidfVectorizer {
    pub fn new() -> Self {
        Self {
            idf: vec![],
            tf_idf: CsMat::zero((0, 0)),
        }
    }

    /// * `x`: 所有文档词频, shape(nd,nt)
    pub fn fit(&mut self, x: CsMatView<usize>) {
        tracing::info!("[TfidfVectorizer] 计算 IDF");
        let start_time = std::time::Instant::now();

        let (nd, nt) = x.shape();

        // 计算逆文档频率 Inverse Document Frequency
        // idf(t) = log10(N / df(t))
        // 分子：语料库中的文档总数
        // 分母：包含词项t的文档数
        let mut count = vec![0; nt];
        for (&v, (_d, t)) in x {
            if v > 0 {
                count[t] += 1;
            }
        }

        self.idf = vec![0.0; nt];
        for t in 0..nt {
            self.idf[t] = (nd as f64 / count[t] as f64).log10();
        }

        tracing::info!(
            "[TfidfVectorizer] IDF 计算完成，用时 {:?}",
            start_time.elapsed()
        );

        self.tf_idf = self.transform(x);
    }

    pub fn get_tf_idf(&self) -> CsMatView<f64> {
        self.tf_idf.view()
    }

    /// * `x`: 所有文档词频, shape(nd,nt)
    /// * `returns`: 所有文档向量, 值为 TF-IDF, shape(nd,nt)
    pub fn transform(&self, x: CsMatView<usize>) -> CsMat<f64> {
        tracing::info!("[TfidfVectorizer] 计算 TF-IDF");
        let start_time = std::time::Instant::now();

        let (nd, nt) = x.shape();

        // 每个文档的所有字词的出现次数之和
        let mut n_tokens = vec![0; nd];
        for (&v, (d, _t)) in x {
            n_tokens[d] += v;
        }

        // 将词频复制一份，转换为 f64
        let indptr = x.indptr().as_slice().unwrap().to_owned();
        let indices = x.indices().to_owned();
        let mut data = x.data().iter().map(|&v| v as f64).collect::<Vec<_>>();

        for d in 0..nd {
            for i in indptr[d]..indptr[d + 1] {
                let t = indices[i];
                // 计算词频 Term Frequency
                // tf(t, d) = f(t, d) / |d|
                // 分子：词项t在文档d中出现的次数
                // 分母：文档d中的所有字词的出现次数之和
                let tf = data[i] / n_tokens[d] as f64;

                // 填入tf(t, d)*idf(t)
                data[i] = tf * self.idf[t];
            }
        }

        // tf-idf 矩阵
        let mat = CsMat::new((nd, nt), indptr, indices, data);

        tracing::info!(
            "[TfidfVectorizer] 计算完成，用时 {:?}",
            start_time.elapsed()
        );

        mat
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::almost_eq::AlmostEq;
    use sprs::CsMat;

    #[test]
    fn test_tf_idf() {
        let mut freq = CsMat::zero((3, 3));
        freq.insert(0, 0, 1);
        freq.insert(1, 1, 1);
        freq.insert(2, 2, 1);

        let mut vsm = TfidfVectorizer::new();
        vsm.fit(freq.view());
        let score = vsm.transform(freq.view());

        let mut expected = CsMat::zero((3, 3));
        expected.insert(0, 0, 0.47712125471966244);
        expected.insert(1, 1, 0.47712125471966244);
        expected.insert(2, 2, 0.47712125471966244);

        assert!(score.to_dense().almost_eq(&expected.to_dense(), 1e-6));
    }
}
