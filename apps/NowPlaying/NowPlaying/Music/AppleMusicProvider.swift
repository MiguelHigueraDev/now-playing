import AppKit
import Foundation

enum AppleMusicError: LocalizedError {
    case scriptFailed(String)
    case unexpectedOutput(String)

    var errorDescription: String? {
        switch self {
        case .scriptFailed(let message):
            return message
        case .unexpectedOutput(let message):
            return "Unexpected Apple Music output: \(message)"
        }
    }
}

struct TrackArtwork {
    let data: Data
    let contentType: String
}

final class AppleMusicProvider {
    private static let trackScript = """
    tell application "Music"
        set ps to player state as string
        if ps is "playing" or ps is "paused" then
            set t to current track
            set isPlaying to ps is "playing"
            return name of t & "||" & artist of t & "||" & album of t & "||" & (duration of t as string) & "||" & (player position as string) & "||" & isPlaying
        else
            return "NOT_PLAYING"
        end if
    end tell
    """

    func currentTrack() throws -> NowPlayingTrack? {
        let raw = try runAppleScript(Self.trackScript)
        return try Self.parseTrackOutput(raw)
    }

    func currentArtwork() throws -> TrackArtwork? {
        let path = Self.artworkCachePath()
        let pathStr = path.path
        let script = """
        tell application "Music"
            set ps to player state as string
            if ps is not "playing" and ps is not "paused" then
                return "NOT_PLAYING"
            end if
            set t to current track
            if (count of artworks of t) is 0 then
                return "NO_ART"
            end if
            set artPath to "\(pathStr)"
            tell artwork 1 of t
                if format is «class PNG » then
                    set ext to ".png"
                else
                    set ext to ".jpg"
                end if
                set srcBytes to raw data
            end tell
            if ext is ".png" then
                set artPath to my replace_text(artPath, ".jpg", ".png")
            end if
            set outFile to open for access POSIX file artPath with write permission
            set eof outFile to 0
            write srcBytes to outFile
            close access outFile
            return artPath
        end tell

        on replace_text(sourceText, findText, replaceText)
            set AppleScript's text item delimiters to findText
            set parts to text items of sourceText
            set AppleScript's text item delimiters to replaceText
            return parts as text
        end replace_text
        """

        let raw = try runAppleScript(script)
        if raw == "NOT_PLAYING" || raw == "NO_ART" {
            return nil
        }

        let contentType = raw.hasSuffix(".png") ? "image/png" : "image/jpeg"
        let data = try Data(contentsOf: URL(fileURLWithPath: raw))
        guard !data.isEmpty else { return nil }
        return TrackArtwork(data: data, contentType: contentType)
    }

    private func runAppleScript(_ source: String) throws -> String {
        var error: NSDictionary?
        guard let script = NSAppleScript(source: source) else {
            throw AppleMusicError.scriptFailed("Failed to create AppleScript")
        }
        let output = script.executeAndReturnError(&error)
        if let error {
            let message = (error[NSAppleScript.errorMessage] as? String) ?? "AppleScript failed"
            throw AppleMusicError.scriptFailed(message)
        }
        return output.stringValue?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
    }

    private static func artworkCachePath() -> URL {
        FileManager.default.temporaryDirectory.appendingPathComponent("now-playing-artwork.jpg")
    }

    private static func parseSeconds(_ raw: String) -> UInt32? {
        let normalized = raw.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: ",", with: ".")
        guard let value = Double(normalized) else { return nil }
        return UInt32(max(0, value.rounded()))
    }

    static func parseTrackOutput(_ raw: String) throws -> NowPlayingTrack? {
        if raw == "NOT_PLAYING" {
            return nil
        }

        let parts = raw.split(separator: "||", omittingEmptySubsequences: false).map(String.init)
        guard parts.count == 6 else {
            throw AppleMusicError.unexpectedOutput(raw)
        }

        let isPlaying = parts[5].trimmingCharacters(in: .whitespaces) == "true" || parts[5] == "1"

        return NowPlayingTrack(
            trackName: parts[0].trimmingCharacters(in: .whitespaces),
            artistName: parts[1].trimmingCharacters(in: .whitespaces),
            albumName: parts[2].trimmingCharacters(in: .whitespaces),
            durationSeconds: parseSeconds(parts[3]),
            positionSeconds: parseSeconds(parts[4]),
            isPlaying: isPlaying
        )
    }
}
