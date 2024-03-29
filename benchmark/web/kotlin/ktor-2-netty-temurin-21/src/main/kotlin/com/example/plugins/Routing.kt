package com.example.plugins

import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

fun Application.configureRouting() {
    val client = HttpClient(CIO)

    routing {
        get("/api/v1/periodic-table/element") {
            val symbol = call.request.queryParameters["symbol"]!!
            val elementsJson = client.get("http://web-data-source/element.json").bodyAsText()
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
            val elementsJson = client.get("http://web-data-source/shells.json").bodyAsText()
            val elements: Map<String, List<Int>> = Json.decodeFromString(elementsJson)

            call.respond(ShellsResponse(
                shells = elements[symbol]!!
            ))
        }
    }
}

@Serializable
data class DataSourceElement(
    val name: String,
    val number: Int,
    val group: Int,
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
