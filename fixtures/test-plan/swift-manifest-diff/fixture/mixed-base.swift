import PackageDescription
let package = Package(name: "App", platforms: [.iOS(.v16)], dependencies: [.package(url: "https://example.com/package.git", from: "1.0.0")])
