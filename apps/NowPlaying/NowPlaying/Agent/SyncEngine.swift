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
    private var previous = SyncAnchor.empty
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
        let oldTask = pollTask
        oldTask?.cancel()
        pollTask = Task { [weak self] in
            if let oldTask {
                _ = await oldTask.value
            }
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

    private struct PollCycleResult {
        let syncAnchor: SyncAnchor
        let payload: UpdateNowPlayingRequest?
        let displayStatus: AgentStatus
        let changed: Bool
    }

    private func pollOnce() async {
        if config.authToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            onStatusChange?(.error("Configure auth token in Preferences"))
            return
        }

        let pollConfig = config
        let previousSnapshot = previous

        let result = await Task.detached(priority: .utility) { () -> Result<PollCycleResult, Error> in
            do {
                let music = AppleMusicProvider()
                let track = try music.currentTrack()
                let syncAnchor = SyncAnchor.from(track: track)
                let changed = previousSnapshot.needsResync(track: track)
                let payload = changed ? try Self.buildUpdateRequest(track: track, music: music) : nil
                let displayStatus = Self.displayStatus(for: track)
                return .success(PollCycleResult(
                    syncAnchor: syncAnchor,
                    payload: payload,
                    displayStatus: displayStatus,
                    changed: changed
                ))
            } catch {
                return .failure(error)
            }
        }.value

        switch result {
        case .success(let cycle):
            if cycle.changed, let payload = cycle.payload {
                onStatusChange?(.syncing)
                let client = ApiClient(
                    baseURL: pollConfig.normalizedBaseURL,
                    authToken: pollConfig.authToken
                )
                do {
                    try await client.postNowPlaying(payload)
                    previous = cycle.syncAnchor.anchored(at: Date())
                    LogService.shared.info("Sent now-playing update to API")
                } catch {
                    LogService.shared.error("Poll cycle failed: \(error.localizedDescription)")
                    onStatusChange?(.error(error.localizedDescription))
                    return
                }
            }
            onStatusChange?(cycle.displayStatus)
        case .failure(let error):
            LogService.shared.error("Poll cycle failed: \(error.localizedDescription)")
            onStatusChange?(.error(error.localizedDescription))
        }
    }

    nonisolated private static func displayStatus(for track: NowPlayingTrack?) -> AgentStatus {
        guard let track else {
            return .lastTrack("Nothing playing")
        }

        if track.trackName.isEmpty {
            return .lastTrack("Nothing playing")
        }

        return .lastTrack("\(track.trackName) — \(track.artistName)")
    }

    nonisolated private static func buildUpdateRequest(
        track: NowPlayingTrack?,
        music: AppleMusicProvider
    ) throws -> UpdateNowPlayingRequest {
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
