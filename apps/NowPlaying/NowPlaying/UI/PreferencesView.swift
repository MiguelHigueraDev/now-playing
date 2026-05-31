import SwiftUI

struct PreferencesView: View {
    @EnvironmentObject private var appState: AppState

    @State private var apiBaseURL: String = ""
    @State private var authToken: String = ""
    @State private var pollIntervalSecs: Int = 3
    @State private var validationError: String?
    @State private var savedMessage: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            LiquidGlassModifiers.preferencesCard {
                VStack(alignment: .leading, spacing: 16) {
                    Text("Now Playing Preferences")
                        .font(.title2.weight(.semibold))

                    Text("Configure the API connection for the menu bar agent.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)

                    Form {
                        LabeledContent("API Base URL") {
                            TextField("http://localhost:3000", text: $apiBaseURL)
                                .textFieldStyle(.roundedBorder)
                        }

                        LabeledContent("Auth Token") {
                            SecureField("Bearer token", text: $authToken)
                                .textFieldStyle(.roundedBorder)
                        }

                        LabeledContent("Poll Interval (seconds)") {
                            HStack {
                                Stepper(value: $pollIntervalSecs, in: 2 ... 5) {
                                    Text("\(pollIntervalSecs)")
                                        .monospacedDigit()
                                        .frame(width: 24, alignment: .trailing)
                                }
                                Text("Must be between 2 and 5")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                    .formStyle(.grouped)

                    if let configLoadError = appState.configLoadError {
                        Text("Config load failed: \(configLoadError). Fix settings below and save.")
                            .font(.callout)
                            .foregroundStyle(.red)
                    }

                    if let validationError {
                        Text(validationError)
                            .font(.callout)
                            .foregroundStyle(.red)
                    }

                    if let savedMessage {
                        Text(savedMessage)
                            .font(.callout)
                            .foregroundStyle(.green)
                    }

                    HStack {
                        Spacer()
                        Button("Cancel") {
                            loadFromConfig()
                            validationError = nil
                            savedMessage = nil
                        }
                        Button("Save") {
                            save()
                        }
                        .nowPlayingPrimaryButtonStyle()
                        .keyboardShortcut(.defaultAction)
                    }
                }
            }
            .padding(24)
        }
        .frame(minWidth: 460, minHeight: 320)
        .onAppear {
            loadFromConfig()
        }
    }

    private func loadFromConfig() {
        apiBaseURL = appState.config.apiBaseURL
        authToken = appState.config.authToken
        pollIntervalSecs = Int(appState.config.pollIntervalSecs)
    }

    private func save() {
        validationError = nil
        savedMessage = nil

        let config = AgentConfig(
            apiBaseURL: apiBaseURL.trimmingCharacters(in: .whitespacesAndNewlines),
            authToken: authToken.trimmingCharacters(in: .whitespacesAndNewlines),
            pollIntervalSecs: pollIntervalSecs
        )

        do {
            try config.validateForSave()
            try appState.saveConfig(config)
            savedMessage = "Settings saved"
        } catch {
            validationError = error.localizedDescription
        }
    }
}
