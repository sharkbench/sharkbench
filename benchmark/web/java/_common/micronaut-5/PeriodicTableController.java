package sharkbench;

import io.micronaut.http.HttpStatus;
import io.micronaut.http.annotation.Controller;
import io.micronaut.http.annotation.Get;
import io.micronaut.http.annotation.QueryValue;
import io.micronaut.http.exceptions.HttpStatusException;
import io.micronaut.serde.annotation.Serdeable;

import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletionStage;

@Controller("/api/v1/periodic-table")
public class PeriodicTableController {
    private final ElementServiceClient client;

    public PeriodicTableController(ElementServiceClient client) {
        this.client = client;
    }

    @Get("/element")
    public CompletionStage<ElementServiceClient.Element> getElement(@QueryValue("symbol") String symbol) {
        return client.getElements().thenApply(elements -> findBySymbol(elements, symbol));
    }

    @Get("/shells")
    public CompletionStage<Shells> getShells(@QueryValue("symbol") String symbol) {
        return client.getShells().thenApply(shells -> new Shells(findBySymbol(shells, symbol)));
    }

    private static <T> T findBySymbol(Map<String, T> data, String symbol) {
        T value = data.get(symbol);
        if (value == null) {
            throw new HttpStatusException(HttpStatus.NOT_FOUND, "Unknown element: " + symbol);
        }
        return value;
    }

    @Serdeable
    public record Shells(List<Integer> shells) {
    }
}
