//
//  ViTypeApp.swift
//  ViType
//
//  Created by Tran Dat on 26/12/25.
//

import SwiftUI

@main
struct ViTypeApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self)
    var appDelegate
    
    var body: some Scene {
        WindowGroup(id: "settings") {
            SettingsWindowContent()
        }
        .windowResizability(.contentSize)
        .defaultPosition(.center)
    }
}

/// Wrapper view that handles the openWindow action for showing settings
struct SettingsWindowContent: View {
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        ContentView()
            .onReceive(NotificationCenter.default.publisher(for: .showSettingsWindow)) { _ in
                // Re-open/focus the settings window
                openWindow(id: "settings")
            }
    }
}
