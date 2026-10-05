// swift-tools-version: 5.10

import PackageDescription

let package = Package(
    name: "benchmark",
    targets: [
        .executableTarget(name: "benchmark", path: "Sources"),
    ]
)
