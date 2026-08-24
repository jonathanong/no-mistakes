// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "TargetOwnership",
    targets: [
        .target(name: "Core"),
        .executableTarget(
            name: "Runner",
            dependencies: ["Core"],
            path: "Tools/../Tools/Runner"
        ),
        .testTarget(
            name: "CustomTests",
            dependencies: ["Core"],
            path: "Checks/Integration"
        ),
        .plugin(
            name: "Plugin",
            capability: .buildTool(),
            dependencies: ["Core"],
            path: "Tooling/Plugin"
        ),
    ]
)
