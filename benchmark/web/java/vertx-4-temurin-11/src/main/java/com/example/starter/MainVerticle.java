package com.example.starter;

import io.vertx.core.AbstractVerticle;
import io.vertx.core.Promise;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.web.Router;
import io.vertx.ext.web.RoutingContext;
import io.vertx.ext.web.client.WebClient;
import io.vertx.ext.web.client.WebClientOptions;
import io.vertx.ext.web.codec.BodyCodec;

public class MainVerticle extends AbstractVerticle {
    private WebClient webClientElement;
    private WebClient webClientShells;

    @Override
    public void start() {
        webClientElement = WebClient.create(vertx, new WebClientOptions().setDefaultHost("web-data-source").setDefaultPort(80));
        webClientShells = WebClient.create(vertx, new WebClientOptions().setDefaultHost("web-data-source").setDefaultPort(80));

        Router router = Router.router(vertx);

        router.get("/api/v1/periodic-table/element").handler(this::getElement);
        router.get("/api/v1/periodic-table/shells").handler(this::getShells);

        vertx.createHttpServer()
                .requestHandler(router)
                .listen(3000);
    }

    private void getElement(RoutingContext context) {
        String symbol = context.request().getParam("symbol");

        webClientElement.get("/element.json")
                .as(BodyCodec.jsonObject())
                .send(ar -> {
                    if (ar.succeeded()) {
                        JsonObject elements = ar.result().body();
                        JsonObject elementData = elements.getJsonObject(symbol);

                        JsonObject response = new JsonObject()
                                .put("name", elementData.getString("name"))
                                .put("number", elementData.getInteger("number"))
                                .put("group", elementData.getInteger("group"));

                        context.response()
                                .putHeader("Content-Type", "application/json")
                                .end(response.encode());
                    } else {
                        context.response().setStatusCode(500).end();
                    }
                });
    }

    private void getShells(RoutingContext context) {
        String symbol = context.request().getParam("symbol");

        webClientShells.get("/shells.json")
                .as(BodyCodec.jsonObject())
                .send(ar -> {
                    if (ar.succeeded()) {
                        JsonObject elements = ar.result().body();
                        JsonObject response = new JsonObject()
                                .put("shells", elements.getJsonArray(symbol));

                        context.response()
                                .putHeader("Content-Type", "application/json")
                                .end(response.encode());
                    } else {
                        context.response().setStatusCode(500).end();
                    }
                });
    }
}
