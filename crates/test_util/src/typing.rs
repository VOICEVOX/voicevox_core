use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DurationExampleData {
    pub length: i64,

    pub phoneme_vector: Vec<i64>,

    pub result: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntonationExampleData {
    pub length: i64,

    pub vowel_phoneme_vector: Vec<i64>,
    pub consonant_phoneme_vector: Vec<i64>,
    pub start_accent_vector: Vec<i64>,
    pub end_accent_vector: Vec<i64>,
    pub start_accent_phrase_vector: Vec<i64>,
    pub end_accent_phrase_vector: Vec<i64>,

    pub result: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecodeExampleData {
    pub f0_length: i64,
    pub phoneme_size: i64,
    pub f0_vector: Vec<f32>,
    pub phoneme_vector: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleData {
    pub speaker_id: i64,

    pub duration: DurationExampleData,
    pub intonation: IntonationExampleData,
    pub decode: DecodeExampleData,
}
