#include "mora_list.h"

std::string mora2text(std::string mora) {
  const std::string* mora_list = mora_list_minimum.data();
  // 万が一走査しても見つからなかった時のための初期値として、moraをセットしておく
  std::string text = mora;
  mora_list++;
  int count = 1;
  while (count < mora_list_minimum.size()) {
    if ((*mora_list + *(mora_list + 1)) == mora) {
      text = *(mora_list - 1);
      break;
    }
    mora_list += 3;
    count += 3;
  }
  return text;
}
