import AppKit
import SwiftUI

@main
struct NowPlayingApp: App {
    @StateObject private var appState: AppState

    init() {
        let store: ConfigStore
        let config: AgentConfig
        var loadError: String?

        do {
            let loaded = try ConfigStore.loadOrCreate()
            store = loaded.0
            config = loaded.1
        } catch {
            loadError = error.localizedDescription
            config = AgentConfig.defaultTemplate()
            do {
                store = try ConfigStore.fallbackStore()
            } catch {
                fatalError("Failed to initialize config store: \(error.localizedDescription)")
            }
        }

        _appState = StateObject(wrappedValue: AppState(
            configStore: store,
            config: config,
            configLoadError: loadError
        ))
    }

    var body: some Scene {
        MenuBarExtra("Now Playing", image: "MenuBarIcon") {
            MenuBarMenuView()
                .environmentObject(appState)
                .onAppear {
                    appState.bootstrap()
                }
        }
        .menuBarExtraStyle(.menu)

        WindowGroup(id: "preferences") {
            PreferencesView()
                .environmentObject(appState)
        }
        .defaultSize(width: 460, height: 360)
    }
}

private struct MenuBarMenuView: View {
    @EnvironmentObject private var appState: AppState
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Text(appState.status.menuLabel)
            .disabled(true)

        Divider()

        Button("Preferences…") {
            NSApp.activate(ignoringOtherApps: true)
            openWindow(id: "preferences")
        }

        Button(loginItemTitle) {
            do {
                try appState.toggleLoginItem()
            } catch {
                appState.status = .error(error.localizedDescription)
            }
        }

        Divider()

        Button("Quit") {
            appState.quit()
        }
        .keyboardShortcut("q")
    }

    private var loginItemTitle: String {
        appState.loginAtLaunchEnabled ? "Disable at Login" : "Enable at Login"
    }
}
