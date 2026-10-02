plugins {
    kotlin("jvm") version "2.4.20"
    kotlin("plugin.serialization") version "2.4.20"
    application
}

group = "com.example"
version = "1.0.0"
val mainVerticleName = "$group.MainVerticle"
val launcherClassName = "io.vertx.launcher.application.VertxApplication"

repositories {
    mavenCentral()
}

dependencies {
    implementation(platform("io.vertx:vertx-stack-depchain:5.2.0"))
    implementation("io.vertx:vertx-core")
    implementation("io.vertx:vertx-lang-kotlin-coroutines")
    implementation("io.vertx:vertx-launcher-application")
    implementation("io.vertx:vertx-web")
    implementation("io.vertx:vertx-web-client")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.11.0")
}

application {
    this.mainClass = launcherClassName
}

tasks.jar {
    this.manifest {
        this.attributes["Main-Class"] = launcherClassName
        this.attributes["Main-Verticle"] = mainVerticleName
    }
}