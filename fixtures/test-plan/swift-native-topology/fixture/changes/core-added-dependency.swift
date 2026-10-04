// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "ExampleCore",
  products: [
    .library(name: "ExampleAPI", targets: ["ExampleAPI"]),
    .library(name: "ExampleCore", targets: ["ExampleCore"]),
  ],
  targets: [
    .target(name: "ExampleAPI"),
    .target(name: "ExampleCore", dependencies: ["ExampleAPI", "AddedDependency"]),
    .testTarget(name: "ExampleCoreTests", dependencies: ["ExampleCore", "ExampleAPI"]),
  ]
)
