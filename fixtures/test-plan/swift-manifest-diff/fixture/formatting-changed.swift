import PackageDescription

let package = Package(
  dependencies: [
    .package(url: "https://example.com/package.git", from: "1.0.0")
  ]
)
