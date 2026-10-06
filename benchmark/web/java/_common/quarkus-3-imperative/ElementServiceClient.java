package org.acme;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;
import org.eclipse.microprofile.rest.client.inject.RegisterRestClient;

import java.util.Map;

@Path("/")
@RegisterRestClient(configKey="element-api")
public interface ElementServiceClient {

    @GET
    @Path("/element.json")
    @Produces(MediaType.APPLICATION_JSON)
    Map<String, Object> getElements();

    @GET
    @Path("/shells.json")
    @Produces(MediaType.APPLICATION_JSON)
    Map<String, Object> getShells();
}

