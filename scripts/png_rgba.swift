// png_rgba.swift — PNG를 8비트 RGBA로 다시 인코딩한다 (tauri-codegen은 RGBA PNG만 받는다).
// 사용: swift scripts/png_rgba.swift <in.png> <out.png> [size] [--squircle]
// --squircle: macOS 앱 아이콘 모양(모서리 반지름 ≈ 22.37%)으로 잘라 투명 모서리를 만든다.
// macOS 26은 네 귀가 불투명한 정사각형 아이콘을 "구형"으로 보고 회색 타일 위에 작게 그리므로,
// .icns용 아트워크는 미리 둥글게 잘라 둔다(iyagi-term 94725c0과 같은 처리).
import AppKit

let args = CommandLine.arguments
guard args.count >= 3, let image = NSImage(contentsOfFile: args[1]) else {
    FileHandle.standardError.write("usage: png_rgba.swift <in.png> <out.png> [size] [--squircle]\n".data(using: .utf8)!)
    exit(1)
}
let squircle = args.contains("--squircle")
let sizeArg = args.dropFirst(3).first { Int($0) != nil }
let size = sizeArg.flatMap { Int($0) } ?? Int(image.size.width)
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
                           samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                           colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
NSGraphicsContext.current?.imageInterpolation = .high
let frame = NSRect(x: 0, y: 0, width: size, height: size)
if squircle {
    let radius = CGFloat(size) * 0.2237
    NSBezierPath(roundedRect: frame, xRadius: radius, yRadius: radius).addClip()
}
image.draw(in: frame, from: .zero, operation: .copy, fraction: 1.0)
NSGraphicsContext.restoreGraphicsState()
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: args[2]))
