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

// TODO: マクロにより次の二つの、コンパイル時最適化されるマップを生成する。
//
// - `enum_map::EnumMap<MoraKana, (OptionalConsonant, MoraTail)>`
//   (text → (consonant?, vowel))（マップとしてはtotal）
//   （kana_parser用）
// - `pdf::Map<&'static str, MoraKana>`
//   (consonant? ++ vowel → text)（マップとしてはpartial）
//   （full_context_label用）

pub(super) enum MoraKana {
    #[mora_kana("v", "o")]
    ヴォ,

    #[mora_kana("v", "e")]
    ヴェ,

    #[mora_kana("v", "i")]
    ヴィ,

    #[mora_kana("v", "a")]
    ヴァ,

    #[mora_kana("v", "u")]
    ヴ,

    #[mora_kana("", "N")]
    ン,

    #[mora_kana("w", "a")]
    ワ,

    #[mora_kana("r", "o")]
    ロ,

    #[mora_kana("r", "e")]
    レ,

    #[mora_kana("r", "u")]
    ル,

    #[mora_kana("ry", "o")]
    リョ,

    #[mora_kana("ry", "u")]
    リュ,

    #[mora_kana("ry", "a")]
    リャ,

    #[mora_kana("ry", "e")]
    リェ,

    #[mora_kana("r", "i")]
    リ,

    #[mora_kana("r", "a")]
    ラ,

    #[mora_kana("y", "o")]
    ヨ,

    #[mora_kana("y", "u")]
    ユ,

    #[mora_kana("y", "a")]
    ヤ,

    #[mora_kana("m", "o")]
    モ,

    #[mora_kana("m", "e")]
    メ,

    #[mora_kana("m", "u")]
    ム,

    #[mora_kana("my", "o")]
    ミョ,

    #[mora_kana("my", "u")]
    ミュ,

    #[mora_kana("my", "a")]
    ミャ,

    #[mora_kana("my", "e")]
    ミェ,

    #[mora_kana("m", "i")]
    ミ,

    #[mora_kana("m", "a")]
    マ,

    #[mora_kana("p", "o")]
    ポ,

    #[mora_kana("b", "o")]
    ボ,

    #[mora_kana("h", "o")]
    ホ,

    #[mora_kana("p", "e")]
    ペ,

    #[mora_kana("b", "e")]
    ベ,

    #[mora_kana("h", "e")]
    ヘ,

    #[mora_kana("p", "u")]
    プ,

    #[mora_kana("b", "u")]
    ブ,

    #[mora_kana("f", "o")]
    フォ,

    #[mora_kana("f", "e")]
    フェ,

    #[mora_kana("f", "i")]
    フィ,

    #[mora_kana("f", "a")]
    ファ,

    #[mora_kana("f", "u")]
    フ,

    #[mora_kana("py", "o")]
    ピョ,

    #[mora_kana("py", "u")]
    ピュ,

    #[mora_kana("py", "a")]
    ピャ,

    #[mora_kana("py", "e")]
    ピェ,

    #[mora_kana("p", "i")]
    ピ,

    #[mora_kana("by", "o")]
    ビョ,

    #[mora_kana("by", "u")]
    ビュ,

    #[mora_kana("by", "a")]
    ビャ,

    #[mora_kana("by", "e")]
    ビェ,

    #[mora_kana("b", "i")]
    ビ,

    #[mora_kana("hy", "o")]
    ヒョ,

    #[mora_kana("hy", "u")]
    ヒュ,

    #[mora_kana("hy", "a")]
    ヒャ,

    #[mora_kana("hy", "e")]
    ヒェ,

    #[mora_kana("h", "i")]
    ヒ,

    #[mora_kana("p", "a")]
    パ,

    #[mora_kana("b", "a")]
    バ,

    #[mora_kana("h", "a")]
    ハ,

    #[mora_kana("n", "o")]
    ノ,

    #[mora_kana("n", "e")]
    ネ,

    #[mora_kana("n", "u")]
    ヌ,

    #[mora_kana("ny", "o")]
    ニョ,

    #[mora_kana("ny", "u")]
    ニュ,

    #[mora_kana("ny", "a")]
    ニャ,

    #[mora_kana("ny", "e")]
    ニェ,

    #[mora_kana("n", "i")]
    ニ,

    #[mora_kana("n", "a")]
    ナ,

    #[mora_kana("d", "u")]
    ドゥ,

    #[mora_kana("d", "o")]
    ド,

    #[mora_kana("t", "u")]
    トゥ,

    #[mora_kana("t", "o")]
    ト,

    #[mora_kana("dy", "o")]
    デョ,

    #[mora_kana("dy", "u")]
    デュ,

    #[mora_kana("dy", "a")]
    デャ,

    #[mora_kana("d", "i")]
    ディ,

    #[mora_kana("d", "e")]
    デ,

    #[mora_kana("ty", "o")]
    テョ,

    #[mora_kana("ty", "u")]
    テュ,

    #[mora_kana("ty", "a")]
    テャ,

    #[mora_kana("t", "i")]
    ティ,

    #[mora_kana("t", "e")]
    テ,

    #[mora_kana("ts", "o")]
    ツォ,

    #[mora_kana("ts", "e")]
    ツェ,

    #[mora_kana("ts", "i")]
    ツィ,

    #[mora_kana("ts", "a")]
    ツァ,

    #[mora_kana("ts", "u")]
    ツ,

    #[mora_kana("", "cl")]
    ッ,

    #[mora_kana("ch", "o")]
    チョ,

    #[mora_kana("ch", "u")]
    チュ,

    #[mora_kana("ch", "a")]
    チャ,

    #[mora_kana("ch", "e")]
    チェ,

    #[mora_kana("ch", "i")]
    チ,

    #[mora_kana("d", "a")]
    ダ,

    #[mora_kana("t", "a")]
    タ,

    #[mora_kana("z", "o")]
    ゾ,

    #[mora_kana("s", "o")]
    ソ,

    #[mora_kana("z", "e")]
    ゼ,

    #[mora_kana("s", "e")]
    セ,

    #[mora_kana("z", "i")]
    ズィ,

    #[mora_kana("z", "u")]
    ズ,

    #[mora_kana("s", "i")]
    スィ,

    #[mora_kana("s", "u")]
    ス,

    #[mora_kana("j", "o")]
    ジョ,

    #[mora_kana("j", "u")]
    ジュ,

    #[mora_kana("j", "a")]
    ジャ,

    #[mora_kana("j", "e")]
    ジェ,

    #[mora_kana("j", "i")]
    ジ,

    #[mora_kana("sh", "o")]
    ショ,

    #[mora_kana("sh", "u")]
    シュ,

    #[mora_kana("sh", "a")]
    シャ,

    #[mora_kana("sh", "e")]
    シェ,

    #[mora_kana("sh", "i")]
    シ,

    #[mora_kana("z", "a")]
    ザ,

    #[mora_kana("s", "a")]
    サ,

    #[mora_kana("g", "o")]
    ゴ,

    #[mora_kana("k", "o")]
    コ,

    #[mora_kana("g", "e")]
    ゲ,

    #[mora_kana("k", "e")]
    ケ,

    #[mora_kana("gw", "a")]
    グヮ,

    #[mora_kana("g", "u")]
    グ,

    #[mora_kana("kw", "a")]
    クヮ,

    #[mora_kana("k", "u")]
    ク,

    #[mora_kana("gy", "o")]
    ギョ,

    #[mora_kana("gy", "u")]
    ギュ,

    #[mora_kana("gy", "a")]
    ギャ,

    #[mora_kana("gy", "e")]
    ギェ,

    #[mora_kana("g", "i")]
    ギ,

    #[mora_kana("ky", "o")]
    キョ,

    #[mora_kana("ky", "u")]
    キュ,

    #[mora_kana("ky", "a")]
    キャ,

    #[mora_kana("ky", "e")]
    キェ,

    #[mora_kana("k", "i")]
    キ,

    #[mora_kana("g", "a")]
    ガ,

    #[mora_kana("k", "a")]
    カ,

    #[mora_kana("", "o")]
    オ,

    #[mora_kana("", "e")]
    エ,

    #[mora_kana("w", "o")]
    ウォ,

    #[mora_kana("w", "e")]
    ウェ,

    #[mora_kana("w", "i")]
    ウィ,

    #[mora_kana("", "u")]
    ウ,

    #[mora_kana("y", "e")]
    イェ,

    #[mora_kana("", "i")]
    イ,

    #[mora_kana("", "a")]
    ア,
}
