// swift-tools-version: 5.9
import PackageDescription
let package = Package(
    name: "UI",
    dependencies: [.package(name: "Core", path: "../core")],
    targets: [
        .target(name: "UI", dependencies: [.product(name: "Core", package: "core")]),
        .testTarget(name: "UITests", dependencies: ["UI"]),
    ]
)
