import Foundation
import OSLog

final class LogService {
    static let shared = LogService()

    private let logger = Logger(subsystem: "com.nowplaying.agent", category: "agent")
    private var logFileURL: URL?
    private let queue = DispatchQueue(label: "com.nowplaying.agent.logging")

    private init() {}

    func configure(logDir: URL) {
        try? FileManager.default.createDirectory(at: logDir, withIntermediateDirectories: true)
        logFileURL = logDir.appendingPathComponent("agent.log")
    }

    func info(_ message: String) {
        logger.info("\(message, privacy: .public)")
        appendToFile(level: "INFO", message: message)
    }

    func warning(_ message: String) {
        logger.warning("\(message, privacy: .public)")
        appendToFile(level: "WARN", message: message)
    }

    func error(_ message: String) {
        logger.error("\(message, privacy: .public)")
        appendToFile(level: "ERROR", message: message)
    }

    private func appendToFile(level: String, message: String) {
        guard let logFileURL else { return }
        let timestamp = ISO8601DateFormatter().string(from: Date())
        let line = "\(timestamp) \(level) \(message)\n"
        queue.async {
            if !FileManager.default.fileExists(atPath: logFileURL.path) {
                FileManager.default.createFile(atPath: logFileURL.path, contents: nil)
            }
            guard let handle = try? FileHandle(forWritingTo: logFileURL) else {
                try? line.write(to: logFileURL, atomically: false, encoding: .utf8)
                return
            }
            handle.seekToEndOfFile()
            if let data = line.data(using: .utf8) {
                handle.write(data)
            }
            try? handle.close()
        }
    }
}
