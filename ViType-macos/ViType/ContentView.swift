//
//  ContentView.swift
//  ViType
//
//  Created by Tran Dat on 24/12/25.
//

import SwiftUI

enum SettingsTab: String, CaseIterable {
    case general
    case appExclusion

    func localizedName() -> String {
        switch self {
        case .general:
            return "General".localized()
        case .appExclusion:
            return "App Exclusion".localized()
        }
    }
}

struct ContentView: View {
    @StateObject private var localizationManager = LocalizationManager.shared
    @ObservedObject private var windowManager = WindowManager.shared
    @State private var selectedTab: SettingsTab = .general

    @AppStorage("autoFixTone") private var autoFixTone = true
    @AppStorage("freeTonePlacement") private var freeTonePlacement = false
    @AppStorage("inputMethod") private var inputMethod = 0
    @AppStorage("outputEncoding") private var outputEncoding = 0
    @AppStorage("tonePlacement") private var tonePlacement = 0
    @AppStorage(AppExclusion.isEnabledKey) private var appExclusionEnabled = true
    @AppStorage(AppExclusion.excludedBundleIDsKey) private var excludedBundleIDsText = ""
    @AppStorage(AppExclusion.viTypeEnabledKey) private var viTypeEnabled = true

    // Shortcut settings
    @AppStorage(AppExclusion.shortcutKeyKey) private var shortcutKey = "x"
    @AppStorage(AppExclusion.shortcutCommandKey) private var shortcutCommand = false
    @AppStorage(AppExclusion.shortcutOptionKey) private var shortcutOption = false
    @AppStorage(AppExclusion.shortcutControlKey) private var shortcutControl = true
    @AppStorage(AppExclusion.shortcutShiftKey) private var shortcutShift = false
    @AppStorage(AppExclusion.playSoundOnToggleKey) private var playSoundOnToggle = true

    // Lazy-init: only created once, not on every re-render
    @StateObject private var frontmostAppMonitor = FrontmostAppMonitor()

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Header - always visible
            VStack(alignment: .leading, spacing: 4) {
                Text("ViType")
                    .font(.title)
                    .fontWeight(.bold)

                Text("Vietnamese Input Method".localized())
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }

            // Segmented control for tabs
            Picker("", selection: $selectedTab) {
                ForEach(SettingsTab.allCases, id: \.self) { tab in
                    Text(tab.localizedName()).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .labelsHidden()

            Divider()

            // Tab content — use ScrollView to avoid layout jumps between tabs
            ScrollView {
                Group {
                    switch selectedTab {
                    case .general:
                        GeneralSettingsView(
                            viTypeEnabled: $viTypeEnabled,
                            shortcutKey: $shortcutKey,
                            shortcutCommand: $shortcutCommand,
                            shortcutOption: $shortcutOption,
                            shortcutControl: $shortcutControl,
                            shortcutShift: $shortcutShift,
                            inputMethod: $inputMethod,
                            tonePlacement: $tonePlacement,
                            autoFixTone: $autoFixTone,
                            freeTonePlacement: $freeTonePlacement,
                            outputEncoding: $outputEncoding,
                            playSoundOnToggle: $playSoundOnToggle
                        )

                    case .appExclusion:
                        AppExclusionView(
                            appExclusionEnabled: $appExclusionEnabled,
                            excludedBundleIDsText: $excludedBundleIDsText,
                            frontmostAppMonitor: frontmostAppMonitor
                        )
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding()
        .frame(width: 420)
        .onChange(of: windowManager.requestedTab) { _, newTab in
            if let tab = newTab {
                selectedTab = tab
                windowManager.requestedTab = nil
            }
        }
    }
}

// Custom text field that only accepts a single character (a-z, 0-9, []\;',./) or "space"
struct ShortcutKeyField: View {
    @Binding var key: String
    @State private var displayText: String = ""
    @ObservedObject private var localizationManager = LocalizationManager.shared
    private static let allowedShortcutCharacters: Set<Character> = Set("abcdefghijklmnopqrstuvwxyz0123456789[]\\;',./`")

    var body: some View {
        TextField("", text: $displayText)
            .textFieldStyle(.roundedBorder)
            .onAppear {
                displayText = key.lowercased() == "space" ? "Space".localized() : key.uppercased()
            }
            .onChange(of: displayText) { oldValue, newValue in
                processInput(oldValue: oldValue, input: newValue)
            }
            .onChange(of: localizationManager.currentLanguage) { _, _ in
                // Update display when language changes
                displayText = key.lowercased() == "space" ? "Space".localized() : key.uppercased()
            }
    }

    private func spaceCount(in value: String) -> Int {
        value.reduce(0) { $0 + ($1 == " " ? 1 : 0) }
    }

    private func processInput(oldValue: String, input: String) {
        let trimmed = input.trimmingCharacters(in: .whitespaces)
        let spaceLocalized = "Space".localized()

        // Get the current display value for reset/validation.
        let currentLocalizedDisplay = key.lowercased() == "space" ? spaceLocalized : key.uppercased()

        // Check for "space" typed out (in either language)
        if trimmed.lowercased() == "space" || trimmed == spaceLocalized {
            key = "space"
            displayText = spaceLocalized
            return
        }

        // If user types a space character.
        // Use old/new comparison so we don't mis-detect spaces that are part of the localized label (e.g., "Dấu cách").
        if input == " " || spaceCount(in: input) > spaceCount(in: oldValue) {
            key = "space"
            displayText = spaceLocalized
            return
        }

        // Take only the last character if multiple are entered
        guard let lastChar = trimmed.last else {
            // Empty input - reset to current key
            displayText = currentLocalizedDisplay
            return
        }

        let char = String(lastChar).lowercased()

        // Only accept allowed shortcut characters
        if let c = char.first, char.count == 1, Self.allowedShortcutCharacters.contains(c) {
            key = char
            displayText = char.uppercased()
        } else {
            // Invalid character - reset to current key
            displayText = currentLocalizedDisplay
        }
    }
}

#Preview {
    ContentView()
}
