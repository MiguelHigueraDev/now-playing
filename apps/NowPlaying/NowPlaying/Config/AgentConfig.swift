import Foundation

struct AgentConfig: Equatable, Codable {
    var apiBaseURL: String
    var authToken: String
    var pollIntervalSecs: Int

    enum CodingKeys: String, CodingKey {
        case apiBaseURL = "api_base_url"
        case authToken = "auth_token"
        case pollIntervalSecs = "poll_interval_secs"
    }

    static func defaultTemplate() -> AgentConfig {
        AgentConfig(
            apiBaseURL: "http://localhost:3000",
            authToken: "",
            pollIntervalSecs: 3
        )
    }

    var normalizedBaseURL: String {
        apiBaseURL.trimmingCharacters(in: .whitespacesAndNewlines)
            .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    }

    var pollInterval: TimeInterval {
        TimeInterval(pollIntervalSecs)
    }

    /// Validates settings for saving (auth token required).
    func validateForSave() throws {
        if authToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            throw ConfigValidationError.emptyAuthToken
        }
        try Self.validatePollInterval(pollIntervalSecs)
    }

    static func validatePollInterval(_ value: Int) throws {
        guard (2 ... 5).contains(value) else {
            throw ConfigValidationError.invalidPollInterval
        }
    }
}

enum ConfigValidationError: LocalizedError {
    case emptyAuthToken
    case invalidPollInterval

    var errorDescription: String? {
        switch self {
        case .emptyAuthToken:
            return "Auth token must not be empty"
        case .invalidPollInterval:
            return "Poll interval must be between 2 and 5 seconds"
        }
    }
}
