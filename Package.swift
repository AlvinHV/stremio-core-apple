// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.
let sha256 = "1bfb2f8863fe0ee5e0f0500e0b221c867451672882f1f13c38b859fd71caa13b  StremioCore.xcframework.zip";
let url = "https://github.com/Stremio/stremio-core-swift/releases/download/test-release-1/StremioCore.xcframework.zip";

import PackageDescription

let package = Package(
    name: "StremioCore",
    platforms: [
        .macCatalyst(.v13),
        .iOS(.v12),
        .visionOS(.v1),
        .tvOS(.v12)
    ],
    products: [
        .library(
            name: "StremioCore",
            targets: ["StremioCore", "XCFramework"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.0.0"),
    ],
    targets: [
        .target(name: "StremioCore",
                dependencies: ["Wrapper", .product(name: "SwiftProtobuf", package: "swift-protobuf")], plugins: [
                    .plugin(name: "SwiftProtobufPlugin", package: "swift-protobuf")
                ]),
        .target(name: "Wrapper", dependencies: []),
        //.binaryTarget(name: "XCFramework", path: ".build/StremioCore.xcframework")
        .binaryTarget(name: "XCFramework", url: url, checksum: sha256)
    ]
)
