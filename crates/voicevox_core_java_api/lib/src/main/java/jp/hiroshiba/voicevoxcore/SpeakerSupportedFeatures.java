package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;

/** 話者の対応機能。 */
public class SpeakerSupportedFeatures {
  /** モーフィング機能への対応。 */
  @SerializedName("permitted_synthesis_morphing")
  @Expose
  @Nonnull
  SpeakerSupportPermittedSynthesisMorphing permittedSynthesisMorphing;

  SpeakerSupportedFeatures() {
    this.permittedSynthesisMorphing = SpeakerSupportPermittedSynthesisMorphing.NOTHING;
  }
}
