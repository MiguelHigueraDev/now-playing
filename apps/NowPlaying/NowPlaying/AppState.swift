import AppKit
import Combine
import Foundation
import SwiftUI

@MainActor
final class AppState: ObservableObject {
    let configStore: ConfigStore

    @Published private(set) var config: AgentConfig
    @Published var status: AgentStatus = .idle
    @Published var loginAtLaunchEnabled: Bool = false

    private var syncEngine: SyncEngine?
    private var didBootstrap = false

    init(configStore: ConfigStore, config: AgentConfig) {
        self.configStore = configStore
        self.config = config
    }

    func bootstrap() {
        guard !didBootstrap else { return }
        didBootstrap = true

        LogService.shared.configure(logDir: configStore.logDir)
        LogService.shared.info("Now Playing agent started")
        loginAtLaunchEnabled = LoginItemService.isEnabled
        startSyncEngine()
    }

    func saveConfig(_ newConfig: AgentConfig) throws {
        try configStore.save(newConfig)
        config = newConfig
        syncEngine?.updateConfig(newConfig)
    }

    func toggleLoginItem() throws {
        loginAtLaunchEnabled = try LoginItemService.toggle()
    }

    func quit() {
        syncEngine?.stop()
        NSApplication.shared.terminate(nil)
    }

    private func startSyncEngine() {
        let engine = SyncEngine(config: config)
        engine.onStatusChange = { [weak self] status in
            Task { @MainActor in
                self?.status = status
            }
        }
        engine.start()
        syncEngine = engine
    }
}
