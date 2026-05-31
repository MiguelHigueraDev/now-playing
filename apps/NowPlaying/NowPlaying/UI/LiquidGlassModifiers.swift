import SwiftUI

enum LiquidGlassModifiers {
    @ViewBuilder
    static func preferencesCard<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        if #available(macOS 26.0, *) {
            GlassEffectContainer {
                content()
                    .padding(20)
                    .glassEffect(.regular, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
            }
        } else {
            content()
                .padding(20)
                .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
        }
    }
}

extension View {
    @ViewBuilder
    func nowPlayingPrimaryButtonStyle() -> some View {
        if #available(macOS 26.0, *) {
            buttonStyle(.glass)
        } else {
            buttonStyle(.borderedProminent)
        }
    }
}
