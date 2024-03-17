import test from 'ava';
import { blocking } from '../..';
import { modelFile, openJtalkDicDir } from '../testUtil';

test('単語を追加した辞書をロードするとAudioQueryのkanaが変化すること', (t) => {
    const openJtalk = blocking.OpenJtalk.create(openJtalkDicDir);
    const model = blocking.VoiceModel.fromPath(modelFile);
    const synthesizer = new blocking.Synthesizer(openJtalk);
    synthesizer.loadVoiceModel(model);
    const query1 = synthesizer.audioQuery('this_word_should_not_exist_in_default_dictionary', model.metas[0].styles[0].id);
    const userDict = new blocking.UserDict();
    userDict.addWord({
        surface: 'this_word_should_not_exist_in_default_dictionary',
        pronunciation: 'テスト',
        accentType: 1,
        wordType: 'COMMON_NOUN',
        priority: 1
    });
    openJtalk.useUserDict(userDict);
    const query2 = synthesizer.audioQuery('this_word_should_not_exist_in_default_dictionary', model.metas[0].styles[0].id);
    t.not(query1, query2);
});
