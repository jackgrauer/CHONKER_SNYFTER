import Vision
import Foundation

func extractText(from imagePath: String) -> String {
    guard let imageData = NSData(contentsOfFile: imagePath) else {
        return ""
    }
    
    let request = VNRecognizeTextRequest()
    request.recognitionLevel = .accurate
    request.usesLanguageCorrection = true
    request.automaticallyDetectsLanguage = true
    
    let handler = VNImageRequestHandler(data: imageData as Data)
    
    do {
        try handler.perform([request])
        
        let results = request.results?.compactMap { result in
            (result as? VNRecognizedTextObservation)?.topCandidates(1).first?.string
        }.joined(separator: "\n") ?? ""
        
        return results
    } catch {
        return ""
    }
}

if CommandLine.argc < 2 {
    print("Usage: vision_ocr <image_path>")
    exit(1)
}

let imagePath = CommandLine.arguments[1]
let extractedText = extractText(from: imagePath)
print(extractedText)
