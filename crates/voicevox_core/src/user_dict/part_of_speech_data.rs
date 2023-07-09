use derive_getters::Getters;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::UserDictWordType;

/// 最小の優先度
pub static MIN_PRIORITY: u32 = 0;
/// 最大の優先度
pub static MAX_PRIORITY: u32 = 10;

/// 品詞ごとの情報
#[derive(Debug, Getters)]
pub struct PartOfSpeechDetail {
    /// 品詞
    pub part_of_speech: String,
    /// 品詞細分類1
    pub part_of_speech_detail_1: String,
    /// 品詞細分類2
    pub part_of_speech_detail_2: String,
    /// 品詞細分類3
    pub part_of_speech_detail_3: String,
    /// 文脈IDは辞書の左・右文脈IDのこと
    /// 参考: https://github.com/VOICEVOX/open_jtalk/blob/427cfd761b78efb6094bea3c5bb8c968f0d711ab/src/mecab-naist-jdic/_left-id.def
    pub context_id: i32,
    /// コストのパーセンタイル
    pub cost_candidates: Vec<i32>,
    /// アクセント結合規則の一覧
    pub accent_associative_rules: Vec<String>,
}

pub static PART_OF_SPEECH_DETAIL: Lazy<HashMap<UserDictWordType, PartOfSpeechDetail>> =
    Lazy::new(|| {
        HashMap::from_iter([
            (
                UserDictWordType::ProperNoun,
                PartOfSpeechDetail {
                    part_of_speech: "名詞".to_string(),
                    part_of_speech_detail_1: "固有名詞".to_string(),
                    part_of_speech_detail_2: "一般".to_string(),
                    part_of_speech_detail_3: "*".to_string(),
                    context_id: 1348,
                    cost_candidates: vec![
                        -988, 3488, 4768, 6048, 7328, 8609, 8734, 8859, 8984, 9110, 14176,
                    ],
                    accent_associative_rules: vec![
                        "*".to_string(),
                        "C1".to_string(),
                        "C2".to_string(),
                        "C3".to_string(),
                        "C4".to_string(),
                        "C5".to_string(),
                    ],
                },
            ),
            (
                UserDictWordType::CommonNoun,
                PartOfSpeechDetail {
                    part_of_speech: "名詞".to_string(),
                    part_of_speech_detail_1: "一般".to_string(),
                    part_of_speech_detail_2: "*".to_string(),
                    part_of_speech_detail_3: "*".to_string(),
                    context_id: 1345,
                    cost_candidates: vec![
                        -4445, 49, 1473, 2897, 4321, 5746, 6554, 7362, 8170, 8979, 15001,
                    ],
                    accent_associative_rules: vec![
                        "*".to_string(),
                        "C1".to_string(),
                        "C2".to_string(),
                        "C3".to_string(),
                        "C4".to_string(),
                        "C5".to_string(),
                    ],
                },
            ),
            (
                UserDictWordType::Verb,
                PartOfSpeechDetail {
                    part_of_speech: "動詞".to_string(),
                    part_of_speech_detail_1: "自立".to_string(),
                    part_of_speech_detail_2: "*".to_string(),
                    part_of_speech_detail_3: "*".to_string(),
                    context_id: 642,
                    cost_candidates: vec![
                        3100, 6160, 6360, 6561, 6761, 6962, 7414, 7866, 8318, 8771, 13433,
                    ],
                    accent_associative_rules: vec!["*".to_string()],
                },
            ),
            (
                UserDictWordType::Adjective,
                PartOfSpeechDetail {
                    part_of_speech: "形容詞".to_string(),
                    part_of_speech_detail_1: "自立".to_string(),
                    part_of_speech_detail_2: "*".to_string(),
                    part_of_speech_detail_3: "*".to_string(),
                    context_id: 20,
                    cost_candidates: vec![
                        1527, 3266, 3561, 3857, 4153, 4449, 5149, 5849, 6549, 7250, 10001,
                    ],
                    accent_associative_rules: vec!["*".to_string()],
                },
            ),
            (
                UserDictWordType::Suffix,
                PartOfSpeechDetail {
                    part_of_speech: "名詞".to_string(),
                    part_of_speech_detail_1: "接尾".to_string(),
                    part_of_speech_detail_2: "一般".to_string(),
                    part_of_speech_detail_3: "*".to_string(),
                    context_id: 1358,
                    cost_candidates: vec![
                        4399, 5373, 6041, 6710, 7378, 8047, 9440, 10834, 12228, 13622, 15847,
                    ],
                    accent_associative_rules: vec![
                        "*".to_string(),
                        "C1".to_string(),
                        "C2".to_string(),
                        "C3".to_string(),
                        "C4".to_string(),
                        "C5".to_string(),
                    ],
                },
            ),
        ])
    });

fn search_cost_candidates(context_id: i32) -> Vec<i32> {
    PART_OF_SPEECH_DETAIL
        .values()
        .find(|&x| x.context_id == context_id)
        .expect("品詞IDが不正です")
        .cost_candidates
        .clone()
}

pub fn priority2cost(context_id: i32, priority: u32) -> i32 {
    let cost_candidates = search_cost_candidates(context_id);
    cost_candidates[(MAX_PRIORITY - priority) as usize]
}
