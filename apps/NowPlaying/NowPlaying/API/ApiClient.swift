import Foundation

enum ApiClientError: LocalizedError {
    case invalidURL
    case requestFailed(String)
    case unexpectedStatus(status: Int, body: String)

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid API base URL"
        case .requestFailed(let message):
            return "HTTP request failed: \(message)"
        case .unexpectedStatus(let status, let body):
            return "API returned status \(status): \(body)"
        }
    }
}

final class ApiClient {
    private let session: URLSession
    private let baseURL: String
    private let authToken: String

    init(baseURL: String, authToken: String, session: URLSession = .shared) {
        self.baseURL = baseURL.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        self.authToken = authToken
        self.session = session
    }

    func postNowPlaying(_ payload: UpdateNowPlayingRequest) async throws {
        guard let url = URL(string: "\(baseURL)/api/now-playing") else {
            throw ApiClientError.invalidURL
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("Bearer \(authToken)", forHTTPHeaderField: "Authorization")

        let encoder = JSONEncoder()
        request.httpBody = try encoder.encode(payload)

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw ApiClientError.requestFailed(error.localizedDescription)
        }

        guard let http = response as? HTTPURLResponse else {
            throw ApiClientError.requestFailed("No HTTP response")
        }

        guard (200 ... 299).contains(http.statusCode) else {
            let body = String(data: data, encoding: .utf8) ?? ""
            throw ApiClientError.unexpectedStatus(status: http.statusCode, body: body)
        }
    }
}
