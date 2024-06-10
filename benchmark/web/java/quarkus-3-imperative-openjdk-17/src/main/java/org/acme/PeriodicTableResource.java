package org.acme;

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
    public Map<String, Object> getElement(@QueryParam("symbol") String symbol) {
        Map<String, Object> elements = elementServiceClient.getElements();
        Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);

        Map<String, Object> map = new HashMap<>();
        map.put("name", elementData.get("name"));
        map.put("number", elementData.get("number"));
        map.put("group", elementData.get("group"));
        return map;
    }

    @GET
    @Path("/shells")
    public Map<String, Object> getShells(@QueryParam("symbol") String symbol) {
        Map<String, Object> elements = elementServiceClient.getShells();

        Map<String, Object> map = new HashMap<>();
        map.put("shells", elements.get(symbol));
        return map;
    }
}
