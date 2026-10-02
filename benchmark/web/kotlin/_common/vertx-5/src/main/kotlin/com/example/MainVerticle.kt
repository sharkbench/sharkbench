package com.example

import io.vertx.ext.web.Router
import io.vertx.ext.web.RoutingContext
import io.vertx.ext.web.client.WebClient
import io.vertx.ext.web.client.WebClientOptions
import io.vertx.ext.web.codec.BodyCodec
import io.vertx.kotlin.coroutines.CoroutineRouterSupport
import io.vertx.kotlin.coroutines.CoroutineVerticle
import io.vertx.kotlin.coroutines.coAwait
import io.vertx.kotlin.coroutines.dispatcher
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class ElementResponse(
    val name: String,
    val number: Int,
    val group: Int? = null
)

@Serializable
data class ShellsResponse(
    val shells: List<Int>
)

class MainVerticle : CoroutineVerticle(), CoroutineRouterSupport {
    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    }
    val options = WebClientOptions()
        .setDefaultHost("web-data-source")
        .setDefaultPort(80)

    private lateinit var webClientElement: WebClient
    private lateinit var webClientShells: WebClient

    override suspend fun start() {
        webClientElement = WebClient.create(vertx, options)
        webClientShells = WebClient.create(vertx, options)
        
        val router = Router.router(vertx)

        router
            .get("/api/v1/periodic-table/element")
            .coHandler(requestHandler = this::getElement)

        router
            .get("/api/v1/periodic-table/shells")
            .coHandler(requestHandler = this::getShells)

        vertx.createHttpServer()
            .requestHandler(router)
            .listen(3000)
            .coAwait()
    }

    private suspend fun getElement(context: RoutingContext) {
        val symbol = context.request().getParam("symbol")
        if (symbol.isNullOrBlank()) {
            context.fail(400)
            return
        }

        val result = webClientElement.get("/element.json")
            .`as`(BodyCodec.string())
            .send()
            .coAwait()

        if (result.statusCode() != 200) {
            context.fail(502)
            return
        }

        val elements = json.decodeFromString<Map<String, ElementResponse>>(
            result.body()
        )

        val element = elements[symbol]
        if (element == null) {
            context.fail(404)
            return
        }

        context.response()
            .putHeader("Content-Type", "application/json")
            .end(json.encodeToString(element))
            .coAwait()
    }

    private suspend fun getShells(context: RoutingContext) {
        val symbol = context.request().getParam("symbol")
        if (symbol.isNullOrBlank()) {
            context.fail(400)
            return
        }

        val result = webClientShells.get("/shells.json")
            .`as`(BodyCodec.string())
            .send()
            .coAwait()

        if (result.statusCode() != 200) {
            context.fail(502)
            return
        }

        val elements = json.decodeFromString<Map<String, List<Int>>>(
            result.body()
        )

        val shells = elements[symbol]
        if (shells == null) {
            context.fail(404)
            return
        }

        context.response()
            .putHeader("Content-Type", "application/json")
            .end(json.encodeToString(ShellsResponse(shells)))
            .coAwait()
    }

    override suspend fun stop() {
        webClientElement.close()
        webClientShells.close()
    }
}