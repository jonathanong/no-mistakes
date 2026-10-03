// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "ExampleUI",
  dependencies: [
    .package(path: "../core"),
  ],
  targets: [
    .target(name: "ExampleFeatures", dependencies: [
      .product(name: "ExampleCore", package: "core"),
      .product(name: "ExampleAPI", package: "core"),
    ]),
    .testTarget(name: "ExampleUITests", dependencies: ["ExampleFeatures", "ExampleCore", "ExampleAPI"]),
  ]
)
