// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Core",
    platforms: [.iOS(.v16)],
    dependencies: [
        .package(url: "https://example.invalid/core-support.git", from: "2.0.0"),
    ],
    targets: [
        .target(name: "Core"),
        .testTarget(name: "CoreTests", dependencies: ["Core"]),
    ]
)
