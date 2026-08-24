// swift-tools-version: 5.9
import PackageDescription
let package = Package(
    name: "Orphan",
    dependencies: [.package(url: "https://example.invalid/orphan.git", from: "1.0.0")],
    targets: []
)
