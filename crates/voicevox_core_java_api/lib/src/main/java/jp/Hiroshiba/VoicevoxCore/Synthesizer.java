package jp.Hiroshiba.VoicevoxCore;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.EnumSet;
import java.util.List;

import javax.annotation.Nonnull;

import com.google.gson.Gson;

/**
 * 音声シンセサイザ。
 *
 * @see Synthesizer#builder
 */
public class Synthesizer extends Dll implements AutoCloseable {
  long internal;

  private Synthesizer(OpenJtalk openJtalk, Builder builder) {
    rsNewWithInitialize(openJtalk, builder);
  }

  /**
   * モデルを読み込む。
   *
   * @param voiceModel 読み込むモデル。
   */
  public void loadVoiceModel(VoiceModel voiceModel) {
    rsLoadVoiceModel(voiceModel);
  }

  /**
   * 音声モデルの読み込みを解除する。
   *
   * @param voiceModelId 読み込みを解除する音声モデルのID。
   */
  public void unloadVoiceModel(String voiceModelId) {
    rsUnloadVoiceModel(voiceModelId);
  }

  /**
   * 指定した音声モデルのIDが読み込まれているかどうかを返す。
   *
   * @param voiceModelId 音声モデルのID。
   * @return 指定した音声モデルのIDが読み込まれているかどうか。
   */
  public boolean isLoadedVoiceModel(String voiceModelId) {
    return rsIsLoadedVoiceModel(voiceModelId);
  }

  /**
   * {@link AudioQuery} を生成する。
   *
   * @param text    テキスト。
   * @param styleId スタイルID。
   * @param options {@link AudioQueryOption} のセット。
   *
   * @return 話者とテキストから生成された {@link AudioQuery}。
   */
  @Nonnull
  public AudioQuery audioQuery(String text, int styleId, EnumSet<AudioQueryOption> options) {
    boolean kana = options.contains(AudioQueryOption.KANA);
    String queryJson = rsAudioQuery(text, styleId, kana);
    Gson gson = new Gson();

    AudioQuery audioQuery = gson.fromJson(queryJson, AudioQuery.class);
    if (audioQuery == null) {
      throw new NullPointerException("audio_query");
    }
    return audioQuery;
  }

  /**
   * {@link AccentPhrase} の配列を生成する。
   *
   * @param text    テキスト。
   * @param styleId スタイルID。
   * @param options {@link AudioQueryOption} のセット。
   *
   * @return 話者とテキストから生成された {@link AccentPhrase} の配列。
   */
  @Nonnull
  public List<AccentPhrase> createAccentPhrases(String text, int styleId, EnumSet<AccentPhrasesOption> options) {
    boolean kana = options.contains(AccentPhrasesOption.KANA);
    String accentPhrasesJson = rsAccentPhrases(text, styleId, kana);
    Gson gson = new Gson();
    AccentPhrase[] rawAccentPhrases = gson.fromJson(accentPhrasesJson, AccentPhrase[].class);
    if (rawAccentPhrases == null) {
      throw new NullPointerException("accent_phrases");
    }
    return new ArrayList<>(Arrays.asList(rawAccentPhrases));
  }

  /**
   * アクセント句の音高・音素長を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId       スタイルID。
   *
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replaceMoraData(List<AccentPhrase> accentPhrases, int styleId) {
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplaceMoraData(accentPhrasesJson, styleId, false);
    return new ArrayList<>(Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  /**
   * アクセント句の音素長を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId       スタイルID。
   *
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replacePhonemeLength(List<AccentPhrase> accentPhrases, int styleId) {
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplacePhonemeLength(accentPhrasesJson, styleId, false);
    return new ArrayList<>(Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  /**
   * アクセント句の音高を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId       スタイルID。
   *
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replaceMoraPitch(List<AccentPhrase> accentPhrases, int styleId) {
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplaceMoraPitch(accentPhrasesJson, styleId, false);
    return new ArrayList<>(Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  /**
   * {@link AudioQuery} から音声合成する。
   *
   * @param audioQuery {@link AudioQuery}。
   * @param styleId    スタイルID。
   * @param options    {@link SynthesisOption} のセット。
   *
   * @return WAVデータ。
   */
  @Nonnull
  public byte[] synthesis(AudioQuery audioQuery, int styleId, EnumSet<SynthesisOption> options) {
    boolean enableInterrogativeUpspeak = options.contains(SynthesisOption.ENABLE_INTERROGATIVE_UPSPEAK);
    Gson gson = new Gson();
    String queryJson = gson.toJson(audioQuery);
    return rsSynthesis(queryJson, styleId, enableInterrogativeUpspeak);
  }

  /**
   * テキスト音声合成を実行する。
   *
   * @param text    テキスト。
   * @param styleId スタイルID。
   * @param options {@link TtsOption} のセット。
   *
   * @return WAVデータ。
   */
  @Nonnull
  public byte[] tts(String text, int styleId, EnumSet<TtsOption> options) {
    boolean kana = options.contains(TtsOption.KANA);
    boolean enableInterrogativeUpspeak = options.contains(TtsOption.ENABLE_INTERROGATIVE_UPSPEAK);
    return rsTts(text, styleId, kana, enableInterrogativeUpspeak);
  }

  /**
   * 音声シンセサイザを破棄する。
   */
  public void close() {
    rsDrop();
  }

  private native void rsNewWithInitialize(OpenJtalk openJtalk, Builder builder);

  private native void rsLoadVoiceModel(VoiceModel voiceModel);

  private native void rsUnloadVoiceModel(String voiceModelId);

  private native boolean rsIsLoadedVoiceModel(String voiceModelId);

  @Nonnull
  private native String rsAudioQuery(String text, int styleId, boolean kana);

  @Nonnull
  private native String rsAccentPhrases(String text, int styleId, boolean kana);

  @Nonnull
  private native String rsReplaceMoraData(String accentPhrasesJson, int styleId, boolean kana);

  @Nonnull
  private native String rsReplacePhonemeLength(String accentPhrasesJson, int styleId, boolean kana);

  @Nonnull
  private native String rsReplaceMoraPitch(String accentPhrasesJson, int styleId, boolean kana);

  @Nonnull
  private native byte[] rsSynthesis(String queryJson, int styleId, boolean enableInterrogativeUpspeak);

  @Nonnull
  private native byte[] rsTts(String text, int styleId, boolean kana, boolean enableInterrogativeUpspeak);

  private native void rsDrop();

  public static Builder builder(OpenJtalk openJtalk) {
    return new Builder(openJtalk);
  }

  /**
   * 音声シンセサイザのビルダー。
   * @see Synthesizer#builder
   */
  public static class Builder {
    private OpenJtalk openJtalk;
    @SuppressWarnings("unused")
    private AccelerationMode accelerationMode;
    @SuppressWarnings("unused")
    private int cpuNumThreads;
    @SuppressWarnings("unused")
    private boolean loadAllModels;

    public Builder(OpenJtalk openJtalk) {
      this.openJtalk = openJtalk;
    }

    /**
     * ハードウェアアクセラレーションモードを設定する。
     *
     * @param accelerationMode ハードウェアアクセラレーションモード。
     * @return ビルダー。
     */
    public Builder accelerationMode(AccelerationMode accelerationMode) {
      this.accelerationMode = accelerationMode;
      return this;
    }

    /**
     * CPU利用数を指定する。0を指定すると環境に合わせたCPUが利用される。
     *
     * @param cpuNumThreads CPU利用数。
     * @return ビルダー。
     */
    public Builder cpuNumThreads(int cpuNumThreads) {
      this.cpuNumThreads = cpuNumThreads;
      return this;
    }

    /**
     * 全てのモデルを読み込むかどうか。
     *
     * @param loadAllModels 全てのモデルを読み込むかどうか。
     * @return ビルダー。
     */
    public Builder loadAllModels(boolean loadAllModels) {
      this.loadAllModels = loadAllModels;
      return this;
    }

    /**
     * {@link Synthesizer} を構築する。
     *
     * @return {@link Synthesizer}。
     */
    public Synthesizer build() {
      Synthesizer synthesizer = new Synthesizer(openJtalk, this);
      return synthesizer;
    }
  }

  /** ハードウェアアクセラレーションモード。 */
  public static enum AccelerationMode {
    /** 実行環境に合わせて自動的に選択する。 */
    AUTO,
    /** CPUに設定する。 */
    CPU,
    /** GPUに設定する。 */
    GPU,
  }

  /** {@link Synthesizer#audioQuery} のオプション。 */
  public static enum AudioQueryOption {
    /** 入力テキストをAquesTalk風記法として解釈するかどうか。 */
    KANA,
  }

  /** {@link Synthesizer#createAccentPhrases} のオプション。 */
  public static enum AccentPhrasesOption {
    /** 入力テキストをAquesTalk風記法として解釈するかどうか。 */
    KANA,
  }

  /** {@link Synthesizer#synthesis} のオプション。 */
  public static enum SynthesisOption {
    /** 疑問文の調整を有効にするかどうか。 */
    ENABLE_INTERROGATIVE_UPSPEAK
  }

  /** {@link Synthesizer#tts} のオプション。 */
  public static enum TtsOption {
    /** 入力テキストをAquesTalk風記法として解釈するかどうか。 */
    KANA,
    /** 疑問文の調整を有効にするかどうか。 */
    ENABLE_INTERROGATIVE_UPSPEAK
  }
}
