import test from "ava";
import { modelFile, openJtalkDicDir, checkAllMoras } from "../_testUtil";
import { promises } from "../..";

async function loadOpenJtalk() {
    return await promises.OpenJtalk.create(openJtalkDicDir);
}

async function loadModel() {
    return await promises.VoiceModel.fromPath(modelFile);
}

test("ハードウエアアクセラレーションモードが指定したとおりに設定されること", async (t) => {
    const openJtalk = await loadOpenJtalk();
    const synthesizer = new promises.Synthesizer(openJtalk, {
        accelerationMode: "CPU",
    });
    t.false(synthesizer.isGpuMode);
});

test("VoiceModelのロード・アンロード時にモデル数を正しく取得できること", async (t) => {
    const model = await loadModel();
    t.true(model.metas.length >= 1);

    const openJtalk = await loadOpenJtalk();
    const synthesizer = new promises.Synthesizer(openJtalk);

    t.is(synthesizer.metas.length, 0);

    await synthesizer.loadVoiceModel(model);
    t.is(synthesizer.metas.length, model.metas.length);

    synthesizer.unloadVoiceModel(model.id);

    t.is(synthesizer.metas.length, 0);
    t.false(synthesizer.isLoadedVoiceModel(model.id));
});

test("AudioQueryからの合成でエラーが発生しないこと", async (t) => {
    const model = await loadModel();
    const openJtalk = await loadOpenJtalk();
    const synthesizer = new promises.Synthesizer(openJtalk);
    await synthesizer.loadVoiceModel(model);
    const query = await synthesizer.audioQuery(
        "こんにちは",
        model.metas[0].styles[0].id,
    );
    await t.notThrowsAsync(() =>
        synthesizer.synthesis(query, model.metas[0].styles[0].id),
    );
});

test("パラメータを変えてAccentPhraseを生成し直すとモーラの値が変わること", async (t) => {
    const model = await loadModel();
    const openJtalk = await loadOpenJtalk();
    const synthesizer = new promises.Synthesizer(openJtalk);
    await synthesizer.loadVoiceModel(model);
    const accentPhrases = await synthesizer.createAccentPhrases(
        "こんにちは",
        model.metas[0].styles[0].id,
    );
    const accentPhrases2 = await synthesizer.replaceMoraPitch(
        accentPhrases,
        model.metas[1].styles[0].id,
    );
    t.true(
        checkAllMoras(
            accentPhrases,
            accentPhrases2,
            (mora, otherMora) => mora.pitch != otherMora.pitch,
        ),
    );
    const accentPhrases3 = await synthesizer.replacePhonemeLength(
        accentPhrases,
        model.metas[1].styles[0].id,
    );
    t.true(
        checkAllMoras(
            accentPhrases,
            accentPhrases3,
            (mora, otherMora) => mora.vowel_length != otherMora.vowel_length,
        ),
    );
    const accentPhrases4 = await synthesizer.replaceMoraData(
        accentPhrases,
        model.metas[1].styles[0].id,
    );
    t.true(
        checkAllMoras(
            accentPhrases,
            accentPhrases4,
            (mora, otherMora) =>
                mora.pitch != otherMora.pitch &&
                mora.vowel_length != otherMora.vowel_length,
        ),
    );
});

test("日本語のテキストからの音声合成でエラーが発生しないこと", async (t) => {
    const model = await loadModel();
    const openJtalk = await loadOpenJtalk();
    const synthesizer = new promises.Synthesizer(openJtalk);
    await synthesizer.loadVoiceModel(model);
    t.notThrows(() =>
        synthesizer.tts("こんにちは", model.metas[0].styles[0].id),
    );
});
