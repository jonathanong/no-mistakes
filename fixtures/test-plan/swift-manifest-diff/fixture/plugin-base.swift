import PackageDescription
let package = Package(targets: [.target(name: "App", plugins: [.plugin(name: "Lint", package: "tools")])])
