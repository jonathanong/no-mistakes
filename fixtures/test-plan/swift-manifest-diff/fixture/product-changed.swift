import PackageDescription
let package = Package(targets: [.target(name: "App", dependencies: [.product(name: "Models", package: "core")])])
