// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ClockKit",
    targets: [
        .target(name: "LocalClock", path: "Sources"),
        .testTarget(name: "ClockTests", path: "Tests"),
    ]
)
