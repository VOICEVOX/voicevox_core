package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.UUID;
import org.junit.jupiter.api.Test;

class VoiceModelTest extends TestUtils {
  @Test
  void checkUuid() throws IOException {
    UUID expected = UUID.fromString(Manifest.readJson().id);
    UUID actual = loadModel().id;
    assertEquals(expected, actual);
  }

  private static class Manifest {
    @SerializedName("id")
    @Expose
    @Nonnull
    String id;

    static Manifest readJson() throws IOException {
      Path path = new File("../../../model/sample.vvm/manifest.json").toPath();
      String json = new String(Files.readAllBytes(path));
      return new Gson().fromJson(json, Manifest.class);
    }
  }
}
