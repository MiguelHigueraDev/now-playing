import Foundation

/// Payload sent to `POST /api/now-playing` (matches `shared-types::UpdateNowPlayingRequest`).
struct UpdateNowPlayingRequest: Codable, Equatable {
    var trackName: String
    var artistName: String
    var albumName: String
    var artworkUrl: String?
    var artworkBase64: String?
    var durationSeconds: UInt32?
    var positionSeconds: UInt32?
    var isPlaying: Bool

    enum CodingKeys: String, CodingKey {
        case trackName = "track_name"
        case artistName = "artist_name"
        case albumName = "album_name"
        case artworkUrl = "artwork_url"
        case artworkBase64 = "artwork_base64"
        case durationSeconds = "duration_seconds"
        case positionSeconds = "position_seconds"
        case isPlaying = "is_playing"
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(trackName, forKey: .trackName)
        try container.encode(artistName, forKey: .artistName)
        try container.encode(albumName, forKey: .albumName)
        try container.encodeIfPresent(artworkUrl, forKey: .artworkUrl)
        if let artworkBase64, !artworkBase64.isEmpty {
            try container.encode(artworkBase64, forKey: .artworkBase64)
        }
        try container.encodeIfPresent(durationSeconds, forKey: .durationSeconds)
        try container.encodeIfPresent(positionSeconds, forKey: .positionSeconds)
        try container.encode(isPlaying, forKey: .isPlaying)
    }

    static func cleared() -> UpdateNowPlayingRequest {
        UpdateNowPlayingRequest(
            trackName: "",
            artistName: "",
            albumName: "",
            artworkUrl: nil,
            artworkBase64: nil,
            durationSeconds: nil,
            positionSeconds: nil,
            isPlaying: false
        )
    }
}

struct NowPlayingTrack {
    var trackName: String
    var artistName: String
    var albumName: String
    var durationSeconds: UInt32?
    var positionSeconds: UInt32?
    var isPlaying: Bool
}
