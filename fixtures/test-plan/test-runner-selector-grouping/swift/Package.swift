// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "SelectorFixture",
  targets: [
    .target(name: "App"),
    .testTarget(name: "AlphaTests", dependencies: ["App"]),
    .testTarget(name: "BetaTests", dependencies: ["App"]),
  ]
)
