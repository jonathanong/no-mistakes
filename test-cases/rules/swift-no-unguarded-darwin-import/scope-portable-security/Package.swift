// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "VaultWorkspace",
    targets: [
        .target(name: "VaultCore"),
        .testTarget(name: "VaultCoreTests"),
    ]
)
