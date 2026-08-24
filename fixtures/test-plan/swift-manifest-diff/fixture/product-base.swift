import PackageDescription
let package = Package(targets: [.target(name: "App", dependencies: [.product(name: "Core", package: "core")])])
