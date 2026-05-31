import Foundation

enum ConfigStoreError: LocalizedError {
    case unsupportedDirectory
    case readFailed(String)
    case writeFailed(String)
    case parseFailed(String)

    var errorDescription: String? {
        switch self {
        case .unsupportedDirectory:
            return "Failed to resolve Application Support directory"
        case .readFailed(let message):
            return "Failed to read config file: \(message)"
        case .writeFailed(let message):
            return "Failed to write config file: \(message)"
        case .parseFailed(let message):
            return "Failed to parse config file: \(message)"
        }
    }
}

final class ConfigStore {
    static let appDirName = "Now Playing"
    static let configFileName = "config.toml"

    let appDir: URL
    let configPath: URL

    init(appDir: URL, configPath: URL) {
        self.appDir = appDir
        self.configPath = configPath
    }

    var logDir: URL {
        appDir.appendingPathComponent("logs", isDirectory: true)
    }

    static func fallbackStore() throws -> ConfigStore {
        let appDir = try applicationSupportDirectory().appendingPathComponent(appDirName, isDirectory: true)
        try FileManager.default.createDirectory(at: appDir, withIntermediateDirectories: true)
        let configPath = appDir.appendingPathComponent(configFileName)
        return ConfigStore(appDir: appDir, configPath: configPath)
    }

    static func loadOrCreate() throws -> (ConfigStore, AgentConfig) {
        let appDir = try applicationSupportDirectory().appendingPathComponent(appDirName, isDirectory: true)
        try FileManager.default.createDirectory(at: appDir, withIntermediateDirectories: true)

        let configPath = appDir.appendingPathComponent(configFileName)
        let store = ConfigStore(appDir: appDir, configPath: configPath)

        if !FileManager.default.fileExists(atPath: configPath.path) {
            let template = AgentConfig.defaultTemplate()
            try store.save(template)
            return (store, template)
        }

        let config = try store.load()
        return (store, config)
    }

    func load() throws -> AgentConfig {
        let contents: String
        do {
            contents = try String(contentsOf: configPath, encoding: .utf8)
        } catch {
            throw ConfigStoreError.readFailed(error.localizedDescription)
        }

        do {
            return try SimpleTOML.decode(AgentConfig.self, from: contents)
        } catch {
            throw ConfigStoreError.parseFailed(error.localizedDescription)
        }
    }

    func save(_ config: AgentConfig) throws {
        let contents: String
        do {
            contents = try SimpleTOML.encode(config)
        } catch {
            throw ConfigStoreError.writeFailed(error.localizedDescription)
        }

        do {
            try contents.write(to: configPath, atomically: true, encoding: .utf8)
        } catch {
            throw ConfigStoreError.writeFailed(error.localizedDescription)
        }
    }

    private static func applicationSupportDirectory() throws -> URL {
        guard let url = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            throw ConfigStoreError.unsupportedDirectory
        }
        return url
    }
}

// MARK: - Minimal TOML (api_base_url, auth_token, poll_interval_secs)

enum SimpleTOML {
    static func encode<T: Encodable>(_ value: T) throws -> String {
        let encoder = JSONEncoder()
        let data = try encoder.encode(value)
        let dict = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]

        var lines: [String] = []
        for (key, raw) in dict.sorted(by: { $0.key < $1.key }) {
            lines.append("\(key) = \(formatValue(raw))")
        }
        return lines.joined(separator: "\n") + "\n"
    }

    static func decode<T: Decodable>(_ type: T.Type, from text: String) throws -> T {
        var dict: [String: Any] = [:]
        for line in text.split(separator: "\n", omittingEmptySubsequences: false) {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.isEmpty || trimmed.hasPrefix("#") { continue }
            guard let eq = trimmed.firstIndex(of: "=") else { continue }
            let key = String(trimmed[..<eq]).trimmingCharacters(in: .whitespaces)
            let valuePart = String(trimmed[trimmed.index(after: eq)...]).trimmingCharacters(in: .whitespaces)
            dict[key] = parseValue(valuePart)
        }
        let data = try JSONSerialization.data(withJSONObject: dict)
        return try JSONDecoder().decode(T.self, from: data)
    }

    private static func formatValue(_ value: Any) -> String {
        switch value {
        case let n as Int:
            return "\(n)"
        case let s as String:
            return "\"\(s.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "\"", with: "\\\""))\""
        case let b as Bool:
            return b ? "true" : "false"
        default:
            return "\"\(String(describing: value))\""
        }
    }

    private static func parseValue(_ raw: String) -> Any {
        if raw.hasPrefix("\""), raw.hasSuffix("\""), raw.count >= 2 {
            let inner = String(raw.dropFirst().dropLast())
            return inner
                .replacingOccurrences(of: "\\\"", with: "\"")
                .replacingOccurrences(of: "\\\\", with: "\\")
        }
        if let intVal = Int(raw) { return intVal }
        if raw == "true" { return true }
        if raw == "false" { return false }
        return raw
    }
}
