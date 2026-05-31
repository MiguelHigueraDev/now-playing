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

    func hasChanged(from other: PlaybackSnapshot) -> Bool {
        self != other
    }
}
