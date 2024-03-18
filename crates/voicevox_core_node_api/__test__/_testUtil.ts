import path from "path";
import { AccentPhrase, Mora } from "voicevox_core";

export const openJtalkDicDir = path.join(
    __dirname,
    "..",
    "..",
    "test_util",
    "data",
    "open_jtalk_dic_utf_8-1.11",
);
export const modelFile = path.join(
    __dirname,
    "..",
    "..",
    "..",
    "model",
    "sample.vvm",
);

export function checkAllMoras(
    accentPhrases: AccentPhrase[],
    otherAccentPhrases: AccentPhrase[],
    checker: (mora: Mora, otherMora: Mora) => boolean,
): boolean {
    for (let i = 0; i < accentPhrases.length; i++) {
        const accentPhrase = accentPhrases[i];
        for (let j = 0; j < accentPhrase.moras.length; j++) {
            const mora = accentPhrase.moras[j];
            const otherMora = otherAccentPhrases[i].moras[j];
            if (!checker(mora, otherMora)) {
                return false;
            }
        }
    }
    return true;
}
