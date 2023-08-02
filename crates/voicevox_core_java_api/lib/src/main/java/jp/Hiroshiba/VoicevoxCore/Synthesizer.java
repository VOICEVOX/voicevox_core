package jp.Hiroshiba.VoicevoxCore;

import java.util.EnumSet;

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

  public void close() {
    rsDrop();
  }

  private native void rsNewWithInitialize(OpenJtalk openJtalk, SynthesizerBuilder builder);

  private native void rsLoadVoiceModel(VoiceModel voiceModel);

  private native void rsUnloadVoiceModel(String voiceModelId);

  private native boolean rsIsLoadedVoiceModel(String voiceModelId);

  @Nonnull
  private native String rsAudioQuery(String text, int styleId, boolean kana);

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
}
