// swift-tools-version: 5.9
import PackageDescription

func buildTargets() -> [Target] {
    [.testTarget(name: "DynamicTargetTests")]
}

let package = Package(
    name: "DynamicTargetsKit",
    targets: buildTargets()
)
