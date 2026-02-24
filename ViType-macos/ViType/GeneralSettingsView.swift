//
//  GeneralSettingsView.swift
//  ViType
//
//  Created by Tran Dat on 30/12/24.
//

import SwiftUI

struct GeneralSettingsView: View {
    @ObservedObject private var localizationManager = LocalizationManager.shared
    
    @Binding var viTypeEnabled: Bool
    @Binding var shortcutKey: String
    @Binding var shortcutCommand: Bool
    @Binding var shortcutOption: Bool
    @Binding var shortcutControl: Bool
    @Binding var shortcutShift: Bool
    @Binding var inputMethod: Int
    @Binding var tonePlacement: Int
    @Binding var autoFixTone: Bool
    @Binding var freeTonePlacement: Bool
    @Binding var outputEncoding: Int
    @Binding var playSoundOnToggle: Bool

    // Local state for Launch at Login – keeps SwiftUI in sync with SMAppService.
    @State private var launchAtLoginEnabled = LaunchAtLoginManager.isOnForToggle

    private var shortcutDisplayString: String {
        var parts: [String] = []
        if shortcutControl { parts.append("^") }
        if shortcutOption { parts.append("\u{2325}") }
        if shortcutCommand { parts.append("\u{2318}") }
        if shortcutShift { parts.append("\u{21E7}") }

        let keyDisplay = shortcutKey.lowercased() == "space" ? "Space".localized() : shortcutKey.uppercased()
        parts.append(keyDisplay)

        return parts.joined()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            // Language Selector
            VStack(alignment: .leading, spacing: 4) {
                Picker("Language:".localized(), selection: $localizationManager.currentLanguage) {
                    ForEach(AppLanguage.allCases) { language in
                        Text(language.displayName).tag(language)
                    }
                }
                .pickerStyle(.menu)
            }
            
            Divider()
            
            // Enable ViType
            VStack(alignment: .leading, spacing: 4) {
                Toggle("Enable ViType".localized(), isOn: $viTypeEnabled)

                Text("Enable ViType Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            // Keyboard shortcut settings
            VStack(alignment: .leading, spacing: 8) {
                Text("Toggle Shortcut:".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)

                HStack(spacing: 12) {
                    Toggle("Control".localized(), isOn: $shortcutControl)
                        .toggleStyle(.checkbox)
                    Toggle("Option".localized(), isOn: $shortcutOption)
                        .toggleStyle(.checkbox)
                    Toggle("Command".localized(), isOn: $shortcutCommand)
                        .toggleStyle(.checkbox)
                    Toggle("Shift".localized(), isOn: $shortcutShift)
                        .toggleStyle(.checkbox)
                }
                .font(.caption)

                HStack(spacing: 8) {
                    Text("Key:".localized())
                        .font(.caption)
                        .foregroundColor(.secondary)

                    ShortcutKeyField(key: $shortcutKey)
                        .frame(width: 60)

                    Text(shortcutDisplayString)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .padding(.leading, 8)
                }
                
                // Play sound on toggle
                Toggle("Play Sound on Toggle".localized(), isOn: $playSoundOnToggle)
                    .padding(.top, 4)
                
                Text("Play Sound on Toggle Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(.leading, 20)

            Divider()

            // Input Method
            VStack(alignment: .leading, spacing: 4) {
                Picker("Input Method:".localized(), selection: $inputMethod) {
                    Text("Telex").tag(0)
                    Text("VNI").tag(1)
                }
                .pickerStyle(.menu)

                Text("Input Method Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Divider()

            // Tone Placement
            VStack(alignment: .leading, spacing: 4) {
                Picker("Tone Placement:".localized(), selection: $tonePlacement) {
                    Text("Orthographic".localized()).tag(0)
                    Text("Nucleus Only".localized()).tag(1)
                }
                .pickerStyle(.menu)

                Text("Tone Placement Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Divider()

            // Auto Fix Tone
            VStack(alignment: .leading, spacing: 4) {
                Toggle("Auto Fix Tone".localized(), isOn: $autoFixTone)

                Text("Auto Fix Tone Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Divider()

            // Free Tone Placement
            VStack(alignment: .leading, spacing: 4) {
                Toggle("Free Tone Placement".localized(), isOn: $freeTonePlacement)

                Text("Free Tone Placement Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Divider()

            // Character Encoding
            VStack(alignment: .leading, spacing: 4) {
                Picker("Character Encoding:".localized(), selection: $outputEncoding) {
                    Text("Unicode".localized()).tag(0)
                    Text("Composite Unicode".localized()).tag(1)
                }
                .pickerStyle(.menu)

                Text("Character Encoding Help".localized())
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Divider()

            // Start at Login
            VStack(alignment: .leading, spacing: 4) {
                Toggle("Start at Login".localized(), isOn: $launchAtLoginEnabled)
                    .onChange(of: launchAtLoginEnabled) { _, newValue in
                        do {
                            try LaunchAtLoginManager.setOn(newValue)
                        } catch {
                            // Revert on failure
                            launchAtLoginEnabled = LaunchAtLoginManager.isOnForToggle
                        }
                        // Sync back in case SMAppService ended up in a different state
                        launchAtLoginEnabled = LaunchAtLoginManager.isOnForToggle
                    }

                if LaunchAtLoginManager.state == .requiresApproval {
                    Text("Approval required in System Settings…".localized())
                        .font(.caption)
                        .foregroundColor(.orange)
                        .fixedSize(horizontal: false, vertical: true)
                } else {
                    Text("Start at Login Help".localized())
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
        }
        .onAppear {
            launchAtLoginEnabled = LaunchAtLoginManager.isOnForToggle
        }
    }
}

#Preview {
    GeneralSettingsView(
        viTypeEnabled: .constant(true),
        shortcutKey: .constant("x"),
        shortcutCommand: .constant(false),
        shortcutOption: .constant(false),
        shortcutControl: .constant(true),
        shortcutShift: .constant(false),
        inputMethod: .constant(0),
        tonePlacement: .constant(0),
        autoFixTone: .constant(true),
        freeTonePlacement: .constant(false),
        outputEncoding: .constant(0),
        playSoundOnToggle: .constant(true)
    )
    .padding()
    .frame(width: 400, height: 300)
}
