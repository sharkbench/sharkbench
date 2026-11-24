package sharkbench;

import io.micronaut.http.annotation.Get;
import io.micronaut.http.client.annotation.Client;
import io.micronaut.serde.annotation.Serdeable;

import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletionStage;

@Client("http://127.0.0.1:5002")
public interface ElementServiceClient {
    @Get("/element.json")
    CompletionStage<Map<String, Element>> getElements();

    @Get("/shells.json")
    CompletionStage<Map<String, List<Integer>>> getShells();

    @Serdeable
    record Element(String name, int number, int group) {
    }
}
