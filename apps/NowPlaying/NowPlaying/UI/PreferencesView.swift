import SwiftUI

struct PreferencesView: View {
    @EnvironmentObject private var appState: AppState

    @State private var apiBaseURL: String = ""
    @State private var authToken: String = ""
    @State private var isAuthTokenVisible = false
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

                    VStack(alignment: .leading, spacing: 20) {
                        preferenceField(label: "API Base URL") {
                            TextField("", text: $apiBaseURL)
                                .textFieldStyle(.roundedBorder)
                                .frame(maxWidth: .infinity)

                            Text("Must be a valid HTTPS URL")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }

                        preferenceField(label: "Auth Token") {
                            HStack(spacing: 8) {
                                Group {
                                    if isAuthTokenVisible {
                                        TextField("", text: $authToken)
                                    } else {
                                        SecureField("", text: $authToken)
                                    }
                                }
                                .textFieldStyle(.roundedBorder)

                                Button {
                                    isAuthTokenVisible.toggle()
                                } label: {
                                    Image(systemName: isAuthTokenVisible ? "eye.slash" : "eye")
                                }
                                .buttonStyle(.borderless)
                                .help(isAuthTokenVisible ? "Hide token" : "Show token")
                            }
                        }

                        preferenceField(label: "Poll Interval") {
                            Picker("", selection: $pollIntervalSecs) {
                                ForEach(2 ... 5, id: \.self) { secs in
                                    Text("\(secs)s").tag(secs)
                                }
                            }
                            .pickerStyle(.segmented)
                            .labelsHidden()
                            .frame(maxWidth: .infinity)
                        }
                    }

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
        .frame(minWidth: 520, minHeight: 440)
        .onAppear {
            loadFromConfig()
        }
    }

    @ViewBuilder
    private func preferenceField<Content: View>(
        label: String,
        @ViewBuilder content: () -> Content
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(label)
                .font(.subheadline.weight(.medium))

            content()
        }
    }

    private func loadFromConfig() {
        apiBaseURL = appState.config.apiBaseURL
        authToken = appState.config.authToken
        pollIntervalSecs = Int(appState.config.pollIntervalSecs)
        isAuthTokenVisible = false
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
