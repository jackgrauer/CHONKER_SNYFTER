// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "apple_vision_ocr",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .executable(
            name: "apple_vision_ocr",
            targets: ["apple_vision_ocr"]
        )
    ],
    targets: [
        .executableTarget(
            name: "apple_vision_ocr",
            path: "Sources"
        )
    ]
)
