// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "ExampleAndroid",
  targets: [
    .target(name: "ExampleAndroid"),
    .testTarget(name: "ExampleAndroidTests", dependencies: ["ExampleAndroid"]),
  ]
)
