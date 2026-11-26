// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "DeskWorkspace",
    targets: [
        .executableTarget(name: "DeskApp"),
        .testTarget(name: "DeskCoreTests"),
    ]
)
