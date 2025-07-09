import Foundation
import Vision
import CoreImage
import AppKit

@main
struct AppleVisionOCR {
    static func main() {
        guard CommandLine.arguments.count > 1 else {
            print("Usage: apple_vision_ocr <image_path>")
            exit(1)
        }
        
        let imagePath = CommandLine.arguments[1]
        
        guard let image = loadImage(from: imagePath) else {
            print("Failed to load image: \(imagePath)")
            exit(1)
        }
        
        performOCR(on: image) { result in
            switch result {
            case .success(let text):
                print(text)
                exit(0)
            case .failure(let error):
                print("OCR failed: \(error)")
                exit(1)
            }
        }
        
        // Keep the program running until OCR completes
        RunLoop.main.run()
    }
    
    static func loadImage(from path: String) -> NSImage? {
        let url = URL(fileURLWithPath: path)
        return NSImage(contentsOf: url)
    }
    
    static func performOCR(on image: NSImage, completion: @escaping (Result<String, Error>) -> Void) {
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            completion(.failure(OCRError.invalidImage))
            return
        }
        
        let request = VNRecognizeTextRequest { request, error in
            if let error = error {
                completion(.failure(error))
                return
            }
            
            guard let observations = request.results as? [VNRecognizedTextObservation] else {
                completion(.failure(OCRError.noTextFound))
                return
            }
            
            let recognizedText = observations.compactMap { observation in
                observation.topCandidates(1).first?.string
            }.joined(separator: "\n")
            
            completion(.success(recognizedText))
        }
        
        // Configure text recognition request for better accuracy
        request.recognitionLevel = .accurate
        request.usesLanguageCorrection = true
        
        // Support multiple languages
        request.recognitionLanguages = ["en-US", "es-ES", "fr-FR", "de-DE", "it-IT", "pt-BR"]
        
        let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
        
        do {
            try handler.perform([request])
        } catch {
            completion(.failure(error))
        }
    }
}

enum OCRError: Error {
    case invalidImage
    case noTextFound
    
    var localizedDescription: String {
        switch self {
        case .invalidImage:
            return "Invalid image format"
        case .noTextFound:
            return "No text found in image"
        }
    }
}
