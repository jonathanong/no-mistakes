// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "VouchaAndroid",
  targets: [
    .target(name: "VouchaAndroid"),
    .testTarget(name: "VouchaAndroidTests", dependencies: ["VouchaAndroid"]),
  ]
)
