package org.acme;

import io.smallrye.mutiny.Uni;
import jakarta.inject.Inject;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.MediaType;
import org.eclipse.microprofile.rest.client.inject.RestClient;

import java.util.HashMap;
import java.util.Map;

@Path("/api/v1/periodic-table")
@Produces(MediaType.APPLICATION_JSON)
public class PeriodicTableResource {

    @Inject
    @RestClient
    ElementServiceClient elementServiceClient;

    @GET
    @Path("/element")
    public Uni<Map<String, Object>> getElement(@QueryParam("symbol") String symbol) {
        return Uni.createFrom().completionStage(elementServiceClient.getElements())
                .onItem().transform(elements -> {
                    Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);
                    return Map.of(
                            "name", elementValueOrDefault(elementData, "name"),
                            "number", elementValueOrDefault(elementData, "number"),
                            "group", elementValueOrDefault(elementData, "group")
                    );
                });
    }

    @GET
    @Path("/shells")
    public Uni<Map<String, Object>> getShells(@QueryParam("symbol") String symbol) {
        return Uni.createFrom().completionStage(elementServiceClient.getShells())
                .onItem().transform(elements -> Map.of("shells", elements.get(symbol)));
    }

    private Object elementValueOrDefault(Map<String, Object> elementData, String key) {
        return elementData != null ? elementData.getOrDefault(key, "") : "";
    }
}