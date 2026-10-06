package example

import kotlin.math.abs
import kotlin.math.roundToLong

fun main() {
    while (true) {
        val line = readlnOrNull()?.trim() ?: break
        if (line.isEmpty()) {
            continue
        }

        val iterations = line.toInt()
        val (pi, sum, customNumber) = calcPi(iterations)

        println("$pi;${formatFixed7(sum)};$customNumber")
    }
}

fun calcPi(iterations: Int): Triple<Double, Double, Double> {
    var pi = 0.0
    var denominator = 1.0

    var sum = 0.0
    var customNumber = 0.0

    for (x in 0 until iterations) {
        if (x % 2 == 0) {
            pi += 1.0 / denominator
        } else {
            pi -= 1.0 / denominator
        }
        denominator += 2.0

        // custom
        sum += pi
        when (x % 3) {
            0 -> customNumber += pi
            1 -> customNumber -= pi
            else -> customNumber /= 2.0
        }
    }

    pi *= 4.0
    return Triple(pi, sum, customNumber)
}

// Kotlin/Native has no String.format, so "%.7f" is done by hand
fun formatFixed7(value: Double): String {
    val scaled = abs(value * 10_000_000).roundToLong()
    val sign = if (value < 0) "-" else ""
    val fraction = (scaled % 10_000_000).toString().padStart(7, '0')
    return "$sign${scaled / 10_000_000}.$fraction"
}
