package jp.hiroshiba.voicevoxcore;

import com.google.gson.Gson;
import jakarta.annotation.Nonnull;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InferenceFailedException;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidModelDataException;

/**
 * 音声シンセサイザ。
 *
 * @see Synthesizer#builder
 */
public class Synthesizer extends Dll {
  private long handle;

  private Synthesizer(OpenJtalk openJtalk, Builder builder) {
    rsNewWithInitialize(openJtalk, builder);
  }

  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  /**
   * モデルを読み込む。
   *
   * @param voiceModel 読み込むモデル。
   */
  public void loadVoiceModel(VoiceModel voiceModel) throws InvalidModelDataException {
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
   * AquesTalk風記法から {@link AudioQuery} を生成する。
   *
   * @param kana AquesTalk風記法。
   * @param styleId スタイルID。
   * @return {@link AudioQuery}。
   */
  @Nonnull
  public AudioQuery createAudioQueryFromKana(String kana, int styleId)
      throws InferenceFailedException {
    if (!Utils.isU32(styleId)) {
      throw new IllegalArgumentException("styleId");
    }
    String queryJson = rsAudioQueryFromKana(kana, styleId);
    Gson gson = new Gson();

    AudioQuery audioQuery = gson.fromJson(queryJson, AudioQuery.class);
    if (audioQuery == null) {
      throw new NullPointerException("audio_query");
    }
    return audioQuery;
  }

  /**
   * 日本語のテキストから {@link AudioQuery} を生成する。
   *
   * @param text 日本語のテキスト。
   * @param styleId スタイルID。
   * @return {@link AudioQuery}。
   */
  @Nonnull
  public AudioQuery createAudioQuery(String text, int styleId) throws InferenceFailedException {
    if (!Utils.isU32(styleId)) {
      throw new IllegalArgumentException("styleId");
    }
    String queryJson = rsAudioQuery(text, styleId);
    Gson gson = new Gson();

    AudioQuery audioQuery = gson.fromJson(queryJson, AudioQuery.class);
    if (audioQuery == null) {
      throw new NullPointerException("audio_query");
    }
    return audioQuery;
  }

  /**
   * AquesTalk風記法から {@link AccentPhrase} のリストを生成する。
   *
   * @param kana AquesTalk風記法。
   * @param styleId スタイルID。
   * @return {@link AccentPhrase} のリスト。
   */
  @Nonnull
  public List<AccentPhrase> createAccentPhrasesFromKana(String kana, int styleId)
      throws InferenceFailedException {
    String accentPhrasesJson = rsAccentPhrasesFromKana(kana, styleId);
    Gson gson = new Gson();
    AccentPhrase[] rawAccentPhrases = gson.fromJson(accentPhrasesJson, AccentPhrase[].class);
    if (rawAccentPhrases == null) {
      throw new NullPointerException("accent_phrases");
    }
    return new ArrayList<AccentPhrase>(Arrays.asList(rawAccentPhrases));
  }

  /**
   * 日本語のテキストから {@link AccentPhrase} のリストを生成する。
   *
   * @param text 日本語のテキスト。
   * @param styleId スタイルID。
   * @return {@link AccentPhrase} のリスト。
   */
  @Nonnull
  public List<AccentPhrase> createAccentPhrases(String text, int styleId)
      throws InferenceFailedException {
    String accentPhrasesJson = rsAccentPhrases(text, styleId);
    Gson gson = new Gson();
    AccentPhrase[] rawAccentPhrases = gson.fromJson(accentPhrasesJson, AccentPhrase[].class);
    if (rawAccentPhrases == null) {
      throw new NullPointerException("accent_phrases");
    }
    return new ArrayList<AccentPhrase>(Arrays.asList(rawAccentPhrases));
  }

  /**
   * アクセント句の音高・音素長を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId スタイルID。
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replaceMoraData(List<AccentPhrase> accentPhrases, int styleId)
      throws InferenceFailedException {
    if (!Utils.isU32(styleId)) {
      throw new IllegalArgumentException("styleId");
    }
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplaceMoraData(accentPhrasesJson, styleId, false);
    return new ArrayList<>(
        Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  /**
   * アクセント句の音素長を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId スタイルID。
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replacePhonemeLength(List<AccentPhrase> accentPhrases, int styleId)
      throws InferenceFailedException {
    if (!Utils.isU32(styleId)) {
      throw new IllegalArgumentException("styleId");
    }
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplacePhonemeLength(accentPhrasesJson, styleId, false);
    return new ArrayList<>(
        Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  /**
   * アクセント句の音高を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId スタイルID。
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replaceMoraPitch(List<AccentPhrase> accentPhrases, int styleId)
      throws InferenceFailedException {
    if (!Utils.isU32(styleId)) {
      throw new IllegalArgumentException("styleId");
    }
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplaceMoraPitch(accentPhrasesJson, styleId, false);
    return new ArrayList<>(
        Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  /**
   * {@link AudioQuery} から音声合成するためのオブジェクトを生成する。
   *
   * @param audioQuery {@link AudioQuery}。
   * @param styleId スタイルID。
   * @return {@link SynthesisConfigurator}。
   * @see SynthesisConfigurator#execute
   */
  @Nonnull
  public SynthesisConfigurator synthesis(AudioQuery audioQuery, int styleId) {
    return new SynthesisConfigurator(this, audioQuery, styleId);
  }

  /**
   * AquesTalk風記法をもとに音声合成を実行するためのオブジェクトを生成する。
   *
   * @param kana AquesTalk風記法。
   * @param styleId スタイルID。
   * @return {@link TtsFromKanaConfigurator}。
   * @see TtsFromKanaConfigurator#execute
   */
  @Nonnull
  public TtsFromKanaConfigurator ttsFromKana(String kana, int styleId) {
    return new TtsFromKanaConfigurator(this, kana, styleId);
  }

  /**
   * 日本語のテキストをもとに音声合成を実行するためのオブジェクトを生成する。
   *
   * @param text 日本語のテキスト。
   * @param styleId スタイルID。
   * @return {@link TtsConfigurator}。
   * @see TtsConfigurator#execute
   */
  @Nonnull
  public TtsConfigurator tts(String text, int styleId) {
    return new TtsConfigurator(this, text, styleId);
  }

  private native void rsNewWithInitialize(OpenJtalk openJtalk, Builder builder);

  private native void rsLoadVoiceModel(VoiceModel voiceModel) throws InvalidModelDataException;

  private native void rsUnloadVoiceModel(String voiceModelId);

  private native boolean rsIsLoadedVoiceModel(String voiceModelId);

  @Nonnull
  private native String rsAudioQueryFromKana(String kana, int styleId)
      throws InferenceFailedException;

  @Nonnull
  private native String rsAudioQuery(String text, int styleId) throws InferenceFailedException;

  @Nonnull
  private native String rsAccentPhrasesFromKana(String kana, int styleId)
      throws InferenceFailedException;

  @Nonnull
  private native String rsAccentPhrases(String text, int styleId) throws InferenceFailedException;

  @Nonnull
  private native String rsReplaceMoraData(String accentPhrasesJson, int styleId, boolean kana)
      throws InferenceFailedException;

  @Nonnull
  private native String rsReplacePhonemeLength(String accentPhrasesJson, int styleId, boolean kana)
      throws InferenceFailedException;

  @Nonnull
  private native String rsReplaceMoraPitch(String accentPhrasesJson, int styleId, boolean kana)
      throws InferenceFailedException;

  @Nonnull
  private native byte[] rsSynthesis(
      String queryJson, int styleId, boolean enableInterrogativeUpspeak)
      throws InferenceFailedException;

  @Nonnull
  private native byte[] rsTtsFromKana(String kana, int styleId, boolean enableInterrogativeUpspeak)
      throws InferenceFailedException;

  @Nonnull
  private native byte[] rsTts(String text, int styleId, boolean enableInterrogativeUpspeak)
      throws InferenceFailedException;

  private native void rsDrop();

  public static Builder builder(OpenJtalk openJtalk) {
    return new Builder(openJtalk);
  }

  /**
   * 音声シンセサイザのビルダー。
   *
   * @see Synthesizer#builder
   */
  public static class Builder {
    private OpenJtalk openJtalk;

    @SuppressWarnings("unused")
    private AccelerationMode accelerationMode;

    @SuppressWarnings("unused")
    private int cpuNumThreads;

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
      if (Utils.isU16(cpuNumThreads)) {
        throw new IllegalArgumentException("cpuNumThreads");
      }
      this.cpuNumThreads = cpuNumThreads;
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

  /** {@link Synthesizer#synthesis} のオプション。 */
  public class SynthesisConfigurator {
    private Synthesizer synthesizer;
    private AudioQuery audioQuery;
    private int styleId;
    private boolean interrogativeUpspeak;

    private SynthesisConfigurator(Synthesizer synthesizer, AudioQuery audioQuery, int styleId) {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      this.synthesizer = synthesizer;
      this.audioQuery = audioQuery;
      this.styleId = styleId;
      this.interrogativeUpspeak = false;
    }

    /**
     * 疑問文の調整を有効にするかどうか。
     *
     * @param interrogativeUpspeak 疑問文の調整を有効にするかどうか。
     * @return {@link SynthesisConfigurator}。
     */
    @Nonnull
    public SynthesisConfigurator interrogativeUpspeak(boolean interrogativeUpspeak) {
      this.interrogativeUpspeak = interrogativeUpspeak;
      return this;
    }

    /**
     * {@link AudioQuery} から音声合成する。
     *
     * @return 音声データ。
     */
    @Nonnull
    public byte[] execute() throws InferenceFailedException {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      Gson gson = new Gson();
      String queryJson = gson.toJson(this.audioQuery);
      return synthesizer.rsSynthesis(queryJson, this.styleId, this.interrogativeUpspeak);
    }
  }

  /** {@link Synthesizer#ttsFromKana} のオプション。 */
  public class TtsFromKanaConfigurator {
    private Synthesizer synthesizer;
    private String kana;
    private int styleId;
    private boolean interrogativeUpspeak;

    private TtsFromKanaConfigurator(Synthesizer synthesizer, String kana, int styleId) {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      this.synthesizer = synthesizer;
      this.kana = kana;
      this.styleId = styleId;
    }

    /**
     * 疑問文の調整を有効にするかどうか。
     *
     * @param interrogativeUpspeak 疑問文の調整を有効にするかどうか。
     * @return {@link SynthesisConfigurator}。
     */
    @Nonnull
    public TtsFromKanaConfigurator interrogativeUpspeak(boolean interrogativeUpspeak) {
      this.interrogativeUpspeak = interrogativeUpspeak;
      return this;
    }

    /**
     * {@link AudioQuery} から音声合成する。
     *
     * @return 音声データ。
     */
    @Nonnull
    public byte[] execute() throws InferenceFailedException {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      return synthesizer.rsTtsFromKana(this.kana, this.styleId, this.interrogativeUpspeak);
    }
  }

  /** {@link Synthesizer#tts} のオプション。 */
  public class TtsConfigurator {
    private Synthesizer synthesizer;
    private String text;
    private int styleId;
    private boolean interrogativeUpspeak;

    private TtsConfigurator(Synthesizer synthesizer, String text, int styleId) {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      this.synthesizer = synthesizer;
      this.text = text;
      this.styleId = styleId;
    }

    /**
     * 疑問文の調整を有効にするかどうか。
     *
     * @param interrogativeUpspeak 疑問文の調整を有効にするかどうか。
     * @return {@link SynthesisConfigurator}。
     */
    @Nonnull
    public TtsConfigurator interrogativeUpspeak(boolean interrogativeUpspeak) {
      this.interrogativeUpspeak = interrogativeUpspeak;
      return this;
    }

    /**
     * {@link AudioQuery} から音声合成する。
     *
     * @return 音声データ。
     */
    @Nonnull
    public byte[] execute() throws InferenceFailedException {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      return synthesizer.rsTts(this.text, this.styleId, this.interrogativeUpspeak);
    }
  }
}
