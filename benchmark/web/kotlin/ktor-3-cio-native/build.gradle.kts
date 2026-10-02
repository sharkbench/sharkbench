import org.jetbrains.kotlin.konan.target.HostManager

val ktor_version = "3.6.0"
val logback_version = "1.4.14"

plugins {
    kotlin("multiplatform") version "2.4.20"
    id("org.jetbrains.kotlin.plugin.serialization") version "2.4.20"
}

group = "com.example"
version = "0.0.1"

kotlin {
    val arch = System.getProperty("os.arch")
    val os = System.getProperty("os.name")

    val native =
        if (os == "Linux" && arch == "amd64") {
            linuxX64("native")
        } else if (os == "Linux" && arch == "aarch64") {
            linuxArm64("native")
        } else if (os == "Mac" && arch == "aarch64") {
            macosArm64("native")
        } else {
            throw IllegalStateException("Your OS is not supported by Ktor")
        }

    native.binaries {
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
