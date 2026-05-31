import Foundation
import ServiceManagement

enum LoginItemError: LocalizedError {
    case registrationFailed(String)

    var errorDescription: String? {
        switch self {
        case .registrationFailed(let message):
            return message
        }
    }
}

enum LoginItemService {
    static var isEnabled: Bool {
        SMAppService.mainApp.status == .enabled
    }

    static func toggle() throws -> Bool {
        if isEnabled {
            try unregister()
            return false
        } else {
            try register()
            return true
        }
    }

    static func register() throws {
        do {
            try SMAppService.mainApp.register()
        } catch {
            if SMAppService.mainApp.status == .requiresApproval {
                SMAppService.openSystemSettingsLoginItems()
            }
            throw LoginItemError.registrationFailed(error.localizedDescription)
        }
    }

    static func unregister() throws {
        do {
            try SMAppService.mainApp.unregister()
        } catch {
            throw LoginItemError.registrationFailed(error.localizedDescription)
        }
    }
}
