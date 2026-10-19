// swift-tools-version: 5.9
import PackageDescription
let package = Package(
    name: "Extra",
    targets: [
        .target(name: "Extra"),
        .testTarget(name: "ExtraTests", dependencies: ["Extra"]),
    ]
)
