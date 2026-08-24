// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Android",
    dependencies: [.package(path: "../core")],
    targets: [
        .target(name: "Android", dependencies: [.product(name: "CoreKit", package: "core")]),
        .testTarget(name: "AndroidTests", dependencies: ["Android"]),
    ]
)
