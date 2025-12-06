//  以下のモーラ対応表はOpenJTalkのソースコードから取得し、
//  カタカナ表記とモーラが一対一対応するように改造した。
//  ライセンス表記：
//  ---------------------------------------------------------------- -
//            The Japanese TTS System "Open JTalk"
//            developed by HTS Working Group
//            http ://open-jtalk.sourceforge.net/
//  ---------------------------------------------------------------- -
//
//   Copyright (c) 2008 - 2014  Nagoya Institute of Technology
//                              Department of Computer Science
//
//  All rights reserved.
//
//  Redistribution and use in source and binary forms, with or
//  without modification, are permitted provided that the following
//  conditions are met :
//
//  - Redistributions of source code must retain the above copyright
//    notice, this list of conditionsand the following disclaimer.
//  - Redistributions in binary form must reproduce the above
//    copyright notice, this list of conditionsand the following
//    disclaimer in the documentationand /or other materials provided
//    with the distribution.
//  - Neither the name of the HTS working group nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND
//  CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
//  MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED.IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS
//  BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
//  EXEMPLARY, OR CONSEQUENTIAL DAMAGES(INCLUDING, BUT NOT LIMITED
//  TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
//  DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
//  ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
//  OR TORT(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
//  OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
//  POSSIBILITY OF SUCH DAMAGE.

use enum_map::{Enum, EnumMap};
use macros::MoraMappings;
use strum::{EnumCount, EnumString, IntoStaticStr};

use super::acoustic_feature_extractor::{MoraTail, OptionalConsonant};

#[derive(
    Clone, Copy, PartialEq, Debug, Enum, EnumCount, EnumString, IntoStaticStr, MoraMappings,
)]
#[mora_mappings(
    mora_phonemes_to_mora_kana {
        pub(super) static MORA_PHONEMES_TO_MORA_KANA: phf::Map<&str, MoraKana> = _;
    }
    mora_kana_to_mora_phonemes {
        pub(super) static MORA_KANA_TO_MORA_PHONEMES: EnumMap<
            MoraKana,
            (OptionalConsonant, MoraTail),
        > = _;
    }
)]
pub(super) enum MoraKana {
    #[mora_mappings("v", "o")]
    ヴォ,

    #[mora_mappings("v", "e")]
    ヴェ,

    #[mora_mappings("v", "i")]
    ヴィ,

    #[mora_mappings("v", "a")]
    ヴァ,

    #[mora_mappings("v", "u")]
    ヴ,

    #[mora_mappings("", "N")]
    ン,

    #[mora_mappings("w", "a")]
    ワ,

    #[mora_mappings("r", "o")]
    ロ,

    #[mora_mappings("r", "e")]
    レ,

    #[mora_mappings("r", "u")]
    ル,

    #[mora_mappings("ry", "o")]
    リョ,

    #[mora_mappings("ry", "u")]
    リュ,

    #[mora_mappings("ry", "a")]
    リャ,

    #[mora_mappings("ry", "e")]
    リェ,

    #[mora_mappings("r", "i")]
    リ,

    #[mora_mappings("r", "a")]
    ラ,

    #[mora_mappings("y", "o")]
    ヨ,

    #[mora_mappings("y", "u")]
    ユ,

    #[mora_mappings("y", "a")]
    ヤ,

    #[mora_mappings("m", "o")]
    モ,

    #[mora_mappings("m", "e")]
    メ,

    #[mora_mappings("m", "u")]
    ム,

    #[mora_mappings("my", "o")]
    ミョ,

    #[mora_mappings("my", "u")]
    ミュ,

    #[mora_mappings("my", "a")]
    ミャ,

    #[mora_mappings("my", "e")]
    ミェ,

    #[mora_mappings("m", "i")]
    ミ,

    #[mora_mappings("m", "a")]
    マ,

    #[mora_mappings("p", "o")]
    ポ,

    #[mora_mappings("b", "o")]
    ボ,

    #[mora_mappings("h", "o")]
    ホ,

    #[mora_mappings("p", "e")]
    ペ,

    #[mora_mappings("b", "e")]
    ベ,

    #[mora_mappings("h", "e")]
    ヘ,

    #[mora_mappings("p", "u")]
    プ,

    #[mora_mappings("b", "u")]
    ブ,

    #[mora_mappings("f", "o")]
    フォ,

    #[mora_mappings("f", "e")]
    フェ,

    #[mora_mappings("f", "i")]
    フィ,

    #[mora_mappings("f", "a")]
    ファ,

    #[mora_mappings("f", "u")]
    フ,

    #[mora_mappings("py", "o")]
    ピョ,

    #[mora_mappings("py", "u")]
    ピュ,

    #[mora_mappings("py", "a")]
    ピャ,

    #[mora_mappings("py", "e")]
    ピェ,

    #[mora_mappings("p", "i")]
    ピ,

    #[mora_mappings("by", "o")]
    ビョ,

    #[mora_mappings("by", "u")]
    ビュ,

    #[mora_mappings("by", "a")]
    ビャ,

    #[mora_mappings("by", "e")]
    ビェ,

    #[mora_mappings("b", "i")]
    ビ,

    #[mora_mappings("hy", "o")]
    ヒョ,

    #[mora_mappings("hy", "u")]
    ヒュ,

    #[mora_mappings("hy", "a")]
    ヒャ,

    #[mora_mappings("hy", "e")]
    ヒェ,

    #[mora_mappings("h", "i")]
    ヒ,

    #[mora_mappings("p", "a")]
    パ,

    #[mora_mappings("b", "a")]
    バ,

    #[mora_mappings("h", "a")]
    ハ,

    #[mora_mappings("n", "o")]
    ノ,

    #[mora_mappings("n", "e")]
    ネ,

    #[mora_mappings("n", "u")]
    ヌ,

    #[mora_mappings("ny", "o")]
    ニョ,

    #[mora_mappings("ny", "u")]
    ニュ,

    #[mora_mappings("ny", "a")]
    ニャ,

    #[mora_mappings("ny", "e")]
    ニェ,

    #[mora_mappings("n", "i")]
    ニ,

    #[mora_mappings("n", "a")]
    ナ,

    #[mora_mappings("d", "u")]
    ドゥ,

    #[mora_mappings("d", "o")]
    ド,

    #[mora_mappings("t", "u")]
    トゥ,

    #[mora_mappings("t", "o")]
    ト,

    #[mora_mappings("dy", "o")]
    デョ,

    #[mora_mappings("dy", "u")]
    デュ,

    #[mora_mappings("dy", "a")]
    デャ,

    #[mora_mappings("d", "i")]
    ディ,

    #[mora_mappings("d", "e")]
    デ,

    #[mora_mappings("ty", "o")]
    テョ,

    #[mora_mappings("ty", "u")]
    テュ,

    #[mora_mappings("ty", "a")]
    テャ,

    #[mora_mappings("t", "i")]
    ティ,

    #[mora_mappings("t", "e")]
    テ,

    #[mora_mappings("ts", "o")]
    ツォ,

    #[mora_mappings("ts", "e")]
    ツェ,

    #[mora_mappings("ts", "i")]
    ツィ,

    #[mora_mappings("ts", "a")]
    ツァ,

    #[mora_mappings("ts", "u")]
    ツ,

    #[mora_mappings("", "cl")]
    ッ,

    #[mora_mappings("ch", "o")]
    チョ,

    #[mora_mappings("ch", "u")]
    チュ,

    #[mora_mappings("ch", "a")]
    チャ,

    #[mora_mappings("ch", "e")]
    チェ,

    #[mora_mappings("ch", "i")]
    チ,

    #[mora_mappings("d", "a")]
    ダ,

    #[mora_mappings("t", "a")]
    タ,

    #[mora_mappings("z", "o")]
    ゾ,

    #[mora_mappings("s", "o")]
    ソ,

    #[mora_mappings("z", "e")]
    ゼ,

    #[mora_mappings("s", "e")]
    セ,

    #[mora_mappings("z", "i")]
    ズィ,

    #[mora_mappings("z", "u")]
    ズ,

    #[mora_mappings("s", "i")]
    スィ,

    #[mora_mappings("s", "u")]
    ス,

    #[mora_mappings("j", "o")]
    ジョ,

    #[mora_mappings("j", "u")]
    ジュ,

    #[mora_mappings("j", "a")]
    ジャ,

    #[mora_mappings("j", "e")]
    ジェ,

    #[mora_mappings("j", "i")]
    ジ,

    #[mora_mappings("sh", "o")]
    ショ,

    #[mora_mappings("sh", "u")]
    シュ,

    #[mora_mappings("sh", "a")]
    シャ,

    #[mora_mappings("sh", "e")]
    シェ,

    #[mora_mappings("sh", "i")]
    シ,

    #[mora_mappings("z", "a")]
    ザ,

    #[mora_mappings("s", "a")]
    サ,

    #[mora_mappings("g", "o")]
    ゴ,

    #[mora_mappings("k", "o")]
    コ,

    #[mora_mappings("g", "e")]
    ゲ,

    #[mora_mappings("k", "e")]
    ケ,

    #[mora_mappings("gw", "a")]
    グヮ,

    #[mora_mappings("g", "u")]
    グ,

    #[mora_mappings("kw", "a")]
    クヮ,

    #[mora_mappings("k", "u")]
    ク,

    #[mora_mappings("gy", "o")]
    ギョ,

    #[mora_mappings("gy", "u")]
    ギュ,

    #[mora_mappings("gy", "a")]
    ギャ,

    #[mora_mappings("gy", "e")]
    ギェ,

    #[mora_mappings("g", "i")]
    ギ,

    #[mora_mappings("ky", "o")]
    キョ,

    #[mora_mappings("ky", "u")]
    キュ,

    #[mora_mappings("ky", "a")]
    キャ,

    #[mora_mappings("ky", "e")]
    キェ,

    #[mora_mappings("k", "i")]
    キ,

    #[mora_mappings("g", "a")]
    ガ,

    #[mora_mappings("k", "a")]
    カ,

    #[mora_mappings("", "o")]
    オ,

    #[mora_mappings("", "e")]
    エ,

    #[mora_mappings("w", "o")]
    ウォ,

    #[mora_mappings("w", "e")]
    ウェ,

    #[mora_mappings("w", "i")]
    ウィ,

    #[mora_mappings("", "u")]
    ウ,

    #[mora_mappings("y", "e")]
    イェ,

    #[mora_mappings("", "i")]
    イ,

    #[mora_mappings("", "a")]
    ア,
}

const _: () = assert!(MoraKana::COUNT == 144);

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use strum::EnumCount as _;

    use super::{MoraKana, MORA_KANA_TO_MORA_PHONEMES, MORA_PHONEMES_TO_MORA_KANA};

    #[test]
    fn mora_phonemes_to_mora_kana_has_the_same_length_as_mora_kana() {
        assert_eq!(MoraKana::COUNT, MORA_PHONEMES_TO_MORA_KANA.len());
    }

    #[test]
    fn mappings_work() {
        for (mora_kana, &(consonant, vowel)) in &MORA_KANA_TO_MORA_PHONEMES {
            let consonant = <&str>::from(consonant);
            let vowel = <&str>::from(vowel);
            assert_eq!(
                mora_kana,
                MORA_PHONEMES_TO_MORA_KANA[&format!("{consonant}{vowel}")]
            );
        }
    }
}
