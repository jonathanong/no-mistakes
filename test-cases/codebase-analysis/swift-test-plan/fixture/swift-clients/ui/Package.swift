// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "VouchaUI",
  dependencies: [
    .package(path: "../core"),
  ],
  targets: [
    .target(name: "VouchaFeatures", dependencies: [
      .product(name: "VouchaCore", package: "core"),
      .product(name: "VouchaAPI", package: "core"),
    ]),
    .testTarget(name: "VouchaUITests", dependencies: ["VouchaFeatures", "VouchaCore", "VouchaAPI"]),
  ]
)
