// swift-tools-version: 5.9
import PackageDescription

let dependencyURL = "https://example.invalid/core-support.git"
let package = Package(
    name: "Core",
    dependencies: [.package(url: dependencyURL, from: "2.0.0")],
    targets: [
        .target(name: "Core"),
        .testTarget(name: "CoreTests", dependencies: ["Core"]),
    ]
)
