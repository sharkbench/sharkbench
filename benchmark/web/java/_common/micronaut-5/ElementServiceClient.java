package sharkbench;

import io.micronaut.http.ByteBodyHttpResponse;
import io.micronaut.http.HttpRequest;
import io.micronaut.http.HttpStatus;
import io.micronaut.http.body.CloseableAvailableByteBody;
import io.micronaut.http.client.RawHttpClient;
import io.micronaut.http.client.annotation.Client;
import io.micronaut.serde.annotation.Serdeable;
import jakarta.inject.Singleton;
import reactor.core.publisher.Mono;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.net.URI;
import java.util.concurrent.CompletionStage;
import java.util.function.Function;

@Singleton
final class ElementServiceClient {
    private static final URI ELEMENTS = URI.create("/element.json");
    private static final URI SHELLS = URI.create("/shells.json");

    private final RawHttpClient client;

    ElementServiceClient(@Client("http://127.0.0.1:5002") RawHttpClient client) {
        this.client = client;
    }

    CompletionStage<Element> getElement(String symbol) {
        return read(ELEMENTS, input -> PeriodicTableJson.readElement(input, symbol));
    }

    CompletionStage<int[]> getShells(String symbol) {
        return read(SHELLS, input -> PeriodicTableJson.readShells(input, symbol));
    }

    private <T> CompletionStage<T> read(URI uri, Function<InputStream, T> decode) {
        return Mono.from(client.exchange(HttpRequest.GET(uri), null, null))
                .flatMap(response -> {
                    if (!(response instanceof ByteBodyHttpResponse<?> rawResponse)) {
                        return Mono.error(new IllegalStateException("Expected a raw upstream response"));
                    }
                    return Mono.using(
                            () -> rawResponse,
                            raw -> raw.code() == HttpStatus.OK.getCode()
                                    ? Mono.fromFuture(raw.byteBody().buffer()).mapNotNull(body -> decodeBody(body, decode))
                                    : Mono.error(new IllegalStateException("Unexpected upstream status: " + raw.code())),
                            ByteBodyHttpResponse::close);
                })
                .doOnDiscard(CloseableAvailableByteBody.class, CloseableAvailableByteBody::close)
                .doOnDiscard(ByteBodyHttpResponse.class, ByteBodyHttpResponse::close)
                .toFuture();
    }

    private static <T> T decodeBody(CloseableAvailableByteBody body, Function<InputStream, T> decode) {
        // This stream reads an already buffered body, without waiting for upstream I/O.
        try (body; var input = body.toInputStream()) {
            return decode.apply(input);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    @Serdeable
    record Element(String name, int number, int group) {
    }
}
