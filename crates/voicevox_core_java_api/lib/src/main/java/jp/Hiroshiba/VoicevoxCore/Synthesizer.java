package jp.Hiroshiba.VoicevoxCore;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.EnumSet;
import java.util.List;

import javax.annotation.Nonnull;

import com.google.gson.Gson;

public class Synthesizer implements AutoCloseable {
  protected long internal;

  private Synthesizer(OpenJtalk openJtalk, SynthesizerBuilder builder) {
    rsNewWithInitialize(openJtalk, builder);
  }

  public void loadVoiceModel(VoiceModel voiceModel) {
    rsLoadVoiceModel(voiceModel);
  }

  public void unloadVoiceModel(String voiceModelId) {
    rsUnloadVoiceModel(voiceModelId);
  }

  public boolean isLoadedVoiceModel(String voiceModelId) {
    return rsIsLoadedVoiceModel(voiceModelId);
  }

  @Nonnull
  public AudioQuery audioQuery(String text, int styleId, EnumSet<AudioQueryOption> options) {
    boolean kana = options.contains(AudioQueryOption.KANA);
    String queryJson = rsAudioQuery(text, styleId, kana);
    Gson gson = new Gson();

    System.out.println(queryJson);

    AudioQuery audioQuery = gson.fromJson(queryJson, AudioQuery.class);
    System.out.println(audioQuery);
    if (audioQuery == null) {
      throw new NullPointerException("audio_query");
    }
    return audioQuery;
  }

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

  @Nonnull
  public List<AccentPhrase> replaceMoraData(List<AccentPhrase> accentPhrases, int styleId) {
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplaceMoraData(accentPhrasesJson, styleId, false);
    return new ArrayList<>(Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  @Nonnull
  public List<AccentPhrase> replacePhonemeLength(List<AccentPhrase> accentPhrases, int styleId) {
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplacePhonemeLength(accentPhrasesJson, styleId, false);
    return new ArrayList<>(Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  @Nonnull
  public List<AccentPhrase> replaceMoraPitch(List<AccentPhrase> accentPhrases, int styleId) {
    String accentPhrasesJson = new Gson().toJson(accentPhrases);
    String replacedAccentPhrasesJson = rsReplaceMoraPitch(accentPhrasesJson, styleId, false);
    return new ArrayList<>(Arrays.asList(new Gson().fromJson(replacedAccentPhrasesJson, AccentPhrase[].class)));
  }

  @Nonnull
  public byte[] synthesis(AudioQuery audioQuery, int styleId, EnumSet<SynthesisOption> options) {
    boolean enableInterrogativeUpspeak = options.contains(SynthesisOption.ENABLE_INTERROGATIVE_UPSPEAK);
    Gson gson = new Gson();
    String queryJson = gson.toJson(audioQuery);
    return rsSynthesis(queryJson, styleId, enableInterrogativeUpspeak);
  }

  @Nonnull
  public byte[] tts(String text, int styleId, EnumSet<TtsOption> options) {
    boolean kana = options.contains(TtsOption.KANA);
    boolean enableInterrogativeUpspeak = options.contains(TtsOption.ENABLE_INTERROGATIVE_UPSPEAK);
    return rsTts(text, styleId, kana, enableInterrogativeUpspeak);
  }

  public void close() {
    rsDrop();
  }

  private native void rsNewWithInitialize(OpenJtalk openJtalk, SynthesizerBuilder builder);

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

  static {
    System.loadLibrary("voicevox_core_java_api");
  }

  public static SynthesizerBuilder builder(OpenJtalk openJtalk) {
    return new SynthesizerBuilder(openJtalk);
  }

  public static class SynthesizerBuilder {
    private OpenJtalk openJtalk;
    @SuppressWarnings("unused")
    private AccelerationMode accelerationMode;
    @SuppressWarnings("unused")
    private int cpuNumThreads;
    @SuppressWarnings("unused")
    private boolean loadAllModels;

    public SynthesizerBuilder(OpenJtalk openJtalk) {
      this.openJtalk = openJtalk;
    }

    public SynthesizerBuilder accelerationMode(AccelerationMode accelerationMode) {
      this.accelerationMode = accelerationMode;
      return this;
    }

    public SynthesizerBuilder cpuNumThreads(int cpuNumThreads) {
      this.cpuNumThreads = cpuNumThreads;
      return this;
    }

    public SynthesizerBuilder loadAllModels(boolean loadAllModels) {
      this.loadAllModels = loadAllModels;
      return this;
    }

    public Synthesizer build() {
      Synthesizer synthesizer = new Synthesizer(openJtalk, this);
      return synthesizer;
    }
  }

  public static enum AccelerationMode {
    AUTO,
    CPU,
    GPU,
  }

  public static enum AudioQueryOption {
    KANA,
  }

  public static enum AccentPhrasesOption {
    KANA,
  }

  public static enum SynthesisOption {
    ENABLE_INTERROGATIVE_UPSPEAK
  }

  public static enum TtsOption {
    KANA,
    ENABLE_INTERROGATIVE_UPSPEAK
  }
}
