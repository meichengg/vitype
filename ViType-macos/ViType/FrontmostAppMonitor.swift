//
//  FrontmostAppMonitor.swift
//  ViType
//
//  Created by Tran Dat on 24/12/25.
//

import Cocoa
import Combine

final class FrontmostAppMonitor: ObservableObject {
    @Published private(set) var bundleIdentifier: String?
    @Published private(set) var lastNonViTypeBundleIdentifier: String?

    private let workspace: NSWorkspace
    private var activationObserver: NSObjectProtocol?
    private let viTypeBundleIdentifier: String?
    private static let lastNonViTypeDefaultsKey = "lastNonViTypeBundleIdentifier"

    init(workspace: NSWorkspace = .shared) {
        self.workspace = workspace
        viTypeBundleIdentifier = Bundle.main.bundleIdentifier?.lowercased()
        bundleIdentifier = workspace.frontmostApplication?.bundleIdentifier?.lowercased()

        lastNonViTypeBundleIdentifier = UserDefaults.standard
            .string(forKey: Self.lastNonViTypeDefaultsKey)?
            .lowercased()

        if let current = bundleIdentifier, current != viTypeBundleIdentifier {
            setLastNonViTypeBundleIdentifier(current)
        }

        activationObserver = workspace.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let self else { return }
            let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            let activeBundleID = app?.bundleIdentifier ?? self.workspace.frontmostApplication?.bundleIdentifier
            let normalizedActive = activeBundleID?.lowercased()

            bundleIdentifier = normalizedActive

            if let normalizedActive,
               normalizedActive != viTypeBundleIdentifier {
                setLastNonViTypeBundleIdentifier(normalizedActive)
            }
        }
    }

    deinit {
        if let activationObserver {
            workspace.notificationCenter.removeObserver(activationObserver)
        }
    }

    private func setLastNonViTypeBundleIdentifier(_ bundleID: String) {
        let normalized = bundleID.lowercased()
        lastNonViTypeBundleIdentifier = normalized
        UserDefaults.standard.set(normalized, forKey: Self.lastNonViTypeDefaultsKey)
    }
}


