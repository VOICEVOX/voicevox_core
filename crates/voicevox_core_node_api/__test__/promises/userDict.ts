import fs from "fs/promises";
import os from "os";
import test from "ava";
import {
    OpenJtalk,
    UserDict,
    Synthesizer,
    VoiceModel,
} from "voicevox_core/promises";
import { modelFile, openJtalkDicDir } from "../_testUtil";
import path from "path";

test("単語を追加した辞書をロードするとAudioQueryのkanaが変化すること", async (t) => {
    const openJtalk = await OpenJtalk.create(openJtalkDicDir);
    const model = await VoiceModel.fromPath(modelFile);
    const synthesizer = new Synthesizer(openJtalk);
    synthesizer.loadVoiceModel(model);
    const query1 = synthesizer.audioQuery(
        "this_word_should_not_exist_in_default_dictionary",
        model.metas[0].styles[0].id,
    );
    const userDict = new UserDict();
    userDict.addWord({
        surface: "this_word_should_not_exist_in_default_dictionary",
        pronunciation: "テスト",
    });
    openJtalk.useUserDict(userDict);
    const query2 = synthesizer.audioQuery(
        "this_word_should_not_exist_in_default_dictionary",
        model.metas[0].styles[0].id,
    );
    t.not(query1, query2);
});

test("ユーザー辞書の操作を正常に行えること", async (t) => {
    const dictA = new UserDict();

    // 単語の追加
    const uuidA = dictA.addWord({
        surface: "hoge",
        pronunciation: "ホゲ",
    });
    t.is(dictA.words[uuidA].surface, "ｈｏｇｅ");
    t.is(dictA.words[uuidA].pronunciation, "ホゲ");

    // 単語の更新
    dictA.updateWord(uuidA, {
        surface: "fuga",
        pronunciation: "フガ",
    });
    t.is(dictA.words[uuidA].surface, "ｆｕｇａ");
    t.is(dictA.words[uuidA].pronunciation, "フガ");

    // ユーザー辞書のインポート
    const dictB = new UserDict();
    const uuidB = dictB.addWord({
        surface: "foo",
        pronunciation: "フー",
    });
    dictA.importDict(dictB);
    t.true(uuidB in dictA.words);

    /// ユーザー辞書のエクスポート
    const dictC = new UserDict();
    const uuidC = dictC.addWord({
        surface: "bar",
        pronunciation: "バー",
    });
    const tempPath = path.join(
        await fs.mkdtemp(path.join(os.tmpdir(), "voicevox_core-")),
        "test_user_dict",
    );
    await dictC.save(tempPath);
    await dictA.load(tempPath);
    t.true(uuidA in dictA.words);
    t.true(uuidC in dictA.words);

    /// 単語の削除
    dictA.removeWord(uuidA);
    t.false(uuidA in dictA.words);
    t.true(uuidC in dictA.words);

    /// 単語のバリデーション
    t.throws(() =>
        dictA.addWord({
            surface: "",
            pronunciation: "カタカナ以外の文字",
        }),
    );
});
