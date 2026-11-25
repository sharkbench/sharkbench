package org.acme;

import io.smallrye.mutiny.Uni;
import jakarta.inject.Inject;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.MediaType;
import org.eclipse.microprofile.rest.client.inject.RestClient;

@Path("/api/v1/periodic-table")
@Produces(MediaType.APPLICATION_JSON)
public class PeriodicTableResource {

    public record ElementResponse(String name, int number, int group) {
    }

    public record ShellsResponse(int[] shells) {
    }

    @Inject
    @RestClient
    ElementServiceClient elementServiceClient;

    @GET
    @Path("/element")
    public Uni<ElementResponse> getElement(@QueryParam("symbol") String symbol) {
        return Uni.createFrom().completionStage(elementServiceClient.getElements())
                .onItem().transform(elements -> {
                    Element e = elements.get(symbol);
                    return e == null ? null : new ElementResponse(e.name, e.number, e.group);
                });
    }

    @GET
    @Path("/shells")
    public Uni<ShellsResponse> getShells(@QueryParam("symbol") String symbol) {
        return Uni.createFrom().completionStage(elementServiceClient.getShells())
                .onItem().transform(shells -> {
                    int[] s = shells.get(symbol);
                    return s == null ? null : new ShellsResponse(s);
                });
    }
}
