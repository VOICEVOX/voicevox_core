const UNVOICE_SYMBOL: char = '_';
const ACCENT_SYMBOL: char = '\'';
const NOPAUSE_DELIMITER: char = '/';
const PAUSE_DELIMITER: char = '、';
const WIDE_INTERROGATION_MARK: char = '？';
const LOOP_LIMIT: usize = 300;

#[derive(Clone)]
struct MoraModel {
	text: String,
	consonant: Option<String>,
	consonant_length: Option<f32>,
	vowel: String,
	vowel_length: f32,
	pitch: f32,
}

struct AccentPhraseModel {
	moras: Vec<MoraModel>,
	accent: usize,
	pause_mora: Option<MoraModel>,
	is_interrogative: bool,
}

struct AudioQueryModel {
	accent_phrases: Vec<AccentPhraseModel>,
	speed_scale: f32,
	pitch_scale: f32,
	intonation_scale: f32,
	volume_scale: f32,
	pre_phoneme_length: f32,
	post_phoneme_length: f32,
	output_sampling_rate: u32,
	output_stereo: bool,
	kana: String,
}

const mora_list_minimum: [&'static str; 432] = [
	"ヴォ", "v", "o",
	"ヴェ", "v", "e",
	"ヴィ", "v", "i",
	"ヴァ", "v", "a",
	"ヴ", "v", "u",
	"ン", "", "N",
	"ワ", "w", "a",
	"ロ", "r", "o",
	"レ", "r", "e",
	"ル", "r", "u",
	"リョ", "ry", "o",
	"リュ", "ry", "u",
	"リャ", "ry", "a",
	"リェ", "ry", "e",
	"リ", "r", "i",
	"ラ", "r", "a",
	"ヨ", "y", "o",
	"ユ", "y", "u",
	"ヤ", "y", "a",
	"モ", "m", "o",
	"メ", "m", "e",
	"ム", "m", "u",
	"ミョ", "my", "o",
	"ミュ", "my", "u",
	"ミャ", "my", "a",
	"ミェ", "my", "e",
	"ミ", "m", "i",
	"マ", "m", "a",
	"ポ", "p", "o",
	"ボ", "b", "o",
	"ホ", "h", "o",
	"ペ", "p", "e",
	"ベ", "b", "e",
	"ヘ", "h", "e",
	"プ", "p", "u",
	"ブ", "b", "u",
	"フォ", "f", "o",
	"フェ", "f", "e",
	"フィ", "f", "i",
	"ファ", "f", "a",
	"フ", "f", "u",
	"ピョ", "py", "o",
	"ピュ", "py", "u",
	"ピャ", "py", "a",
	"ピェ", "py", "e",
	"ピ", "p", "i",
	"ビョ", "by", "o",
	"ビュ", "by", "u",
	"ビャ", "by", "a",
	"ビェ", "by", "e",
	"ビ", "b", "i",
	"ヒョ", "hy", "o",
	"ヒュ", "hy", "u",
	"ヒャ", "hy", "a",
	"ヒェ", "hy", "e",
	"ヒ", "h", "i",
	"パ", "p", "a",
	"バ", "b", "a",
	"ハ", "h", "a",
	"ノ", "n", "o",
	"ネ", "n", "e",
	"ヌ", "n", "u",
	"ニョ", "ny", "o",
	"ニュ", "ny", "u",
	"ニャ", "ny", "a",
	"ニェ", "ny", "e",
	"ニ", "n", "i",
	"ナ", "n", "a",
	"ドゥ", "d", "u",
	"ド", "d", "o",
	"トゥ", "t", "u",
	"ト", "t", "o",
	"デョ", "dy", "o",
	"デュ", "dy", "u",
	"デャ", "dy", "a",
	"ディ", "d", "i",
	"デ", "d", "e",
	"テョ", "ty", "o",
	"テュ", "ty", "u",
	"テャ", "ty", "a",
	"ティ", "t", "i",
	"テ", "t", "e",
	"ツォ", "ts", "o",
	"ツェ", "ts", "e",
	"ツィ", "ts", "i",
	"ツァ", "ts", "a",
	"ツ", "ts", "u",
	"ッ", "", "cl",
	"チョ", "ch", "o",
	"チュ", "ch", "u",
	"チャ", "ch", "a",
	"チェ", "ch", "e",
	"チ", "ch", "i",
	"ダ", "d", "a",
	"タ", "t", "a",
	"ゾ", "z", "o",
	"ソ", "s", "o",
	"ゼ", "z", "e",
	"セ", "s", "e",
	"ズィ", "z", "i",
	"ズ", "z", "u",
	"スィ", "s", "i",
	"ス", "s", "u",
	"ジョ", "j", "o",
	"ジュ", "j", "u",
	"ジャ", "j", "a",
	"ジェ", "j", "e",
	"ジ", "j", "i",
	"ショ", "sh", "o",
	"シュ", "sh", "u",
	"シャ", "sh", "a",
	"シェ", "sh", "e",
	"シ", "sh", "i",
	"ザ", "z", "a",
	"サ", "s", "a",
	"ゴ", "g", "o",
	"コ", "k", "o",
	"ゲ", "g", "e",
	"ケ", "k", "e",
	"グヮ", "gw", "a",
	"グ", "g", "u",
	"クヮ", "kw", "a",
	"ク", "k", "u",
	"ギョ", "gy", "o",
	"ギュ", "gy", "u",
	"ギャ", "gy", "a",
	"ギェ", "gy", "e",
	"ギ", "g", "i",
	"キョ", "ky", "o",
	"キュ", "ky", "u",
	"キャ", "ky", "a",
	"キェ", "ky", "e",
	"キ", "k", "i",
	"ガ", "g", "a",
	"カ", "k", "a",
	"オ", "", "o",
	"エ", "", "e",
	"ウォ", "w", "o",
	"ウェ", "w", "e",
	"ウィ", "w", "i",
	"ウ", "", "u",
	"イェ", "y", "e",
	"イ", "", "i",
	"ア", "", "a"
];

fn mora2text(mora: &str) -> &str {
	let (c, v) = mora.split_at(mora.len() - 1);
	for chunk in mora_list_minimum.chunks(3) {
		let text = chunk[0];
		let consonant = chunk[1];
		let vowel = chunk[2];
		if consonant == c && vowel == v {
			return text
		}
	}
	mora
}

fn text2mora_with_unvioce() -> std::collections::BTreeMap<String, MoraModel> {
	let mut text2mora_with_unvioce = std::collections::BTreeMap::new();
	for chunk in mora_list_minimum.chunks(3) {
		let text = chunk[0];
		let consonant = chunk[1];
		let vowel = chunk[2];
		let consonant = if consonant != "" { Some(consonant.to_string()) } else { None };
		let consonant_length = if consonant.is_some() { Some(0.0) } else { None };

		if ["a", "i", "u", "e", "o"].contains(&vowel) {
			let upper_vowel = vowel.chars().next().unwrap().to_uppercase().to_string();
			let unvoice_mora = MoraModel {
				text: text.to_string(),
				consonant: consonant.clone(),
				consonant_length: consonant_length.clone(),
				vowel: upper_vowel,
				vowel_length: 0.0,
				pitch: 0.0,
			};
			text2mora_with_unvioce.insert(UNVOICE_SYMBOL.to_string() + text, unvoice_mora);
		}

		let mora = MoraModel {
			text: text.to_string(),
			consonant,
			consonant_length,
			vowel: vowel.to_string(),
			vowel_length: 0.0,
			pitch: 0.0,
		};
		text2mora_with_unvioce.insert(text.to_string(), mora);
	}
	text2mora_with_unvioce
}

fn text_to_accent_phrase(phrase: &str) -> AccentPhraseModel {
	let phrase_vec: Vec<char> = phrase.chars().collect();
	let mut accent_index: Option<usize> = None;
	let mut moras: Vec<MoraModel> = Vec::new();
	let mut stack = String::new();
	let mut matched_text: Option<String> = None;
	let text2mora = text2mora_with_unvioce();
	let mut index = 0;
	let mut loop_count = 0;
	while index < phrase_vec.len() {
		loop_count += 1;
		let letter = phrase_vec[index];
		if letter == ACCENT_SYMBOL {
			panic!("accent cannot be set at beginning of accent phrase: {}", phrase);
		}
		if accent_index.is_some() {
			panic!("second accent cannot be set at an accent phrase: {}", phrase);
		}
		accent_index = Some(moras.len());

		for &watch_letter in &phrase_vec[index..] {
			if watch_letter == ACCENT_SYMBOL {
				break;
			}
			stack.push(watch_letter);
			if text2mora.contains_key(&stack) {
				matched_text = Some(stack.clone());
			}
		}
		if let Some(matched_text) = matched_text.take() {
			index += matched_text.chars().count();
			moras.push(text2mora.get(&matched_text).unwrap().clone());
			stack.clear();
		} else {
			panic!("unknown text in accent phrase: {}", phrase);
		}
		if loop_count > LOOP_LIMIT {
			panic!("detected infinity loop!");
		}
	}
	if accent_index.is_none() {
		panic!("accent not found in accent phrase: {}", phrase);
	}
	AccentPhraseModel {
		moras,
		accent: accent_index.unwrap(),
		pause_mora: None,
		is_interrogative: false
	}
}