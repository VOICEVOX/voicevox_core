package jp.Hiroshiba.VoicevoxCore;

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

  public void close() {
    rsDrop();
  }

  private native void rsNewWithInitialize(OpenJtalk openJtalk, SynthesizerBuilder builder);

  private native void rsLoadVoiceModel(VoiceModel voiceModel);

  private native void rsUnloadVoiceModel(String voiceModelId);

  private native boolean rsIsLoadedVoiceModel(String voiceModelId);

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

}
