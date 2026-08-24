import PackageDescription
let package = Package(name: "App", platforms: [.iOS(.v17)], dependencies: [.package(url: "https://example.com/package.git", from: "2.0.0")])
