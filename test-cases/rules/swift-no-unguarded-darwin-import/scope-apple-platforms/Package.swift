// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AppleKit",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    targets: [
        .testTarget(name: "AppleKitTests"),
    ]
)
