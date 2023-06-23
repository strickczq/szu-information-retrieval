use crate::{core::cs_helper, dataset::Doc, AppState};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchParams {
    pub keyword: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub filter: Option<SearchParamsFilter>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchParamsFilter {
    pub infotype: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub total_hits: usize,
    pub hits: Vec<Hit>,
    pub time: u128,
    pub keyword: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hit {
    pub id: usize,
    pub score: f64,
    pub doc: Doc,
}

pub async fn handler(
    State(AppState { dataset, core }): State<AppState>,
    Json(SearchParams {
        keyword,
        offset,
        limit,
        filter,
    }): Json<SearchParams>,
) -> Json<SearchResult> {
    tracing::info!("[Search] 开始搜索: {:?}", keyword);
    let start_time = std::time::Instant::now();

    let limit = limit.unwrap_or(10);
    let offset = offset.unwrap_or(0);

    // 搜索词词频矩阵（只有一行）
    let search_count = core.count_vectorizer.transform(&[keyword.clone()]);
    // 搜索词 TF-IDF 矩阵（只有一行）
    let search_tf_idf = core.tfidf_vectorizer.transform(search_count.view());

    // 搜索结果（文档向量）
    let searched_doc_vec = core.index.search(search_count.outer_view(0).unwrap());

    // 计算得分: Vec<(d, score)>
    let mut d_score = searched_doc_vec
        .iter()
        .map(|(d, _)| {
            // 计算相似度
            let score = cs_helper::cos_sim(
                search_tf_idf.outer_view(0).unwrap(),
                core.tfidf_vectorizer.get_tf_idf().outer_view(d).unwrap(),
            );
            (d, score)
        })
        .collect::<Vec<_>>();

    // 按得分排序
    d_score.sort_by(|(_, s1), (_, s2)| s2.partial_cmp(s1).unwrap());

    // 过滤
    if let Some(filter) = filter {
        if let Some(infotype) = &filter.infotype {
            tracing::info!("[Search] 过滤 infotype: {:?}", infotype);
        }
        if let Some(user) = &filter.user {
            tracing::info!("[Search] 过滤 user: {:?}", user);
        }

        d_score = d_score
            .into_iter()
            .filter(|&(d, _)| {
                let doc = &dataset.docs[d];
                if let Some(infotype) = &filter.infotype {
                    if infotype != &doc.infotype {
                        return false;
                    }
                }
                if let Some(user) = &filter.user {
                    if user != &doc.user {
                        return false;
                    }
                }

                true
            })
            .collect::<Vec<_>>();
    }

    // 总命中数
    let total_hits = d_score.len();

    // 截断前 top_n 个
    let d_score = d_score
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    let hits = d_score
        .iter()
        .map(|&(d, score)| Hit {
            id: d,
            score,
            doc: dataset.docs[d].clone(),
        })
        .collect::<Vec<_>>();

    let time = start_time.elapsed().as_millis();

    tracing::info!("[Search] 完成搜索 {:?}，耗时 {} ms", keyword, time);

    Json(SearchResult {
        total_hits,
        hits,
        time,
        keyword,
    })
}
