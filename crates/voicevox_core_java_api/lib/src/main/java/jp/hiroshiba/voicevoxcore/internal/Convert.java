package jp.hiroshiba.voicevoxcore.internal;

import com.google.gson.Gson;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;

public class Convert {
  private Convert() {}

  public static String jsonFromQueryLike(Object object, String description) {
    Gson gson = new Gson();
    try {
      return gson.toJson(object);
    } catch (IllegalArgumentException e) {
      throw new InvalidQueryException(description, e);
    }
  }
}
