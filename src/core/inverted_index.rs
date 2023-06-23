use sprs::{CsMat, CsMatView, CsVec, CsVecView};

use crate::core::cs_helper;

#[derive(Debug)]
pub struct InvertedIndex {
    /// 倒排索引，值表示存在性，shape(nt,nd)
    data: CsMat<bool>,
}

impl InvertedIndex {
    /// * `x`: 所有文档词频, shape(nd,nt)
    pub fn build(x: CsMatView<usize>) -> InvertedIndex {
        tracing::info!("[InvertedIndex] 开始构建索引");
        let start_time = std::time::Instant::now();

        let data = x.map(|&v| v > 0).transpose_into().into_csr();

        tracing::info!(
            "[InvertedIndex] 构建索引完成，用时 {:?}，稀疏度 {}",
            start_time.elapsed(),
            data.density()
        );
        InvertedIndex { data }
    }

    /// * `x`: 文档词频, shape(nt,)
    /// * `returns`: 包含词项t的文档, shape(nd,)
    pub fn search(&self, x: CsVecView<usize>) -> CsVec<bool> {
        tracing::info!("[InvertedIndex] 搜索");
        let start_time = std::time::Instant::now();

        let (_nt, nd) = self.data.shape();
        let mut result: Option<CsVec<bool>> = None;

        // 遍历 nnz 的词项
        for (t, &v) in x.iter() {
            if v == 0 {
                continue; // 跳过词频为 0 的词项 (其实理论上稀疏矩阵里不会有，但是稀疏矩阵确实可以存储 0，也算做 nnz)
            }

            // 取出 t 对应的文档列表 (Posting List)
            let docs = self.data.outer_view(t).unwrap().to_owned();

            result = Some(match result {
                Some(old) => cs_helper::intersection(old.view(), docs.view()),
                None => docs,
            })
        }
        tracing::info!("[InvertedIndex] 搜索完成，用时 {:?}", start_time.elapsed());

        result.unwrap_or(CsVec::new(nd, vec![], vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprs::CsVec;

    #[test]
    fn test_inverted_index() {
        let x = cs_helper::cs_mat_from_cs_vecs(&[
            CsVec::new(3, vec![0, 1], vec![1, 1]),
            CsVec::new(3, vec![1, 2], vec![1, 1]),
            CsVec::new(3, vec![0, 2], vec![1, 1]),
            CsVec::new(3, vec![0, 1, 2], vec![1, 1, 1]),
        ]);
        let index = InvertedIndex::build(x.view());

        let x = CsVec::new(3, vec![0, 1], vec![1, 1]);
        let result = index.search(x.view());
        assert_eq!(result.nnz(), 2);
        assert_eq!(result.indices(), vec![0, 3]);
        assert_eq!(result.data(), vec![true, true]);
    }
}
