package sharkbench;

import tools.jackson.core.JsonParser;
import tools.jackson.core.JsonToken;
import tools.jackson.core.StreamReadFeature;
import tools.jackson.core.json.JsonFactory;

import java.io.InputStream;
import java.util.Arrays;

final class PeriodicTableJson {
    private static final JsonFactory JSON = JsonFactory.builder()
            .disable(StreamReadFeature.AUTO_CLOSE_SOURCE)
            .build();

    private PeriodicTableJson() {
    }

    static ElementServiceClient.Element readElement(InputStream data, String symbol) {
        try (var parser = JSON.createParser(data)) {
            if (!selectSymbol(parser, symbol)) {
                return null;
            }
            if (parser.currentToken() != JsonToken.START_OBJECT) {
                throw new IllegalArgumentException("Expected an element object");
            }
            String name = null;
            int number = 0;
            int group = 0;
            String field;
            while ((field = parser.nextName()) != null) {
                parser.nextToken();
                switch (field) {
                    case "name" -> name = parser.currentToken() == JsonToken.VALUE_NULL ? null : parser.getString();
                    case "number" -> number = parser.getIntValue();
                    case "group" -> group = parser.getIntValue();
                    default -> parser.skipChildren();
                }
            }
            return new ElementServiceClient.Element(name, number, group);
        }
    }

    static int[] readShells(InputStream data, String symbol) {
        try (var parser = JSON.createParser(data)) {
            if (!selectSymbol(parser, symbol)) {
                return null;
            }
            if (parser.currentToken() != JsonToken.START_ARRAY) {
                throw new IllegalArgumentException("Expected a shell array");
            }
            int[] shells = new int[8];
            int count = 0;
            JsonToken token;
            while ((token = parser.nextToken()) != JsonToken.END_ARRAY) {
                if (token != JsonToken.VALUE_NUMBER_INT) {
                    throw new IllegalArgumentException("Expected a shell count");
                }
                if (count == shells.length) {
                    shells = Arrays.copyOf(shells, 2 * count);
                }
                shells[count++] = parser.getIntValue();
            }
            return count == shells.length ? shells : Arrays.copyOf(shells, count);
        }
    }

    private static boolean selectSymbol(JsonParser parser, String symbol) {
        if (parser.nextToken() != JsonToken.START_OBJECT) {
            throw new IllegalArgumentException("Expected an upstream JSON object");
        }
        String key;
        while ((key = parser.nextName()) != null) {
            JsonToken value = parser.nextToken();
            // The upstream periodic-table documents contain one value per symbol.
            if (symbol.equals(key)) {
                return value != JsonToken.VALUE_NULL;
            }
            parser.skipChildren();
        }
        return false;
    }
}
