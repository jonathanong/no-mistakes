// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "ExampleAndroid",
  dependencies: [
    .package(path: "../core"),
    .package(path: "../ui"),
  ],
  targets: [
    .target(name: "ExampleAndroid", dependencies: ["ExampleCore", "ExampleFeatures"]),
    .testTarget(name: "ExampleAndroidTests", dependencies: ["ExampleAndroid"]),
  ]
)
