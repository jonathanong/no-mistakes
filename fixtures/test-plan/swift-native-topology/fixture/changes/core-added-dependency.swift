// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "VouchaCore",
  products: [
    .library(name: "VouchaAPI", targets: ["VouchaAPI"]),
    .library(name: "VouchaCore", targets: ["VouchaCore"]),
  ],
  targets: [
    .target(name: "VouchaAPI"),
    .target(name: "VouchaCore", dependencies: ["VouchaAPI", "AddedDependency"]),
    .testTarget(name: "VouchaCoreTests", dependencies: ["VouchaCore", "VouchaAPI"]),
  ]
)
