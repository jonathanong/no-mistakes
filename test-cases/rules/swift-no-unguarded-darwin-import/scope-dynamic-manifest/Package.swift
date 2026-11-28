// swift-tools-version: 5.9
import PackageDescription

let configuredPlatforms: [SupportedPlatform] = [.macOS(.v14)]

let package = Package(
    name: "DynamicManifestKit",
    platforms: configuredPlatforms,
    targets: [
        .testTarget(name: "DynamicManifestTests"),
    ]
)
