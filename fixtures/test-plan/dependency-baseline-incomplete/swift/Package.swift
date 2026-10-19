// swift-tools-version: 5.9
import PackageDescription
let package = Package(
    name: "Fixture",
    dependencies: [
        .package(url: "https://example.invalid/fixture-support.git", from: "1.0.0"),
    ],
    targets: [
        .target(name: "Fixture"),
    ]
)
