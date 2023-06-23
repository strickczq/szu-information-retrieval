use sprs::{CsMat, CsVec, CsVecView};
use std::cmp::Ordering;

/// 余弦相似度
/// * `v1`: 值为分数, shape(nt,)
/// * `v2`: 值为分数, shape(nt,)
pub fn cos_sim(v1: CsVecView<f64>, v2: CsVecView<f64>) -> f64 {
    v1.dot(&v2) / (v1.dot(&v1).sqrt() * v2.dot(&v2).sqrt())
}

/// 文档列表交集
/// * `v1`: 值为t是否在d中, shape(nt,)
/// * `v2`: 值为t是否在d中, shape(nt,)
pub fn intersection(v1: CsVecView<bool>, v2: CsVecView<bool>) -> CsVec<bool> {
    // 文档id列表
    let mut result = vec![];
    let mut i = 0;
    let mut j = 0;

    while i < v1.indices().len() && j < v2.indices().len() {
        let t1 = v1.indices()[i];
        let t2 = v2.indices()[j];

        match t1.cmp(&t2) {
            Ordering::Equal => {
                result.push(t1); // 交集, 保留文档id
                i += 1;
                j += 1;
            }
            Ordering::Less => {
                i += 1;
            }
            Ordering::Greater => {
                j += 1;
            }
        }
    }

    let len = result.len();
    CsVec::new(v1.dim(), result, vec![true; len])
}

/// 计算数量的稀疏向量
/// * `n`: 向量长度
/// * `elems`: 序号列表（重复、要求排好序）
pub fn cs_vec_count(n: usize, elems: &[usize]) -> CsVec<usize> {
    let mut indices = vec![];
    let mut data = vec![];
    for &e in elems {
        assert!(e < n);
        match indices.last() {
            Some(&last) if last == e => {
                *data.last_mut().unwrap() += 1;
            }
            _ => {
                indices.push(e);
                data.push(1);
            }
        }
    }
    CsVec::new(n, indices, data)
}

/// 从稀疏向量列表构造稀疏矩阵
/// * `vecs`: 稀疏向量列表
pub fn cs_mat_from_cs_vecs<T: Clone>(vecs: &[CsVec<T>]) -> CsMat<T> {
    assert!(!vecs.is_empty());
    let (rows, cols) = (vecs.len(), vecs[0].dim());

    let mut indptr = vec![0];
    let mut indices = vec![];
    let mut data = vec![];

    for vec in vecs {
        indices.extend(vec.indices().to_vec());
        data.extend(vec.data().to_vec());
        indptr.push(indices.len());
    }

    CsMat::new((rows, cols), indptr, indices, data)
}

#[cfg(test)]
mod tests {
    use crate::almost_eq::AlmostEq;

    use super::*;

    #[test]
    fn test_cos_sim() {
        let v1 = CsVec::new(3, vec![0, 1], vec![1.0, 1.0]);
        let v2 = CsVec::new(3, vec![1, 2], vec![1.0, 1.0]);
        assert!(cos_sim(v1.view(), v2.view()).almost_eq(&0.5, 1e-6));
    }

    #[test]
    fn test_intersection() {
        let v1 = CsVec::new(10, vec![1, 2, 5, 6, 9], vec![true; 5]);
        let v2 = CsVec::new(10, vec![1, 3, 5, 7, 9], vec![true; 5]);
        let v3 = intersection(v1.view(), v2.view());
        assert_eq!(v3.indices(), vec![1, 5, 9]);
        assert_eq!(v3.data(), vec![true; 3]);
    }

    #[test]
    fn test_cs_vec_count() {
        let tokens = vec![0, 0, 1, 1, 1, 2, 2, 2, 2];
        let vec = cs_vec_count(3, &tokens);
        assert_eq!(vec.indices(), vec![0, 1, 2]);
        assert_eq!(vec.data(), vec![2, 3, 4]);
    }

    #[test]
    fn test_cs_mat_from_cs_vecs() {
        let vecs = vec![
            CsVec::new(3, vec![0, 1], vec![1, 2]),
            CsVec::new(3, vec![1, 2], vec![3, 4]),
        ];
        let mat = cs_mat_from_cs_vecs(&vecs);

        assert_eq!(mat.rows(), 2);
        assert_eq!(mat.cols(), 3);
        assert_eq!(mat.indptr().as_slice().unwrap(), &[0, 2, 4]);
        assert_eq!(mat.indices(), &[0, 1, 1, 2]);
        assert_eq!(mat.data(), &[1, 2, 3, 4]);
    }
}
