import test from 'ava';
import { modelFile, openJtalkDicDir } from '../_testUtil';
import { blocking } from '../..';

function loadOpenJtalk() {
    return blocking.OpenJtalk.create(openJtalkDicDir);
}

function loadModel() {
    return blocking.VoiceModel.fromPath(modelFile);
}

test('ハードウエアアクセラレーションモードが指定したとおりに設定されること', (t) => {
    const openJtalk = loadOpenJtalk();
    const synthesizer = new blocking.Synthesizer(openJtalk, {
        accelerationMode: 'CPU'
    });
    t.false(synthesizer.isGpuMode);
})

test('VoiceModelのロード・アンロード時にモデル数を正しく取得できること', (t) => {
    const model = loadModel();
    t.true(model.metas.length >= 1);

    const openJtalk = loadOpenJtalk();
    const synthesizer = new blocking.Synthesizer(openJtalk);

    t.is(synthesizer.metas.length, 0);

    synthesizer.loadVoiceModel(model);
    t.is(synthesizer.metas.length, model.metas.length);

    synthesizer.unloadVoiceModel(model.id);

    t.is(synthesizer.metas.length, 0)
    t.false(synthesizer.isLoadedVoiceModel(model.id));
})
