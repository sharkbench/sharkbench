package com.example

import io.ktor.server.application.Application
import io.ktor.server.cio.CIO
import io.ktor.server.engine.embeddedServer

fun main(args: Array<String>) {
    embeddedServer(
        CIO,
        port = 3000,
        host = "0.0.0.0",
        module = Application::module,
    ).start(wait = true)
}

fun Application.module() {
    configureSerialization()
    configureRouting()
}
