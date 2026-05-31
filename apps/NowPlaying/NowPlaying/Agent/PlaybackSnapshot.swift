import Foundation

struct PlaybackSnapshot: Equatable {
    var trackName: String
    var artistName: String
    var albumName: String
    var isPlaying: Bool

    static let empty = PlaybackSnapshot(
        trackName: "",
        artistName: "",
        albumName: "",
        isPlaying: false
    )

    static func from(track: NowPlayingTrack?) -> PlaybackSnapshot {
        guard let track else { return .empty }
        return PlaybackSnapshot(
            trackName: track.trackName,
            artistName: track.artistName,
            albumName: track.albumName,
            isPlaying: track.isPlaying
        )
    }
}

/// Last state successfully posted to the API; used to detect seeks and metadata changes.
struct SyncAnchor: Equatable {
    var snapshot: PlaybackSnapshot
    var positionSeconds: UInt32?
    var isPlaying: Bool
    var syncedAt: Date

    static let empty = SyncAnchor(
        snapshot: .empty,
        positionSeconds: nil,
        isPlaying: false,
        syncedAt: .distantPast
    )

    /// Seconds of drift between Apple Music and our extrapolated anchor before treating it as a seek.
    private static let positionDriftThreshold = 3

    static func from(track: NowPlayingTrack?, syncedAt: Date = Date()) -> SyncAnchor {
        guard let track else { return .empty }
        return SyncAnchor(
            snapshot: PlaybackSnapshot.from(track: track),
            positionSeconds: track.positionSeconds,
            isPlaying: track.isPlaying,
            syncedAt: syncedAt
        )
    }

    func anchored(at date: Date) -> SyncAnchor {
        SyncAnchor(
            snapshot: snapshot,
            positionSeconds: positionSeconds,
            isPlaying: isPlaying,
            syncedAt: date
        )
    }

    func needsResync(track: NowPlayingTrack?, now: Date = Date()) -> Bool {
        let currentSnapshot = PlaybackSnapshot.from(track: track)

        guard let track else {
            return snapshot != .empty
        }

        if snapshot != currentSnapshot {
            return true
        }

        guard let anchorPos = positionSeconds else {
            return true
        }

        let currentPos = track.positionSeconds ?? 0

        if !track.isPlaying {
            return currentPos != anchorPos
        }

        guard isPlaying else {
            return true
        }

        let elapsed = UInt32(max(0, now.timeIntervalSince(syncedAt).rounded()))
        var expected = anchorPos &+ elapsed
        if let duration = track.durationSeconds {
            expected = min(expected, duration)
        }

        let drift = abs(Int(currentPos) - Int(expected))
        return drift > Self.positionDriftThreshold
    }
}
