// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "benchmark",
    dependencies: [
        .package(url: "https://github.com/vapor/vapor.git", .upToNextMinor(from: "4.101.3"))
    ],
    targets: [
        .target(
            name: "benchmark",
            dependencies: [
                .product(name: "Vapor", package: "vapor")
            ],
            swiftSettings: [
                .unsafeFlags(["-cross-module-optimization"], .when(configuration: .release))
            ]),
    ]
)
