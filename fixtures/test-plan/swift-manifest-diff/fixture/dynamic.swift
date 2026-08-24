import PackageDescription
let url = "https://example.com/package.git"
let package = Package(dependencies: [.package(url: url, from: "1.0.0")])
