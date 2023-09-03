package jp.hiroshiba.voicevoxcore;

class Utils {
  static boolean isU8(int value) {
    return value >= 0 && value <= 255;
  }

  static boolean isU16(int value) {
    return value >= 0 && value <= 65535;
  }

  static boolean isU32(long value) {
    return value >= 0 && value <= 4294967295L;
  }
}
