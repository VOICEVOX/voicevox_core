package jp.Hiroshiba.VoicevoxCore;

import java.util.HashMap;

import javax.annotation.Nonnull;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

public class UserDict
    implements AutoCloseable {
  protected long internal;

  public UserDict() {
    rsNew();
  }

  @Nonnull
  public String addWord(Word word) {
    Gson gson = new Gson();
    String wordJson = gson.toJson(word);

    return rsAddWord(wordJson);
  }

  public void updateWord(String uuid, Word word) {
    Gson gson = new Gson();
    String wordJson = gson.toJson(word);

    rsUpdateWord(uuid, wordJson);
  }

  public void removeWord(String uuid) {
    rsRemoveWord(uuid);
  }

  public void load(String path) {
    rsLoad(path);
  }

  public void save(String path) {
    rsSave(path);
  }

  @Nonnull
  public HashMap<String, Word> getWords() {
    String json = rsGetWords();
    Gson gson = new Gson();
    @SuppressWarnings("unchecked")
    HashMap<String, Word> words = gson.fromJson(json, HashMap.class);
    if (words == null) {
      throw new NullPointerException("words");
    }
    return words;
  }

  public void close() {
    rsDrop();
  }

  private native void rsNew();

  @Nonnull
  private native String rsAddWord(String word);

  private native void rsUpdateWord(String uuid, String word);

  private native void rsRemoveWord(String uuid);

  private native void rsLoad(String path);

  private native void rsSave(String path);

  @Nonnull
  private native String rsGetWords();

  private native void rsDrop();

  @Nonnull
  private static native String rsToZenkaku(String surface);

  private static native void rsValidatePronunciation(String pronunciation);

  public static class Word {
    @JsonProperty("surface")
    @SerializedName("surface")
    @Expose
    @Nonnull
    public String surface;

    @JsonProperty("pronunciation")
    @SerializedName("pronunciation")
    @Expose
    @Nonnull
    public String pronunciation;

    @JsonProperty("word_type")
    @SerializedName("word_type")
    @Expose
    @Nonnull
    public Type wordType;

    @JsonProperty("accent_type")
    @SerializedName("accent_type")
    @Expose
    public int accentType;

    @JsonProperty("priority")
    @SerializedName("priority")
    @Expose
    @Min(1)
    @Max(10)
    public int priority;

    public Word(String surface, String pronunciation) {
      if (surface == null) {
        throw new NullPointerException("surface");
      }
      if (pronunciation == null) {
        throw new NullPointerException("pronunciation");
      }

      this.surface = rsToZenkaku(surface);
      rsValidatePronunciation(pronunciation);
      this.pronunciation = pronunciation;
      this.wordType = Type.COMMON_NOUN;
      this.accentType = 0;
      this.priority = 5;
    }

    public Word wordType(Type wordType) {
      if (wordType == null) {
        throw new NullPointerException("wordType");
      }
      this.wordType = wordType;
      return this;
    }

    public Word accentType(int accentType) {
      if (accentType < 0) {
        throw new IllegalArgumentException("accentType");
      }
      this.accentType = accentType;
      return this;
    }

    public Word priority(int priority) {
      if (priority < 0 || priority > 10) {
        throw new IllegalArgumentException("priority");
      }
      this.priority = priority;
      return this;
    }

    static enum Type {
      @JsonProperty("PROPER_NOUN")
      @SerializedName("PROPER_NOUN")
      @Expose
      PROPER_NOUN,

      @JsonProperty("COMMON_NOUN")
      @SerializedName("COMMON_NOUN")
      @Expose
      COMMON_NOUN,

      @JsonProperty("VERB")
      @SerializedName("VERB")
      @Expose
      VERB,

      @JsonProperty("ADJECTIVE")
      @SerializedName("ADJECTIVE")
      @Expose
      ADJECTIVE,

      @JsonProperty("SUFFIX")
      @SerializedName("SUFFIX")
      @Expose
      SUFFIX,
    }
  }
}
