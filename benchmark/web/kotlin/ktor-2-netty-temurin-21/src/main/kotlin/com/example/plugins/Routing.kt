package com.example.plugins

import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.serialization.kotlinx.json.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

fun Application.configureRouting() {
    val client = HttpClient(CIO) {
        install(ContentNegotiation) {
            json(Json {
                ignoreUnknownKeys = true
            })
        }
    }
    val dataSourceUrl = "http://web-data-source/data.json"

    routing {
        get("/api/v1/periodic-table/element") {
            val symbol = call.request.queryParameters["symbol"]!!
            val elementsJson = client.get(dataSourceUrl).bodyAsText()
            val elements: Map<String, DataSourceElement> = Json.decodeFromString(elementsJson)
            val elementData = elements[symbol]!!

            call.respond(ElementResponse(
                name = elementData.name,
                number = elementData.number,
                group = elementData.group,
            ))
        }

        get("/api/v1/periodic-table/shells") {
            val symbol = call.request.queryParameters["symbol"]!!
            val elementsJson = client.get(dataSourceUrl).bodyAsText()
            val elements: Map<String, DataSourceElement> = Json.decodeFromString(elementsJson)
            val elementData = elements[symbol]!!

            call.respond(ShellsResponse(
                shells = elementData.shells
            ))
        }
    }
}

@Serializable
data class DataSourceElement(
    val name: String,
    val number: Int,
    val group: Int,
    val shells: List<Int>
)

@Serializable
data class ElementResponse(
    val name: String,
    val number: Int,
    val group: Int,
)

@Serializable
data class ShellsResponse(
    val shells: List<Int>
)