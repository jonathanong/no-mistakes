// swift-tools-version: 5.9
import PackageDescription
let package = Package(
  name: "Android",
  dependencies: [.package(path: "../ui")],
  targets: [
    .target(name: "Android", dependencies: [.product(name: "UI", package: "ui")]),
    .testTarget(name: "AndroidTests", dependencies: ["Android"])
  ]
)
