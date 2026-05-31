import Foundation

enum AgentStatus: Equatable {
    case idle
    case syncing
    case lastTrack(String)
    case error(String)

    var menuLabel: String {
        switch self {
        case .idle:
            return "Status: Idle"
        case .syncing:
            return "Status: Syncing…"
        case .lastTrack(let track):
            return "Status: \(track)"
        case .error(let message):
            return "Status: Error — \(message)"
        }
    }
}

@MainActor
final class SyncEngine {
    private let music = AppleMusicProvider()
    private var previous = PlaybackSnapshot.empty
    private var pollTask: Task<Void, Never>?
    private var config: AgentConfig
    var onStatusChange: ((AgentStatus) -> Void)?

    init(config: AgentConfig) {
        self.config = config
    }

    func updateConfig(_ config: AgentConfig) {
        self.config = config
        restartPolling()
    }

    func start() {
        restartPolling()
    }

    func stop() {
        pollTask?.cancel()
        pollTask = nil
    }

    private func restartPolling() {
        pollTask?.cancel()
        pollTask = Task { [weak self] in
            guard let self else { return }
            await self.runLoop()
        }
    }

    private func runLoop() async {
        onStatusChange?(.idle)

        while !Task.isCancelled {
            let interval = config.pollInterval
            await pollOnce()
            try? await Task.sleep(for: .seconds(interval))
        }
    }

    private func pollOnce() async {
        if config.authToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            onStatusChange?(.error("Configure auth token in Preferences"))
            return
        }

        do {
            let track = try music.currentTrack()
            let snapshot = PlaybackSnapshot.from(track: track)
            let displayStatus = displayStatus(for: track)

            if snapshot.hasChanged(from: previous) {
                onStatusChange?(.syncing)
                let payload = try buildUpdateRequest(track: track)
                let client = ApiClient(
                    baseURL: config.normalizedBaseURL,
                    authToken: config.authToken
                )
                try await client.postNowPlaying(payload)
                previous = snapshot
                LogService.shared.info("Sent now-playing update to API")
            }

            onStatusChange?(displayStatus)
        } catch {
            LogService.shared.error("Poll cycle failed: \(error.localizedDescription)")
            onStatusChange?(.error(error.localizedDescription))
        }
    }

    private func displayStatus(for track: NowPlayingTrack?) -> AgentStatus {
        guard let track else {
            return .lastTrack("Nothing playing")
        }

        if track.trackName.isEmpty {
            return .lastTrack("Nothing playing")
        }

        return .lastTrack("\(track.trackName) — \(track.artistName)")
    }

    private func buildUpdateRequest(track: NowPlayingTrack?) throws -> UpdateNowPlayingRequest {
        guard let track else {
            return .cleared()
        }

        var request = UpdateNowPlayingRequest(
            trackName: track.trackName,
            artistName: track.artistName,
            albumName: track.albumName,
            artworkUrl: nil,
            artworkBase64: nil,
            durationSeconds: track.durationSeconds,
            positionSeconds: track.positionSeconds,
            isPlaying: track.isPlaying
        )

        if let artwork = try? music.currentArtwork() {
            request.artworkBase64 = artwork.data.base64EncodedString()
        }

        return request
    }
}
