// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "VouchaAndroid",
  dependencies: [
    .package(path: "../core"),
    .package(path: "../ui"),
  ],
  targets: [
    .target(name: "VouchaAndroid", dependencies: ["VouchaCore", "VouchaFeatures"]),
    .testTarget(name: "VouchaAndroidTests", dependencies: ["VouchaAndroid"]),
  ]
)
