package jp.hiroshiba.voicevoxcore;

import jp.hiroshiba.voicevoxcore.internal.Dll;

/** AudioFeature (音声の中間表現)。 */
public final class AudioFeature {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  private AudioFeature() {}

  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  /**
   * 音声のフレーム数を取得する。
   *
   * @return 音声のフレーム数。
   */
  public int length() {
    return rsLength();
  }

  public boolean isEmpty() {
    return length() == 0;
  }

  /**
   * @param startInclusive 開始フレーム番号。
   * @param endExclusive 終了フレーム番号（この番号は含まれない）。
   * @return スライスされた{@link Slice}。
   * @throws IndexOutOfBoundsException {@code startInclusive}または{@code endExclusive}が{@link
   *     #length()}を超えている場合。
   * @throws IllegalArgumentException {@code startInclusive > endExclusive}の場合
   */
  public Slice slice(int startInclusive, int endExclusive) {
    return new Slice(this).slice(startInclusive, endExclusive);
  }

  private native int rsLength();

  private native void rsDrop();

  public static final class Slice {
    private final AudioFeature audioFeature;

    // Invariant: [startInclusive, endExclusive) must be a valid range for audioFeature.
    private final int startInclusive;

    // Invariant: [startInclusive, endExclusive) must be a valid range audioFeature.
    private final int endExclusive;

    public Slice(AudioFeature audioFeature) {
      this.audioFeature = audioFeature;
      this.startInclusive = 0;
      this.endExclusive = audioFeature.length();
    }

    private Slice(AudioFeature audioFeature, int startInclusive, int endExclusive) {
      this.audioFeature = audioFeature;
      this.startInclusive = startInclusive;
      this.endExclusive = endExclusive;
    }

    public AudioFeature getAudioFeature() {
      return audioFeature;
    }

    public int getStartOffset() {
      return startInclusive;
    }

    public int length() {
      return endExclusive - startInclusive;
    }

    /**
     * @param startInclusive 開始フレーム番号。
     * @param endExclusive 終了フレーム番号（この番号は含まれない）。
     * @return スライスされた{@code Slice}。
     * @throws IndexOutOfBoundsException {@code startInclusive}または{@code endExclusive}が{@link
     *     #length()}を超えている場合。
     * @throws IllegalArgumentException {@code startInclusive > endExclusive}の場合
     */
    public Slice slice(int startInclusive, int endExclusive) {
      long range =
          rsValidateRange(
              audioFeature, this.startInclusive, this.endExclusive, startInclusive, endExclusive);
      return new Slice(audioFeature, (int) range, (int) (range >> 32));
    }

    private static native long rsValidateRange(
        AudioFeature audioFeature,
        int currentStartInclusive,
        int currentEndExclusive,
        int slicingStartInclusive,
        int slicingEndExclusive);
  }
}
