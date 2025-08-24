import org.jetbrains.kotlin.konan.target.HostManager

val ktor_version: String by project
val kotlin_version: String by project
val logback_version: String by project

plugins {
    kotlin("multiplatform") version "2.2.10"
    id("org.jetbrains.kotlin.plugin.serialization") version "2.2.10"
}

group = "com.example"
version = "0.0.1"

kotlin {
    val arch = System.getProperty("os.arch")
    val nativeTarget = when {
        HostManager.hostIsMac && arch == "x86_64" -> macosX64("native")
        HostManager.hostIsMac && arch == "aarch64" -> macosArm64("native")
        HostManager.hostIsLinux -> linuxX64("native")
        // Other supported targets are listed here: https://ktor.io/docs/native-server.html#targets
        else -> throw GradleException("Host OS is not supported in Kotlin/Native.")
    }

    nativeTarget.binaries {
        executable {
            entryPoint = "com.example.main"
        }
    }

    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation("io.ktor:ktor-client-core:$ktor_version")
                implementation("io.ktor:ktor-serialization-kotlinx-json:$ktor_version")
                implementation("io.ktor:ktor-server-cio:$ktor_version")
                implementation("io.ktor:ktor-server-content-negotiation:$ktor_version")
                implementation("io.ktor:ktor-server-core:$ktor_version")
            }
        }

        val nativeMain by getting {
            dependencies {
                implementation("io.ktor:ktor-client-curl:${ktor_version}")
            }
        }
    }
}

repositories {
    mavenCentral()
}
