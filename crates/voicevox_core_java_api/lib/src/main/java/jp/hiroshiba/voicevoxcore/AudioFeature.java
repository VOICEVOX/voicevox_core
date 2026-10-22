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
   * 総フレーム数。
   *
   * <p>この{@code AudioFeature}を生成するときに用いた{@code AudioQuery}の{@link
   * AudioQuery#frameLength() frameLength}に等しい。
   *
   * @return 総フレーム数。
   */
  public int length() {
    return rsLength();
  }

  /**
   * {@link #length() 長さ}が{@code 0}かどうか。
   *
   * @return 長さが{@code 0}なら{@code true}、それ以外は{@code false}。
   */
  public boolean isEmpty() {
    return length() == 0;
  }

  /**
   * 指定された範囲に対応した{@link Slice}を返す。
   *
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

  /** ある{@link AudioFeature}上の区間。 */
  public static final class Slice {
    private final AudioFeature audioFeature;

    // Invariant: [startInclusive, endExclusive) must be a valid range for audioFeature.
    private final int startInclusive;

    // Invariant: [startInclusive, endExclusive) must be a valid range audioFeature.
    private final int endExclusive;

    /**
     * 与えられた{@link AudioFeature}の全体を表す{@code Slice}を生成する。
     *
     * @param audioFeature 元の{@link AudioFeature}。
     */
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

    /**
     * 元の{@link AudioFeature}。
     *
     * @return 元の{@link AudioFeature}。
     */
    public AudioFeature getAudioFeature() {
      return audioFeature;
    }

    /**
     * この{@code Slice}の始端が、元の{@link AudioFeature}のどの箇所にあたるか。
     *
     * @return 元の{@link AudioFeature}上での開始フレーム番号。
     */
    public int getStartOffset() {
      return startInclusive;
    }

    /**
     * この{@code Slice}が持つフレーム数。
     *
     * @return フレーム数。
     */
    public int length() {
      return endExclusive - startInclusive;
    }

    /**
     * この{@code Slice}のサブスライスを返す。
     *
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
