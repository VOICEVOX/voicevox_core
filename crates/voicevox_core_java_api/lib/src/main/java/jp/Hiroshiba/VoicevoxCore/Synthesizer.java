package jp.Hiroshiba.VoicevoxCore;

import com.google.gson.Gson;
import java.lang.ref.Cleaner;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import javax.annotation.Nonnull;

/**
 * 音声シンセサイザ。
 *
 * @see Synthesizer#builder
 */
public class Synthesizer extends Dll {
  private long handle;
  private static final Cleaner cleaner = Cleaner.create();

  private Synthesizer(OpenJtalk openJtalk, Builder builder) {
    rsNewWithInitialize(openJtalk, builder);
    cleaner.register(this, () -> rsDrop());
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
   * {@link AudioQuery} を生成するためのオブジェクトを生成する。
   *
   * @param text テキスト。
   * @param styleId スタイルID。
   * @return {@link CreateAudioQueryConfigurator}。
   *
   * @see CreateAudioQueryConfigurator#execute
   */
  @Nonnull
  public CreateAudioQueryConfigurator createAudioQuery(String text, int styleId) {
    return new CreateAudioQueryConfigurator(this, text, styleId);
  }

  /**
   * {@link AccentPhrase} のリストを生成するためのオブジェクトを生成する。
   *
   * @param text テキスト。
   * @param styleId スタイルID。
   * @return {@link CreateAccentPhrasesConfigurator}。
   *
   * @see CreateAccentPhrasesConfigurator#execute
   */
  @Nonnull
  public CreateAccentPhrasesConfigurator createAccentPhrases(String text, int styleId) {
    return new CreateAccentPhrasesConfigurator(this, text, styleId);
  }

  /**
   * アクセント句の音高・音素長を変更する。
   *
   * @param accentPhrases 変更元のアクセント句の配列。
   * @param styleId スタイルID。
   * @return 変更後のアクセント句の配列。
   */
  @Nonnull
  public List<AccentPhrase> replaceMoraData(List<AccentPhrase> accentPhrases, int styleId) {
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
  public List<AccentPhrase> replacePhonemeLength(List<AccentPhrase> accentPhrases, int styleId) {
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
  public List<AccentPhrase> replaceMoraPitch(List<AccentPhrase> accentPhrases, int styleId) {
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
   *
   * @see SynthesisConfigurator#execute
   */
  @Nonnull
  public SynthesisConfigurator synthesis(AudioQuery audioQuery, int styleId) {
    return new SynthesisConfigurator(this, audioQuery, styleId);
  }

  /**
   * テキスト音声合成を実行するためのオブジェクトを生成する。
   *
   * @param text テキスト。
   * @param styleId スタイルID。
   * @return {@link TtsConfigurator}。
   *
   * @see TtsConfigurator#execute
   */
  @Nonnull
  public TtsConfigurator tts(String text, int styleId) {
    return new TtsConfigurator(this, text, styleId);
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
  private native byte[] rsSynthesis(
      String queryJson, int styleId, boolean enableInterrogativeUpspeak);

  @Nonnull
  private native byte[] rsTts(
      String text, int styleId, boolean kana, boolean enableInterrogativeUpspeak);

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
      if (Utils.isU16(cpuNumThreads)) {
        throw new IllegalArgumentException("cpuNumThreads");
      }
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

  /** {@link Synthesizer#createAudioQuery} のオプション。 */
  public class CreateAudioQueryConfigurator {
    private Synthesizer synthesizer;
    private String text;
    private int styleId;
    private boolean kana;

    private CreateAudioQueryConfigurator(Synthesizer synthesizer, String text, int styleId) {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      this.synthesizer = synthesizer;
      this.text = text;
      this.styleId = styleId;
      this.kana = false;
    }

    /**
     * 入力テキストをAquesTalk風記法として解釈するかどうか。
     *
     * @param kana 入力テキストをAquesTalk風記法として解釈するかどうか。
     * @return {@link CreateAudioQueryConfigurator}。
     */
    @Nonnull
    public CreateAudioQueryConfigurator kana(boolean kana) {
      this.kana = kana;
      return this;
    }

    /**
     * {@link AudioQuery} を生成する。
     *
     * @return {@link AudioQuery}。
     */
    @Nonnull
    public AudioQuery execute() {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      String queryJson = synthesizer.rsAudioQuery(this.text, this.styleId, this.kana);
      Gson gson = new Gson();

      AudioQuery audioQuery = gson.fromJson(queryJson, AudioQuery.class);
      if (audioQuery == null) {
        throw new NullPointerException("audio_query");
      }
      return audioQuery;
    }
  }

  /** {@link Synthesizer#createAccentPhrases} のオプション。 */
  public class CreateAccentPhrasesConfigurator {
    private Synthesizer synthesizer;
    private String text;
    private int styleId;
    private boolean kana;

    private CreateAccentPhrasesConfigurator(Synthesizer synthesizer, String text, int styleId) {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      this.synthesizer = synthesizer;
      this.text = text;
      this.styleId = styleId;
      this.kana = false;
    }

    /**
     * 入力テキストをAquesTalk風記法として解釈するかどうか。
     *
     * @param kana 入力テキストをAquesTalk風記法として解釈するかどうか。
     * @return {@link CreateAudioQueryConfigurator}。
     */
    @Nonnull
    public CreateAccentPhrasesConfigurator kana(boolean kana) {
      this.kana = kana;
      return this;
    }

    /**
     * {@link AccentPhrase} のリストを取得する。
     *
     * @return {@link AccentPhrase} のリスト。
     */
    @Nonnull
    public List<AccentPhrase> execute() {
      String accentPhrasesJson = synthesizer.rsAccentPhrases(this.text, this.styleId, this.kana);
      Gson gson = new Gson();
      AccentPhrase[] rawAccentPhrases = gson.fromJson(accentPhrasesJson, AccentPhrase[].class);
      if (rawAccentPhrases == null) {
        throw new NullPointerException("accent_phrases");
      }
      return new ArrayList<AccentPhrase>(Arrays.asList(rawAccentPhrases));
    }
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
    public byte[] execute() {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      Gson gson = new Gson();
      String queryJson = gson.toJson(this.audioQuery);
      return synthesizer.rsSynthesis(queryJson, this.styleId, this.interrogativeUpspeak);
    }
  }

  /** {@link Synthesizer#tts} のオプション。 */
  public class TtsConfigurator {
    private Synthesizer synthesizer;
    private String text;
    private int styleId;
    private boolean kana;
    private boolean interrogativeUpspeak;

    private TtsConfigurator(Synthesizer synthesizer, String text, int styleId) {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      this.synthesizer = synthesizer;
      this.text = text;
      this.styleId = styleId;
      this.kana = false;
    }

    /**
     * 入力テキストをAquesTalk風記法として解釈するかどうか。
     *
     * @param kana 入力テキストをAquesTalk風記法として解釈するかどうか。
     * @return {@link CreateAudioQueryConfigurator}。
     */
    @Nonnull
    public TtsConfigurator kana(boolean kana) {
      this.kana = kana;
      return this;
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
    public byte[] execute() {
      if (!Utils.isU32(styleId)) {
        throw new IllegalArgumentException("styleId");
      }
      return synthesizer.rsTts(this.text, this.styleId, this.kana, this.interrogativeUpspeak);
    }
  }
}
