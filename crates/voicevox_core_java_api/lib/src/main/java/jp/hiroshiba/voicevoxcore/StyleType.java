package jp.hiroshiba.voicevoxcore;

import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonParseException;
import com.google.gson.JsonPrimitive;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import java.lang.reflect.Type;

/** スタイル（style）に対応するモデルの種類。 */
public class StyleType {
  /** 音声合成クエリの作成と音声合成が可能。 */
  public static final StyleType TALK = new StyleType("talk");

  /** 歌唱音声合成用のクエリの作成が可能。 */
  public static final StyleType SINGING_TEACHER = new StyleType("singing_teacher");

  /** 歌唱音声合成が可能。 */
  public static final StyleType FRAME_DECODE = new StyleType("frame_decode");

  /** 歌唱音声合成用のクエリの作成と歌唱音声合成が可能。 */
  public static final StyleType SING = new StyleType("sing");

  public static final class Serializer implements JsonSerializer<StyleType> {
    @Override
    public JsonElement serialize(StyleType src, Type typeOfSrc, JsonSerializationContext context) {
      return new JsonPrimitive(src.toString());
    }
  }

  public static final class Deserializer implements JsonDeserializer<StyleType> {
    @Override
    public StyleType deserialize(JsonElement json, Type typeOfT, JsonDeserializationContext context)
        throws JsonParseException {
      String value = json.getAsString();
      switch (value) {
        case "talk":
          return TALK;
        case "singing_teacher":
          return SINGING_TEACHER;
        case "frame_decode":
          return FRAME_DECODE;
        case "sing":
          return SING;
        default:
          throw new JsonParseException(String.format("Invalid value: `%s`", value));
      }
    }
  }

  private final String identifier;

  private StyleType(String identifier) {
    this.identifier = identifier;
  }

  @Override
  public String toString() {
    return identifier;
  }
}
