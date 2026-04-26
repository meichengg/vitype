//
//  AppDelegate.swift
//  ViType
//
//  Created by Tran Dat on 24/12/25.
//

import Cocoa
import Carbon
import Sparkle

private final class SparkleUserDriverDelegate: NSObject, SPUStandardUserDriverDelegate {
    var beginForegroundPresentation: (() -> Void)?
    var endForegroundPresentation: (() -> Void)?

    func standardUserDriverWillHandleShowingUpdate(
        _ handleShowingUpdate: Bool,
        forUpdate update: SUAppcastItem,
        state: SPUUserUpdateState
    ) {
        guard handleShowingUpdate else { return }

        DispatchQueue.main.async { [weak self] in
            self?.beginForegroundPresentation?()
        }
    }

    func standardUserDriverWillFinishUpdateSession() {
        DispatchQueue.main.async { [weak self] in
            self?.endForegroundPresentation?()
        }
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private enum ForegroundPresentationReason: Hashable {
        case settings
        case updater
    }

    private let sparkleUserDriverDelegate: SparkleUserDriverDelegate
    // Sparkle updater controller for automatic updates
    let updaterController: SPUStandardUpdaterController
    private var foregroundPresentationReasons: Set<ForegroundPresentationReason> = []
    
    override init() {
        sparkleUserDriverDelegate = SparkleUserDriverDelegate()
        updaterController = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: nil,
            userDriverDelegate: sparkleUserDriverDelegate
        )
        super.init()

        sparkleUserDriverDelegate.beginForegroundPresentation = { [weak self] in
            self?.beginForegroundPresentation(for: .updater)
        }
        sparkleUserDriverDelegate.endForegroundPresentation = { [weak self] in
            self?.endForegroundPresentation(for: .updater)
        }
    }
    
    private var keyTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var transformer = KeyTransformer()
    private let injectedEventTag: Int64 = kViTypeEventMarker
    private let characterInjector = CharacterInjector()
    
    private var isInjectingReplacement: Bool = false
    private var pendingInjectedKeyDownCount: Int = 0
    private var queuedKeyDownEvents: [QueuedKeyDownEvent] = []
    private var flushQueuedEventsScheduled: Bool = false

    /// The event tap proxy from the current CGEventTap callback invocation.
    /// Only valid during the synchronous execution of the callback.
    /// Used for `tapPostEvent(proxy)` which posts events inline into the
    /// event stream (like GoxKey), skipping our own tap — much faster than
    /// `CGEvent.post(tap: .cghidEventTap)` which round-trips through HID.
    private var currentProxy: CGEventTapProxy?
    /// Tracks what is currently displayed on screen for the current word.
    /// Used to compute minimal diff (like GoxKey's get_diff_parts) so we only
    /// delete/retype the suffix that actually changed — eliminates flicker.
    private var displayBuffer: String = ""
    private var rawCompositionBuffer: String = ""
    private var cursorContextToneEditArmed: Bool = false
    private var cursorContextToneEditCache: CursorToneContext?
    private var localTextContext: [Character] = []
    private var localCursorOffset: Int = 0

    // Modifier-only shortcut tracking
    private var modifierShortcutArmed = false
    private var keyPressedDuringModifiers = false

    private var frontmostBundleID: String?
    private var excludedBundleIDs: Set<String> = []
    private var appActivationObserver: NSObjectProtocol?
    private var userDefaultsObserver: NSObjectProtocol?
    private var inputSourceObserver: NSObjectProtocol?
    private var sessionActiveObserver: NSObjectProtocol?
    private var wakeObserver: NSObjectProtocol?
    private var cachedInputSourceID: String?
    private var cachedInputSourceType: CFString?
    private var tapRestartPending: Bool = false
    private var lastRawTextInputResult: Bool = false
    private var rawTextInputCacheBundleID: String?
    private var rawTextInputCacheValue: Bool = false
    private var rawTextInputCacheTime: TimeInterval = 0
    private let rawTextInputCacheTTL: TimeInterval = 10
    
    // Unsupported input sources that should bypass Vietnamese transformation
    private let unsupportedInputSources: Set<String> = [
        "com.apple.inputmethod.SCIM.ITABC",           // Pinyin - Simplified
        "com.apple.inputmethod.TCIM.Pinyin",          // Pinyin - Traditional
        "com.apple.inputmethod.Korean",               // Korean
        "com.apple.inputmethod.Japanese",             // Japanese
        "com.apple.inputmethod.TCIM.Cangjie",         // Cangjie
        "com.apple.inputmethod.TCIM.Shuangpin",       // Shuangpin
        "com.apple.inputmethod.SCIM.Shuangpin",       // Shuangpin (Simplified)
    ]

    private var menuBarManager: MenuBarManager?
    private var settingsWindowObserver: NSObjectProtocol?

    // Cached shortcut settings for performance
    private var shortcutKey: String = ""
    private var shortcutKeyCode: Int64 = -1
    private var shortcutModifiers: CGEventFlags = []
    
    // Sound feedback
    private var toggleSound: NSSound?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // This is a menu-bar app by default; hide it from the Dock unless a window is explicitly shown.
        NSApp.setActivationPolicy(.accessory)

        // Register default values
        UserDefaults.standard.register(defaults: [
            "autoFixTone": true,
            "freeTonePlacement": false,
            "inputMethod": 0, // 0 = Telex, 1 = VNI
            "outputEncoding": 0,
            "tonePlacement": 0, // 0 = Orthographic, 1 = NucleusOnly
            AppExclusion.isEnabledKey: true,
            AppExclusion.excludedBundleIDsKey: "",
            AppExclusion.viTypeEnabledKey: true,
            AppExclusion.shortcutKeyKey: "x",
            AppExclusion.shortcutCommandKey: false,
            AppExclusion.shortcutOptionKey: false,
            AppExclusion.shortcutControlKey: true,
            AppExclusion.shortcutShiftKey: false,
            AppExclusion.playSoundOnToggleKey: true,
        ])

        refreshTransformerSettings()
        refreshFrontmostBundleID()
        refreshExcludedBundleIDs()
        refreshShortcutSettings()
        startAppExclusionObservers()
        startInputSourceObservers()
        startSessionObservers()
        startKeyTap()

        // Initialize menu bar with Sparkle updater
        menuBarManager = MenuBarManager(updaterController: updaterController)
        
        // Listen for settings window requests
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(showSettingsWindow),
            name: .showSettingsWindow,
            object: nil
        )
        
        // Observe the initial settings window for close events (to hide from Dock when closed)
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 500_000_000) // 0.5 seconds for SwiftUI to create window
            for window in NSApp.windows {
                if window is NSPanel { continue }
                if window.className.contains("StatusBar") { continue }
                if window.level == .statusBar { continue }
                
                if window.contentView != nil && window.isVisible {
                    self.observeWindowClose(window)
                    break
                }
            }
        }
    }
    
    @objc private func showSettingsWindow(_ notification: Notification) {
        // Extract target tab from notification userInfo if provided
        let targetTab = notification.userInfo?[SettingsNotificationKey.tab] as? SettingsTab
        openSettingsWindow(tab: targetTab)
    }

    private func openSettingsWindow(tab: SettingsTab? = nil) {
        Task { @MainActor in
            beginForegroundPresentation(for: .settings)

            let settingsWindow = await WindowManager.shared.openSettings(tab: tab)
            self.observeWindowClose(settingsWindow)
        }
    }
    
    private func observeWindowClose(_ window: NSWindow) {
        // Remove any existing observer
        if let observer = settingsWindowObserver {
            NotificationCenter.default.removeObserver(observer)
        }
        
        // Observe when this window closes
        settingsWindowObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.willCloseNotification,
            object: window,
            queue: .main
        ) { [weak self] _ in
            self?.endForegroundPresentation(for: .settings)
            self?.settingsWindowObserver = nil
        }
    }

    private func beginForegroundPresentation(for reason: ForegroundPresentationReason) {
        let inserted = foregroundPresentationReasons.insert(reason).inserted
        guard inserted || NSApp.activationPolicy() != .regular else {
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        NSApp.setActivationPolicy(.regular)
        NSApp.unhide(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    private func endForegroundPresentation(for reason: ForegroundPresentationReason) {
        foregroundPresentationReasons.remove(reason)

        guard foregroundPresentationReasons.isEmpty else { return }
        NSApp.setActivationPolicy(.accessory)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        // Keep the app running (menu bar icon) even when the settings window is closed
        return false
    }
    
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        // Called when user clicks the app icon while it's already running (e.g., from Finder, Dock, Spotlight)
        // Show the settings window and Dock icon for consistent behavior
        openSettingsWindow(tab: nil)
        return false // We handled it
    }

    deinit {
        if let appActivationObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(appActivationObserver)
        }
        if let userDefaultsObserver {
            NotificationCenter.default.removeObserver(userDefaultsObserver)
        }
        if let inputSourceObserver {
            DistributedNotificationCenter.default().removeObserver(inputSourceObserver)
        }
        if let sessionActiveObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(sessionActiveObserver)
        }
        if let wakeObserver {
            NSWorkspace.shared.notificationCenter.removeObserver(wakeObserver)
        }
        if let settingsWindowObserver {
            NotificationCenter.default.removeObserver(settingsWindowObserver)
        }
        NotificationCenter.default.removeObserver(self)
    }
    private func startKeyTap() {
        let mask = CGEventMask(1 << CGEventType.keyDown.rawValue)
                     | CGEventMask(1 << CGEventType.flagsChanged.rawValue)
                     | CGEventMask(1 << CGEventType.leftMouseDown.rawValue)
                     | CGEventMask(1 << CGEventType.rightMouseDown.rawValue)
                     | CGEventMask(1 << CGEventType.otherMouseDown.rawValue)

        keyTap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: mask,
            callback: { proxy, type, event, refcon in
                let delegate = Unmanaged<AppDelegate>
                    .fromOpaque(refcon!)
                    .takeUnretainedValue()
                if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
                    delegate.handleTapDisabled()
                    return Unmanaged.passUnretained(event)
                }
                if type == .leftMouseDown || type == .rightMouseDown || type == .otherMouseDown {
                    delegate.handleMouseDown()
                    return Unmanaged.passUnretained(event)
                }
                // Store proxy for inline event posting (like GoxKey's CGEventTapPostEvent).
                // Events posted via proxy skip our tap and go directly to the app.
                delegate.currentProxy = proxy
                defer { delegate.currentProxy = nil }
                if type == .flagsChanged {
                    let suppress = delegate.handleFlagsChangedEvent(event: event)
                    return suppress ? nil : Unmanaged.passUnretained(event)
                }
                let suppress = delegate.handle(event: event)
                return suppress ? nil : Unmanaged.passUnretained(event)
            },
            userInfo: UnsafeMutableRawPointer(
                Unmanaged.passUnretained(self).toOpaque()
            )
        )

        guard let tap = keyTap else {
            print("No Accessibility permission.")
            return
        }

        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
        CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
        CGEvent.tapEnable(tap: tap, enable: true)
    }

    private func stopKeyTap() {
        if let runLoopSource {
            CFRunLoopSourceInvalidate(runLoopSource)
            CFRunLoopRemoveSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
            self.runLoopSource = nil
        }

        if let keyTap {
            CFMachPortInvalidate(keyTap)
            self.keyTap = nil
        }
    }

    private func restartKeyTap() {
        stopKeyTap()
        resetKeyTapState()
        startKeyTap()
    }

    private func resetKeyTapState() {
        resetInputState()
    }

    private func resetInputState() {
        isInjectingReplacement = false
        pendingInjectedKeyDownCount = 0
        queuedKeyDownEvents.removeAll(keepingCapacity: true)
        flushQueuedEventsScheduled = false
        displayBuffer = ""
        rawCompositionBuffer = ""
        cursorContextToneEditArmed = false
        cursorContextToneEditCache = nil
        localTextContext.removeAll(keepingCapacity: true)
        localCursorOffset = 0
        lastRawTextInputResult = false
        transformer.reset()
    }

    private func requestKeyTapRestart() {
        guard !tapRestartPending else { return }
        tapRestartPending = true
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.tapRestartPending = false
            self.restartKeyTap()
        }
    }

    private func handleTapDisabled() {
        requestKeyTapRestart()
    }

    func handleMouseDown() {
        resetInputState()
    }

    private func startSessionObservers() {
        sessionActiveObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.sessionDidBecomeActiveNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.requestKeyTapRestart()
        }

        wakeObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didWakeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.requestKeyTapRestart()
        }
    }

    private func startAppExclusionObservers() {
        appActivationObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let self else { return }
            let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            frontmostBundleID = app?.bundleIdentifier ?? NSWorkspace.shared.frontmostApplication?.bundleIdentifier
            invalidateRawTextInputCache()
            resetInputState()
        }

        userDefaultsObserver = NotificationCenter.default.addObserver(
            forName: UserDefaults.didChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            guard let self else { return }
            let wasBypassing = shouldBypassVietnameseInput()
            refreshExcludedBundleIDs()
            refreshShortcutSettings()
            refreshTransformerSettings()
            let isBypassing = shouldBypassVietnameseInput()
            if wasBypassing != isBypassing {
                transformer.reset()
            }
        }
    }

    private func startInputSourceObservers() {
        refreshCachedInputSourceInfo(resetTransformerIfNeeded: false)

        inputSourceObserver = DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name(kTISNotifySelectedKeyboardInputSourceChanged as String),
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.refreshCachedInputSourceInfo(resetTransformerIfNeeded: true)
        }
    }

    private func refreshCachedInputSourceInfo(resetTransformerIfNeeded: Bool) {
        let wasUnsupported = isInputSourceUnsupported(
            inputSourceID: cachedInputSourceID,
            inputSourceType: cachedInputSourceType
        )

        let info = getCurrentInputSourceInfo()
        cachedInputSourceID = info?.id
        cachedInputSourceType = info?.type

        let isUnsupported = isInputSourceUnsupported(
            inputSourceID: cachedInputSourceID,
            inputSourceType: cachedInputSourceType
        )

        if resetTransformerIfNeeded && wasUnsupported != isUnsupported {
            transformer.reset()
        }
    }

    private func refreshTransformerSettings() {
        transformer.autoFixTone = UserDefaults.standard.bool(forKey: "autoFixTone")
        transformer.freeTonePlacement = UserDefaults.standard.bool(forKey: "freeTonePlacement")
        let methodValue = UserDefaults.standard.integer(forKey: "inputMethod")
        transformer.inputMethod = InputMethod(rawValue: Int32(methodValue)) ?? .telex

        let encodingValue = UserDefaults.standard.integer(forKey: "outputEncoding")
        transformer.outputEncoding = OutputEncoding(rawValue: Int32(encodingValue)) ?? .unicode

        let tonePlacementValue = UserDefaults.standard.integer(forKey: "tonePlacement")
        transformer.tonePlacement = TonePlacement(rawValue: Int32(tonePlacementValue)) ?? .orthographic
    }

    private func refreshFrontmostBundleID() {
        frontmostBundleID = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
    }

    private func refreshExcludedBundleIDs() {
        let text = UserDefaults.standard.string(forKey: AppExclusion.excludedBundleIDsKey) ?? ""
        excludedBundleIDs = AppExclusion.parseBundleIDList(text)
    }

    private func refreshShortcutSettings() {
        shortcutKey = UserDefaults.standard.string(forKey: AppExclusion.shortcutKeyKey) ?? "x"
        shortcutKeyCode = Self.keyCodeForCharacter(shortcutKey)

        var modifiers: CGEventFlags = []
        if UserDefaults.standard.bool(forKey: AppExclusion.shortcutCommandKey) {
            modifiers.insert(.maskCommand)
        }
        if UserDefaults.standard.bool(forKey: AppExclusion.shortcutOptionKey) {
            modifiers.insert(.maskAlternate)
        }
        if UserDefaults.standard.bool(forKey: AppExclusion.shortcutControlKey) {
            modifiers.insert(.maskControl)
        }
        if UserDefaults.standard.bool(forKey: AppExclusion.shortcutShiftKey) {
            modifiers.insert(.maskShift)
        }
        shortcutModifiers = modifiers
    }

    private func shouldBypassVietnameseInput() -> Bool {
        // Check global enable toggle first
        guard UserDefaults.standard.bool(forKey: AppExclusion.viTypeEnabledKey) else { return true }
        
        // Check if current input source is unsupported (e.g., Pinyin, Korean, Japanese)
        if isUnsupportedInputSource() {
            return true
        }

        // Then check app exclusion
        guard UserDefaults.standard.bool(forKey: AppExclusion.isEnabledKey) else { return false }
        guard let frontmostBundleID else { return false }
        let normalizedFrontmost = AppExclusion.normalizeBundleID(frontmostBundleID)
        if let viTypeBundleID = Bundle.main.bundleIdentifier.map({ AppExclusion.normalizeBundleID($0) }),
           normalizedFrontmost == viTypeBundleID {
            return true
        }
        return excludedBundleIDs.contains(normalizedFrontmost)
    }

    private func toggleViType() {
        let currentState = UserDefaults.standard.bool(forKey: AppExclusion.viTypeEnabledKey)
        let newState = !currentState
        UserDefaults.standard.set(newState, forKey: AppExclusion.viTypeEnabledKey)
        transformer.reset()
        displayBuffer = ""
        
        // Play sound feedback if enabled
        if UserDefaults.standard.bool(forKey: AppExclusion.playSoundOnToggleKey) {
            // Stop any currently playing sound to ensure new sound plays immediately
            toggleSound?.stop()
            // Different sounds for enable vs disable
            // "Tink" for enable (short, high), "Pop" for disable (short, low)
            let soundName = newState ? "Tink" : "Pop"
            toggleSound = NSSound(named: NSSound.Name(soundName))
            toggleSound?.play()
        }
    }
    
    private func isUnsupportedInputSource() -> Bool {
        if cachedInputSourceID == nil && cachedInputSourceType == nil {
            let info = getCurrentInputSourceInfo()
            return isInputSourceUnsupported(inputSourceID: info?.id, inputSourceType: info?.type)
        }
        return isInputSourceUnsupported(inputSourceID: cachedInputSourceID, inputSourceType: cachedInputSourceType)
    }
    
    private func isInputSourceUnsupported(inputSourceID: String?, inputSourceType: CFString?) -> Bool {
        if inputSourceType == kTISTypeKeyboardInputMethodWithoutModes ||
            inputSourceType == kTISTypeKeyboardInputMethodModeEnabled ||
            inputSourceType == kTISTypeKeyboardInputMode {
            return true
        }
        guard let inputSourceID else { return false }
        return unsupportedInputSources.contains(inputSourceID)
    }

    private struct InputSourceInfo {
        let id: String
        let type: CFString?
    }

    private func getCurrentInputSourceInfo() -> InputSourceInfo? {
        guard let currentSource = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue() else { return nil }

        guard let sourceID = TISGetInputSourceProperty(currentSource, kTISPropertyInputSourceID) else {
            return nil
        }
        let id = Unmanaged<CFString>.fromOpaque(sourceID).takeUnretainedValue() as String

        var type: CFString?
        if let sourceType = TISGetInputSourceProperty(currentSource, kTISPropertyInputSourceType) {
            type = Unmanaged<CFString>.fromOpaque(sourceType).takeUnretainedValue()
        }

        return InputSourceInfo(id: id, type: type)
    }
}

// MARK: - Diff-based minimal edit (ported from GoxKey's get_diff_parts)
extension AppDelegate {
    /// Compute the minimal edit to transform `old` (on screen) into `new` (desired).
    /// Returns (backspaceCount, suffix) where backspaceCount is chars to delete
    /// and suffix is the new text to type after those backspaces.
    private static func diffParts(old: String, new: String) -> (Int, String) {
        let oldChars = Array(old)
        let newChars = Array(new)

        var common = 0
        let minLen = min(oldChars.count, newChars.count)
        while common < minLen && oldChars[common] == newChars[common] {
            common += 1
        }

        let backspaceCount = oldChars.count - common
        let prefixLength = common
        let suffix = String(newChars[prefixLength...])
        return (backspaceCount, suffix)
    }

    private static func isWordBoundary(_ string: String) -> Bool {
        guard !string.isEmpty else { return false }
        return string.unicodeScalars.allSatisfy {
            CharacterSet.whitespacesAndNewlines.contains($0) ||
            CharacterSet.punctuationCharacters.contains($0)
        }
    }

    private static func splitTrailingBoundary(_ text: String) -> (String, String) {
        var body = Array(text)
        var boundary: [Character] = []
        while let last = body.last, Self.isWordBoundary(String(last)) {
            boundary.insert(body.removeLast(), at: 0)
        }
        return (String(body), String(boundary))
    }

    private static func isCursorContextTrailingBoundary(_ string: String) -> Bool {
        guard isWordBoundary(string) else { return false }
        return !string.unicodeScalars.contains { CharacterSet.newlines.contains($0) }
    }

    private static func isToneEditKey(_ string: String, inputMethod: InputMethod) -> Bool {
        guard string.count == 1, let ch = string.first else { return false }
        switch inputMethod {
        case .telex:
            return ["s", "f", "r", "x", "j", "z"].contains(String(ch).lowercased())
        case .vni:
            return ("0"..."5").contains(String(ch))
        }
    }
}

extension AppDelegate {
    private struct QueuedKeyDownEvent {
        let keyCode: Int64
        let flags: CGEventFlags
        let unicodeString: String?
    }
    
    // Key codes for navigation and special keys
    private static let backspaceKey: Int64 = 51
    private static let forwardDeleteKey: Int64 = 117
    private static let escapeKey: Int64 = 53
    private static let wKey: Int64 = 13
    private static let navigationKeys: Set<Int64> = [
        48,                   // Tab (covers Tab/Shift+Tab focus traversal)
        123, 124, 125, 126,  // Arrow keys: left, right, down, up
        115, 119,            // Home, End
        116, 121             // Page Up, Page Down
    ]
    private static let rawTextInputBundleIDs: Set<String> = [
        "dev.warp.warp-stable"
    ]
    private static let directReplacementBundleIDs: Set<String> = [
        "com.exafunction.windsurf",
        "net.kovidgoyal.kitty",
        "io.alacritty",
        "com.github.wez.wezterm",
        "com.mitchellh.ghostty",
        "com.apple.terminal",
        "com.googlecode.iterm2",
        "com.microsoft.vscode",
        "com.microsoft.vscodeinsiders",
        "com.visualstudio.code.oss"
    ]
    private static let directReplacementBundleIDFragments: [String] = [
        "windsurf",
        "cursor",
        "vscode",
        "codeium",
        "electron"
    ]

    // Key code mapping for a-z, 0-9, punctuation, and space
    private static let keyCodeMap: [String: Int64] = [
        "a": 0, "b": 11, "c": 8, "d": 2, "e": 14, "f": 3, "g": 5, "h": 4,
        "i": 34, "j": 38, "k": 40, "l": 37, "m": 46, "n": 45, "o": 31, "p": 35,
        "q": 12, "r": 15, "s": 1, "t": 17, "u": 32, "v": 9, "w": 13, "x": 7,
        "y": 16, "z": 6,
        "0": 29, "1": 18, "2": 19, "3": 20, "4": 21, "5": 23, "6": 22, "7": 26, "8": 28, "9": 25,
        "[": 33, "]": 30, "\\": 42, ";": 41, "'": 39, ",": 43, ".": 47, "/": 44, "`": 50,
        "space": 49
    ]

    static func keyCodeForCharacter(_ char: String) -> Int64 {
        keyCodeMap[char.lowercased()] ?? -1
    }

    private func isToggleShortcut(keyCode: Int64, flags: CGEventFlags) -> Bool {
        guard shortcutModifiers.rawValue != 0 else { return false }
        // Modifier-only shortcut — handled in handleFlagsChangedEvent instead
        if shortcutKey.isEmpty { return false }
        guard keyCode == shortcutKeyCode else { return false }

        let relevantModifiers: CGEventFlags = [.maskCommand, .maskAlternate, .maskControl, .maskShift]
        let pressedModifiers = flags.intersection(relevantModifiers)
        return pressedModifiers == shortcutModifiers
    }

    private static func isControlW(keyCode: Int64, flags: CGEventFlags) -> Bool {
        keyCode == wKey &&
            flags.contains(.maskControl) &&
            !flags.contains(.maskCommand) &&
            !flags.contains(.maskAlternate)
    }

    private static func isOptionBackspace(keyCode: Int64, flags: CGEventFlags) -> Bool {
        keyCode == backspaceKey &&
            flags.contains(.maskAlternate) &&
            !flags.contains(.maskCommand) &&
            !flags.contains(.maskControl)
    }

    /// Handles flagsChanged events for modifier-only shortcuts (e.g. Ctrl+Shift).
    /// Toggles on release if no key was pressed while the exact modifiers were held.
    func handleFlagsChangedEvent(event: CGEvent) -> Bool {
        // Pass through during recording
        if AppExclusion.isRecordingShortcut { return false }
        // Only for modifier-only shortcuts (shortcutKey is empty)
        guard shortcutKey.isEmpty, shortcutModifiers.rawValue != 0 else { return false }

        let relevantModifiers: CGEventFlags = [.maskCommand, .maskAlternate, .maskControl, .maskShift]
        let currentModifiers = event.flags.intersection(relevantModifiers)

        if currentModifiers == shortcutModifiers {
            // Exact match — arm the toggle
            modifierShortcutArmed = true
            keyPressedDuringModifiers = false
        } else if currentModifiers.isEmpty && modifierShortcutArmed {
            // All modifiers released — toggle if no key was pressed while held
            if !keyPressedDuringModifiers {
                toggleViType()
            }
            modifierShortcutArmed = false
            keyPressedDuringModifiers = false
        }
        // If modifiers partially released (not empty, not matching), stay armed —
        // user is releasing keys one at a time, wait until all are released.

        return false
    }
    
    /// Returns `true` if the event should be suppressed (not passed to the application).
    func handle(event: CGEvent) -> Bool {
        // Skip injected events
        if event.getIntegerValueField(.eventSourceUserData) == injectedEventTag {
            noteInjectedKeyDown()
            return false
        }
        
        if isInjectingReplacement {
            enqueueKeyDownEvent(event)
            return true
        }

        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
        let flags = event.flags

        // Track key press for modifier-only shortcut detection
        if modifierShortcutArmed { keyPressedDuringModifiers = true }

        // Check for toggle shortcut (skip during shortcut recording)
        if !AppExclusion.isRecordingShortcut && isToggleShortcut(keyCode: keyCode, flags: flags) {
            toggleViType()
            return true  // Suppress the event
        }

        // App exclusion: bypass Vietnamese transformation for excluded apps.
        if shouldBypassVietnameseInput() {
            transformer.reset()
            displayBuffer = ""
            rawCompositionBuffer = ""
            clearCursorToneEditContext()
            return false
        }
        
        // Check for action modifiers (Cmd, Ctrl, Option) - these typically perform actions
        let hasActionModifier = flags.contains(.maskCommand) ||
                                flags.contains(.maskControl) ||
                                flags.contains(.maskAlternate)

        if Self.isControlW(keyCode: keyCode, flags: flags) {
            transformer.deleteCurrentWord()
            deleteLocalWordBeforeCursor()
            displayBuffer = ""
            rawCompositionBuffer = ""
            armCursorToneEditContext()
            lastRawTextInputResult = false
            return false
        }

        if Self.isOptionBackspace(keyCode: keyCode, flags: flags) {
            transformer.deleteCurrentWord()
            deleteLocalWordBeforeCursor()
            displayBuffer = ""
            rawCompositionBuffer = ""
            armCursorToneEditContext()
            lastRawTextInputResult = false
            return false
        }
        
        // Backspace without modifiers - remove one char from buffer
        if keyCode == Self.backspaceKey && !hasActionModifier {
            let shouldKeepCursorToneContext = cursorContextToneEditArmed
            transformer.deleteLastCharacter()
            deleteLocalCharacterBeforeCursor()
            displayBuffer = transformer.currentText()
            if shouldKeepCursorToneContext {
                armCursorToneEditContext()
            } else {
                clearCursorToneEditContext()
            }
            if !rawCompositionBuffer.isEmpty {
                rawCompositionBuffer.removeLast()
            }
            return false
        }
        
        let isNavigationKey = Self.navigationKeys.contains(keyCode)
        if keyCode == Self.forwardDeleteKey ||
           keyCode == Self.escapeKey ||
           isNavigationKey ||
           hasActionModifier {
            transformer.reset()
            displayBuffer = ""
            rawCompositionBuffer = ""
            if isNavigationKey {
                updateLocalCursorForNavigation(keyCode: keyCode, flags: flags)
                armCursorToneEditContext()
            } else {
                clearCursorToneEditContext()
            }
            lastRawTextInputResult = false
            return false
        }

        guard let s = event.keyboardGetUnicodeString() else { return false }

        // Update settings from UserDefaults
        refreshTransformerSettings()

        if tryApplyCursorContextTone(input: s) {
            return true
        }

        if Self.isWordBoundary(s) {
            _ = transformer.process(input: s)
            insertLocalText(s)
            rawCompositionBuffer = ""
            displayBuffer = ""
            clearCursorToneEditContext()
            return false
        }
        
        if let action = transformer.process(input: s) {
            let rawMode = shouldUseRawTextInputReplacement()
            let extraDeleteCount = rawMode ? 0 : (shouldWipeGhostSuggestion() ? 1 : 0)

            if displayBuffer.isEmpty && action.deleteCount > 0 {
                let adjusted = adjustedReplacementForCursorContext(deleteCount: action.deleteCount, text: action.text)
                replace(last: adjusted.0, with: adjusted.1, extraDeleteCount: 0, rawModeOverride: rawMode)
                replaceLocalTextBeforeCursor(deleteCount: adjusted.0, with: adjusted.1)
                displayBuffer = adjusted.1
                return true
            }

            // Reconstruct what the new on-screen text should be:
            // Engine says "delete `action.deleteCount` chars from the end and replace with `action.text`".
            // Apply that to our displayBuffer to get the desired new screen state.
            let newDisplay: String
            if action.deleteCount >= displayBuffer.count {
                // Engine wants to replace everything (including the just-typed char that hasn't appeared yet)
                newDisplay = action.text
            } else {
                // Keep prefix, replace suffix
                let keepCount = displayBuffer.count - action.deleteCount
                let prefix = String(displayBuffer.prefix(keepCount))
                newDisplay = prefix + action.text
            }

            if rawMode {
                let (diffBS, diffSuffix) = Self.diffParts(old: displayBuffer, new: newDisplay)
                replace(last: diffBS, with: diffSuffix, extraDeleteCount: 0, rawModeOverride: true)
                replaceLocalTextBeforeCursor(deleteCount: diffBS, with: diffSuffix)
                displayBuffer = newDisplay
                clearCursorToneEditContext()
                return true
            } else {
                // Compute minimal diff between what's on screen and desired output.
                // The just-typed char `s` has NOT appeared on screen yet (we'll suppress this event),
                // so displayBuffer reflects the true on-screen state.
                let (diffBS, diffSuffix) = Self.diffParts(old: displayBuffer, new: newDisplay)
                let totalBS = extraDeleteCount + diffBS
                replace(last: totalBS, with: diffSuffix, extraDeleteCount: 0, rawModeOverride: rawMode)
                replaceLocalTextBeforeCursor(deleteCount: diffBS, with: diffSuffix)
            }
            displayBuffer = newDisplay
            clearCursorToneEditContext()
            return true
        }

        // No transformation — the char passes through to the app normally.
        // Track it in displayBuffer since it will appear on screen.
        if Self.isWordBoundary(s) {
            displayBuffer = ""
        } else {
            displayBuffer.append(s)
        }
        insertLocalText(s)
        clearCursorToneEditContext()
        return false
    }


    private func replace(last count: Int, with text: String, extraDeleteCount: Int, rawModeOverride: Bool? = nil) {
        let totalBS = extraDeleteCount + count
        let rawMode = rawModeOverride ?? shouldUseRawTextInputReplacement()

        // Fast sync path (like GoxKey): use event tap proxy for ALL apps.
        // Events posted via proxy skip our tap → zero round-trip, no flicker.
        if currentProxy != nil {
            if rawMode {
                sendRawReplacementSync(backspaceCount: totalBS, text: text)
                flushQueuedKeyDownEvents()
                return
            }

            let needsSentinel = totalBS > 1 &&
                totalBS >= displayBuffer.count &&
                !text.isEmpty &&
                !Self.prefersDirectReplacement(for: currentFrontmostBundleID())
            if needsSentinel {
                sendBackspacesSync(count: totalBS - 1)
                let firstChar = String(text.prefix(1))
                let rest = String(text.dropFirst())
                sendTextSync(firstChar)
                sendArrowSync(left: true)
                sendBackspacesSync(count: 1)
                sendArrowSync(left: false)
                if !rest.isEmpty {
                    sendTextSync(rest)
                }
            } else {
                if totalBS > 0 {
                    sendBackspacesSync(count: totalBS)
                }
                if !text.isEmpty {
                    sendTextSync(text)
                }
            }
            flushQueuedKeyDownEvents()
            return
        }

        if rawMode {
            sendRawReplacementPost(backspaceCount: totalBS, text: text)
            flushQueuedKeyDownEvents()
            return
        }

        // Async fallback when no proxy (should rarely happen)
        beginReplacementInjection()

        DispatchQueue.global(qos: .userInteractive).async { [weak self] in
            guard let self = self else { return }
            self.characterInjector.injectSync(
                backspaceCount: totalBS,
                text: text,
                proxy: nil
            )
            DispatchQueue.main.async {
                self.finishReplacementInjection()
            }
        }
    }

    private func adjustedReplacementForCursorContext(deleteCount: Int, text: String) -> (Int, String) {
        guard deleteCount > 0, !text.isEmpty else {
            return (deleteCount, text)
        }

        let split = Self.splitTrailingBoundary(text)
        guard !split.1.isEmpty else {
            return (deleteCount, text)
        }
        guard let textBeforeCursor = readTextBeforeCursor() else {
            return (deleteCount, text)
        }

        var presentBoundary = split.1
        while !presentBoundary.isEmpty && !textBeforeCursor.hasSuffix(presentBoundary) {
            presentBoundary.removeFirst()
        }

        let wordDeleteCount = max(0, deleteCount - split.1.count)
        return (wordDeleteCount + presentBoundary.count, split.0 + presentBoundary)
    }

    private func tryApplyCursorContextTone(input: String) -> Bool {
        guard cursorContextToneEditArmed else { return false }
        guard Self.isToneEditKey(input, inputMethod: transformer.inputMethod) else {
            clearCursorToneEditContext()
            return false
        }
        let context: CursorToneContext
        if let localContext = cursorContextToneEditCache ?? previousWordBeforeLocalCursor() {
            context = localContext
        } else if let accessibilityContext = previousWordBeforeCursor() {
            context = accessibilityContext
        } else {
            clearCursorToneEditContext()
            return false
        }
        guard let action = transformer.applyTone(to: context.word, input: input) else {
            clearCursorToneEditContext()
            return false
        }

        if action.text == context.word {
            cursorContextToneEditArmed = true
            cursorContextToneEditCache = context
            return true
        }

        let oldChars = Array(context.word)
        let newChars = Array(action.text)
        var commonPrefixCount = 0
        let comparableCount = min(oldChars.count, newChars.count)
        while commonPrefixCount < comparableCount && oldChars[commonPrefixCount] == newChars[commonPrefixCount] {
            commonPrefixCount += 1
        }

        let changedSuffix = commonPrefixCount < newChars.count ? String(newChars[commonPrefixCount...]) : ""
        let replacement = changedSuffix + context.trailingBoundary
        let deleteCount = (oldChars.count - commonPrefixCount) + context.trailingBoundary.count
        let rawMode = shouldUseRawTextInputReplacement(allowAXLookup: false)
        replaceCursorContextTone(last: deleteCount, with: replacement, rawMode: rawMode)
        replaceLocalTextBeforeCursor(deleteCount: deleteCount, with: replacement)
        displayBuffer = ""
        rawCompositionBuffer = ""
        cursorContextToneEditArmed = true
        cursorContextToneEditCache = previousWordBeforeLocalCursor() ?? CursorToneContext(word: action.text, trailingBoundary: context.trailingBoundary)
        return true
    }

    private func previousWordBeforeCursor() -> CursorToneContext? {
        guard let textBeforeCursor = readTextBeforeCursor() else { return nil }

        var remaining = Array(textBeforeCursor)
        var trailingBoundary: [Character] = []
        while let last = remaining.last, Self.isCursorContextTrailingBoundary(String(last)) {
            trailingBoundary.insert(remaining.removeLast(), at: 0)
        }

        var word: [Character] = []
        while let last = remaining.last, !Self.isWordBoundary(String(last)) {
            word.insert(remaining.removeLast(), at: 0)
        }

        guard !word.isEmpty else { return nil }
        return CursorToneContext(word: String(word), trailingBoundary: String(trailingBoundary))
    }

    private func clearCursorToneEditContext() {
        cursorContextToneEditArmed = false
        cursorContextToneEditCache = nil
    }

    private func armCursorToneEditContext() {
        cursorContextToneEditArmed = true
        cursorContextToneEditCache = previousWordBeforeLocalCursor()
    }

    private func insertLocalText(_ text: String) {
        replaceLocalTextBeforeCursor(deleteCount: 0, with: text)
    }

    private func replaceLocalTextBeforeCursor(deleteCount: Int, with text: String) {
        localCursorOffset = min(max(localCursorOffset, 0), localTextContext.count)
        let deleteCount = min(max(deleteCount, 0), localCursorOffset)
        if deleteCount > 0 {
            let start = localCursorOffset - deleteCount
            localTextContext.removeSubrange(start..<localCursorOffset)
            localCursorOffset = start
        }

        let inserted = Array(text)
        if !inserted.isEmpty {
            localTextContext.insert(contentsOf: inserted, at: localCursorOffset)
            localCursorOffset += inserted.count
        }

        trimLocalTextContext()
        if cursorContextToneEditArmed {
            cursorContextToneEditCache = previousWordBeforeLocalCursor()
        }
    }

    private func deleteLocalCharacterBeforeCursor() {
        guard localCursorOffset > 0, localCursorOffset <= localTextContext.count else { return }
        localTextContext.remove(at: localCursorOffset - 1)
        localCursorOffset -= 1
        cursorContextToneEditCache = previousWordBeforeLocalCursor()
    }

    private func deleteLocalWordBeforeCursor() {
        localCursorOffset = min(max(localCursorOffset, 0), localTextContext.count)
        guard localCursorOffset > 0 else { return }

        let end = localCursorOffset
        var start = end
        while start > 0 && Self.isCursorContextTrailingBoundary(String(localTextContext[start - 1])) {
            start -= 1
        }
        while start > 0 && !Self.isWordBoundary(String(localTextContext[start - 1])) {
            start -= 1
        }

        guard start < end else { return }
        localTextContext.removeSubrange(start..<end)
        localCursorOffset = start
        cursorContextToneEditCache = previousWordBeforeLocalCursor()
    }

    private func updateLocalCursorForNavigation(keyCode: Int64, flags: CGEventFlags) {
        localCursorOffset = min(max(localCursorOffset, 0), localTextContext.count)
        let wordNavigation = flags.contains(.maskAlternate) || flags.contains(.maskControl)

        switch keyCode {
        case 123:
            if wordNavigation {
                moveLocalCursorToPreviousWordBoundary()
            } else {
                localCursorOffset = max(0, localCursorOffset - 1)
            }
        case 124:
            if wordNavigation {
                moveLocalCursorToNextWordBoundary()
            } else {
                localCursorOffset = min(localTextContext.count, localCursorOffset + 1)
            }
        case 115:
            localCursorOffset = 0
        case 119:
            localCursorOffset = localTextContext.count
        default:
            localTextContext.removeAll(keepingCapacity: true)
            localCursorOffset = 0
        }

        cursorContextToneEditCache = previousWordBeforeLocalCursor()
    }

    private func moveLocalCursorToPreviousWordBoundary() {
        guard localCursorOffset > 0 else { return }
        var index = localCursorOffset

        while index > 0 && Self.isCursorContextTrailingBoundary(String(localTextContext[index - 1])) {
            index -= 1
        }
        while index > 0 && !Self.isWordBoundary(String(localTextContext[index - 1])) {
            index -= 1
        }

        localCursorOffset = index
    }

    private func moveLocalCursorToNextWordBoundary() {
        guard localCursorOffset < localTextContext.count else { return }
        var index = localCursorOffset

        while index < localTextContext.count && Self.isCursorContextTrailingBoundary(String(localTextContext[index])) {
            index += 1
        }
        while index < localTextContext.count && !Self.isWordBoundary(String(localTextContext[index])) {
            index += 1
        }

        localCursorOffset = index
    }

    private func previousWordBeforeLocalCursor() -> CursorToneContext? {
        localCursorOffset = min(max(localCursorOffset, 0), localTextContext.count)
        guard localCursorOffset > 0 else { return nil }

        var boundaryStart = localCursorOffset
        while boundaryStart > 0 && Self.isCursorContextTrailingBoundary(String(localTextContext[boundaryStart - 1])) {
            boundaryStart -= 1
        }

        var wordStart = boundaryStart
        while wordStart > 0 && !Self.isWordBoundary(String(localTextContext[wordStart - 1])) {
            wordStart -= 1
        }

        guard wordStart < boundaryStart else { return nil }
        return CursorToneContext(
            word: String(localTextContext[wordStart..<boundaryStart]),
            trailingBoundary: String(localTextContext[boundaryStart..<localCursorOffset])
        )
    }

    private func trimLocalTextContext() {
        let maxLength = 512
        guard localTextContext.count > maxLength else { return }
        let overflow = localTextContext.count - maxLength
        localTextContext.removeFirst(overflow)
        localCursorOffset = max(0, localCursorOffset - overflow)
    }

    /// Send backspaces synchronously via the event tap proxy (like GoxKey).
    /// Events posted via proxy skip our own event tap → zero round-trip latency.
    /// Reuses a single event pair for all backspaces (like GoxKey does).
    private func sendBackspacesSync(count: Int) {
        guard let proxy = currentProxy else { return }

        // Use nil event source like GoxKey (null_event_source).
        // Events with nil source get source_state_id != 1, so they won't
        // be re-processed even if they somehow reach our tap.
        guard let keyDown = CGEvent(keyboardEventSource: nil, virtualKey: 0x33, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: nil, virtualKey: 0x33, keyDown: false) else { return }

        keyDown.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        keyUp.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)

        // Reuse same event objects for all backspaces (like GoxKey)
        for _ in 0..<count {
            keyDown.tapPostEvent(proxy)
            keyUp.tapPostEvent(proxy)
        }
    }

    /// Send replacement text as a single CGEvent via the event tap proxy.
    /// Only keyDown, no keyUp — matches GoxKey's send_string behavior.
    private func sendTextSync(_ text: String) {
        guard let proxy = currentProxy else { return }

        var utf16 = Array(text.utf16)

        // CGEvent unicode string limit is 20 UniChars per event
        var offset = 0
        while offset < utf16.count {
            let end = min(offset + 20, utf16.count)
            var chunk = Array(utf16[offset..<end])

            if let keyDown = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true) {
                keyDown.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: &chunk)
                keyDown.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
                keyDown.tapPostEvent(proxy)
            }

            offset = end
        }
    }

    private func replaceCursorContextTone(last count: Int, with text: String, rawMode: Bool) {
        guard count > 0 || !text.isEmpty else { return }
        if currentProxy != nil {
            if rawMode {
                sendRawReplacementSync(backspaceCount: count, text: text)
            } else {
                if count > 0 {
                    sendBackspacesSync(count: count)
                }
                if !text.isEmpty {
                    sendTextSync(text)
                }
            }
            flushQueuedKeyDownEvents()
            return
        }

        replace(last: count, with: text, extraDeleteCount: 0, rawModeOverride: rawMode)
    }

    private func shouldUseRawTextInputReplacement(allowAXLookup: Bool = true) -> Bool {
        let bundleID = currentFrontmostBundleID()
        if let preferred = Self.preferredRawTextInputMode(for: bundleID) {
            cacheRawTextInputResult(preferred, bundleID: bundleID)
            lastRawTextInputResult = preferred
            return preferred
        }

        let now = ProcessInfo.processInfo.systemUptime
        if rawTextInputCacheTime > 0,
           rawTextInputCacheBundleID == bundleID,
           now - rawTextInputCacheTime <= rawTextInputCacheTTL {
            lastRawTextInputResult = rawTextInputCacheValue
            return rawTextInputCacheValue
        }

        guard allowAXLookup else {
            if rawTextInputCacheBundleID == bundleID {
                lastRawTextInputResult = rawTextInputCacheValue
                return rawTextInputCacheValue
            }
            lastRawTextInputResult = false
            return false
        }

        guard AXIsProcessTrusted() else {
            lastRawTextInputResult = false
            cacheRawTextInputResult(false, bundleID: bundleID)
            return false
        }
        let result = isFocusedElementRawTextSurface()
        lastRawTextInputResult = result
        cacheRawTextInputResult(result, bundleID: bundleID)
        return result
    }

    private func currentFrontmostBundleID() -> String? {
        frontmostBundleID ?? NSWorkspace.shared.frontmostApplication?.bundleIdentifier
    }

    private func cacheRawTextInputResult(_ value: Bool, bundleID: String?) {
        rawTextInputCacheBundleID = bundleID
        rawTextInputCacheValue = value
        rawTextInputCacheTime = ProcessInfo.processInfo.systemUptime
    }

    private func invalidateRawTextInputCache() {
        rawTextInputCacheBundleID = nil
        rawTextInputCacheValue = false
        rawTextInputCacheTime = 0
    }

    private static func preferredRawTextInputMode(for bundleID: String?) -> Bool? {
        guard let bundleID else { return nil }
        let normalized = bundleID.lowercased()
        if prefersDirectReplacement(for: normalized) {
            return false
        }
        return rawTextInputBundleIDs.contains(normalized) ? true : nil
    }

    private static func prefersDirectReplacement(for bundleID: String?) -> Bool {
        guard let bundleID else { return false }
        let normalized = bundleID.lowercased()
        if directReplacementBundleIDs.contains(normalized) {
            return true
        }
        return directReplacementBundleIDFragments.contains { normalized.contains($0) }
    }

    private func isFocusedElementRawTextSurface() -> Bool {
        guard let element = focusedElement() else { return false }

        let role = copyAttribute(element, name: kAXRoleAttribute) as? String
        if role == (kAXTextFieldRole as String) || role == (kAXComboBoxRole as String) {
            return false
        }

        if copyAXValue(from: element, name: kAXSelectedTextRangeAttribute) != nil {
            return false
        }

        if let parameterizedNames = copyAttribute(element, name: "AXParameterizedAttributeNames") as? [String] {
            let richTextAttributes: Set<String> = [
                "AXStringForRange",
                "AXRangeForLine",
                "AXLineForIndex",
                "AXRangeForPosition",
                "AXBoundsForRange"
            ]
            if parameterizedNames.contains(where: { richTextAttributes.contains($0) }) {
                return false
            }
        }

        var valueSettable = DarwinBoolean(false)
        if AXUIElementIsAttributeSettable(element, kAXValueAttribute as CFString, &valueSettable) == .success,
           valueSettable.boolValue {
            return false
        }

        var rangeSettable = DarwinBoolean(false)
        if AXUIElementIsAttributeSettable(element, kAXSelectedTextRangeAttribute as CFString, &rangeSettable) == .success,
           rangeSettable.boolValue {
            return false
        }

        if role == (kAXTextAreaRole as String) {
            return true
        }

        return copyAttribute(element, name: kAXValueAttribute) is String
    }

    private func sendRawReplacementSync(backspaceCount: Int, text: String) {
        guard currentProxy != nil else { return }
        guard backspaceCount > 0 || !text.isEmpty else { return }
        guard let source = CGEventSource(stateID: .privateState) else { return }

        let payload = String(repeating: "\u{8}", count: backspaceCount) + text
        sendRawTextSync(payload, source: source)
    }

    private func sendRawBackspacesSync(count: Int, source: CGEventSource) {
        guard let proxy = currentProxy else { return }
        guard let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 0x33, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: source, virtualKey: 0x33, keyDown: false) else { return }

        keyDown.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        keyUp.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)

        for _ in 0..<count {
            keyDown.tapPostEvent(proxy)
            keyUp.tapPostEvent(proxy)
        }
    }

    private func sendRawTextSync(_ text: String, source: CGEventSource) {
        guard let proxy = currentProxy else { return }

        let utf16 = Array(text.utf16)
        var offset = 0
        while offset < utf16.count {
            let end = min(offset + 20, utf16.count)
            var chunk = Array(utf16[offset..<end])

            if let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true) {
                keyDown.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: &chunk)
                keyDown.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
                keyDown.tapPostEvent(proxy)
            }

            offset = end
        }
    }

    private func sendRawReplacementPost(backspaceCount: Int, text: String) {
        guard backspaceCount > 0 || !text.isEmpty else { return }
        guard let source = CGEventSource(stateID: .privateState) else { return }

        let payload = String(repeating: "\u{8}", count: backspaceCount) + text
        sendTextPost(payload, source: source)
    }

    private func sendTextPost(_ text: String, source: CGEventSource) {
        var utf16 = Array(text.utf16)
        var offset = 0
        while offset < utf16.count {
            let end = min(offset + 20, utf16.count)
            var chunk = Array(utf16[offset..<end])

            if let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true) {
                keyDown.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: &chunk)
                keyDown.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
                keyDown.post(tap: .cghidEventTap)
            }

            offset = end
        }
    }

    /// Send a single arrow key event via the event tap proxy.
    /// Used for sentinel mechanism — navigate around the sentinel char.
    private func sendArrowSync(left: Bool) {
        guard let proxy = currentProxy else { return }
        let keyCode: CGKeyCode = left ? 123 : 124  // 123 = left arrow, 124 = right arrow
        guard let keyDown = CGEvent(keyboardEventSource: nil, virtualKey: keyCode, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: nil, virtualKey: keyCode, keyDown: false) else { return }
        keyDown.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        keyUp.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        keyDown.tapPostEvent(proxy)
        keyUp.tapPostEvent(proxy)
    }

    private func enqueueKeyDownEvent(_ event: CGEvent) {
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
        queuedKeyDownEvents.append(
            QueuedKeyDownEvent(
                keyCode: keyCode,
                flags: event.flags,
                unicodeString: event.keyboardGetUnicodeString()
            )
        )
        
        if queuedKeyDownEvents.count > 128 {
            queuedKeyDownEvents.removeAll(keepingCapacity: true)
            isInjectingReplacement = false
            displayBuffer = ""
            transformer.reset()
        }
    }
    
    private func beginReplacementInjection(backspaceCount: Int = 0) {
        isInjectingReplacement = true
    }
    
    private func finishReplacementInjection() {
        isInjectingReplacement = false
        scheduleFlushQueuedEvents()
    }
    
    private func noteInjectedKeyDown() {
        // No-op: we manage state via completion handler now
    }
    
    private func scheduleFlushQueuedEvents() {
        guard !flushQueuedEventsScheduled else { return }
        flushQueuedEventsScheduled = true
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.flushQueuedEventsScheduled = false
            self.flushQueuedKeyDownEvents()
        }
    }
    
    private func flushQueuedKeyDownEvents() {
        while !isInjectingReplacement && !queuedKeyDownEvents.isEmpty {
            let next = queuedKeyDownEvents.removeFirst()
            replayQueuedKeyDown(next)
        }
    }
    
    private func replayQueuedKeyDown(_ queued: QueuedKeyDownEvent) {
        let keyCode = queued.keyCode
        let flags = queued.flags

        if isToggleShortcut(keyCode: keyCode, flags: flags) {
            toggleViType()
            return
        }

        if shouldBypassVietnameseInput() {
            transformer.reset()
            displayBuffer = ""
            rawCompositionBuffer = ""
            clearCursorToneEditContext()
            lastRawTextInputResult = false
            sendQueuedKey(queued)
            return
        }

        let hasActionModifier = flags.contains(.maskCommand) ||
                                flags.contains(.maskControl) ||
                                flags.contains(.maskAlternate)

        if Self.isControlW(keyCode: keyCode, flags: flags) {
            transformer.deleteCurrentWord()
            deleteLocalWordBeforeCursor()
            displayBuffer = ""
            rawCompositionBuffer = ""
            armCursorToneEditContext()
            lastRawTextInputResult = false
            sendQueuedKey(queued)
            return
        }

        if Self.isOptionBackspace(keyCode: keyCode, flags: flags) {
            transformer.deleteCurrentWord()
            deleteLocalWordBeforeCursor()
            displayBuffer = ""
            rawCompositionBuffer = ""
            armCursorToneEditContext()
            lastRawTextInputResult = false
            sendQueuedKey(queued)
            return
        }

        if keyCode == Self.backspaceKey && !hasActionModifier {
            let shouldKeepCursorToneContext = cursorContextToneEditArmed
            transformer.deleteLastCharacter()
            deleteLocalCharacterBeforeCursor()
            displayBuffer = transformer.currentText()
            if shouldKeepCursorToneContext {
                armCursorToneEditContext()
            } else {
                clearCursorToneEditContext()
            }
            if !rawCompositionBuffer.isEmpty {
                rawCompositionBuffer.removeLast()
            }
            sendKey(CGKeyCode(Self.backspaceKey))
            return
        }

        let isNavigationKey = Self.navigationKeys.contains(keyCode)
        if keyCode == Self.forwardDeleteKey ||
           keyCode == Self.escapeKey ||
           isNavigationKey ||
           hasActionModifier {
            transformer.reset()
            displayBuffer = ""
            rawCompositionBuffer = ""
            if isNavigationKey {
                updateLocalCursorForNavigation(keyCode: keyCode, flags: flags)
                armCursorToneEditContext()
            } else {
                clearCursorToneEditContext()
            }
            lastRawTextInputResult = false
            sendKey(CGKeyCode(keyCode))
            return
        }

        guard let s = queued.unicodeString else {
            sendQueuedKey(queued)
            return
        }

        refreshTransformerSettings()

        if tryApplyCursorContextTone(input: s) {
            return
        }

        if Self.isWordBoundary(s) {
            _ = transformer.process(input: s)
            insertLocalText(s)
            rawCompositionBuffer = ""
            displayBuffer = ""
            clearCursorToneEditContext()
            sendQueuedKey(queued)
            return
        }

        if let action = transformer.process(input: s) {
            let rawMode = shouldUseRawTextInputReplacement()
            let extraDeleteCount = rawMode ? 0 : (shouldWipeGhostSuggestion() ? 1 : 0)

            if displayBuffer.isEmpty && action.deleteCount > 0 {
                let adjusted = adjustedReplacementForCursorContext(deleteCount: action.deleteCount, text: action.text)
                replace(last: adjusted.0, with: adjusted.1, extraDeleteCount: 0, rawModeOverride: rawMode)
                replaceLocalTextBeforeCursor(deleteCount: adjusted.0, with: adjusted.1)
                displayBuffer = adjusted.1
                return
            }

            // Same diff logic as handle(event:)
            let newDisplay: String
            if action.deleteCount >= displayBuffer.count {
                newDisplay = action.text
            } else {
                let keepCount = displayBuffer.count - action.deleteCount
                let prefix = String(displayBuffer.prefix(keepCount))
                newDisplay = prefix + action.text
            }

            if rawMode {
                let (diffBS, diffSuffix) = Self.diffParts(old: displayBuffer, new: newDisplay)
                replace(last: diffBS, with: diffSuffix, extraDeleteCount: 0, rawModeOverride: true)
                replaceLocalTextBeforeCursor(deleteCount: diffBS, with: diffSuffix)
            } else {
                let (diffBS, diffSuffix) = Self.diffParts(old: displayBuffer, new: newDisplay)
                let totalBS = extraDeleteCount + diffBS
                replace(last: totalBS, with: diffSuffix, extraDeleteCount: 0, rawModeOverride: rawMode)
                replaceLocalTextBeforeCursor(deleteCount: diffBS, with: diffSuffix)
            }
            displayBuffer = newDisplay
            clearCursorToneEditContext()
        } else {
            displayBuffer.append(s)
            insertLocalText(s)
            clearCursorToneEditContext()
            sendQueuedKey(queued)
        }
    }

    private func sendQueuedKey(_ queued: QueuedKeyDownEvent) {
        guard let source = CGEventSource(stateID: .combinedSessionState) else { return }
        let key = CGKeyCode(queued.keyCode)

        let down = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: true)
        down?.flags = queued.flags
        down?.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        down?.post(tap: .cghidEventTap)

        let up = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: false)
        up?.flags = queued.flags
        up?.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        up?.post(tap: .cghidEventTap)
    }

    private func sendKey(_ key: CGKeyCode) {
        guard let source = CGEventSource(stateID: .combinedSessionState) else { return }

        let down = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: true)
        down?.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        down?.post(tap: .cghidEventTap)

        let up = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: false)
        up?.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        up?.post(tap: .cghidEventTap)
    }

    private func sendText(_ text: String) {
        guard let source = CGEventSource(stateID: .combinedSessionState) else { return }
        var utf16 = Array(text.utf16)

        let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true)
        down?.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        utf16.withUnsafeMutableBufferPointer { buffer in
            guard let baseAddress = buffer.baseAddress else { return }
            down?.keyboardSetUnicodeString(
                stringLength: buffer.count,
                unicodeString: baseAddress
            )
        }
        down?.post(tap: .cghidEventTap)

        let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false)
        up?.setIntegerValueField(.eventSourceUserData, value: injectedEventTag)
        up?.post(tap: .cghidEventTap)
    }
}

extension AppDelegate {
    private func shouldWipeGhostSuggestion() -> Bool {
        guard AXIsProcessTrusted() else { return false }
        return readSelectionRange()
            .map { $0.isGhostSuggestion }
            ?? false
    }

    private func readSelectionRange() -> SelectionRangeContext? {
        let work: () -> SelectionRangeContext? = { [weak self] in
            guard let self, let element = self.focusedElement() else { return nil }
            guard let rangeValue = copyAXValue(from: element, name: kAXSelectedTextRangeAttribute) else { return nil }

            var range = CFRange()
            guard AXValueGetValue(rangeValue, .cfRange, &range) else { return nil }
            guard range.length > 0 else { return nil }

            let valueLength: Int?
            if let value = copyAttribute(element, name: kAXValueAttribute) as? String {
                valueLength = value.utf16.count
            } else {
                valueLength = nil
            }

            let selectedTextLength: Int?
            if let selectedText = copyAttribute(element, name: kAXSelectedTextAttribute) as? String {
                selectedTextLength = selectedText.utf16.count
            } else {
                selectedTextLength = nil
            }

            return SelectionRangeContext(range: range, valueLength: valueLength, selectedTextLength: selectedTextLength)
        }

        if Thread.isMainThread {
            return work()
        }
        return DispatchQueue.main.sync { work() }
    }

    private func readTextBeforeCursor() -> String? {
        let work: () -> String? = { [weak self] in
            guard let self, let element = self.focusedElement() else { return nil }
            guard let rangeValue = copyAXValue(from: element, name: kAXSelectedTextRangeAttribute) else { return nil }

            var range = CFRange()
            guard AXValueGetValue(rangeValue, .cfRange, &range) else { return nil }
            guard range.location >= 0 else { return nil }

            if let value = copyAttribute(element, name: kAXValueAttribute) as? String,
               range.location <= value.utf16.count {
                let utf16Index = value.utf16.index(value.utf16.startIndex, offsetBy: range.location)
                guard let cursorIndex = String.Index(utf16Index, within: value) else { return nil }
                return String(value[..<cursorIndex])
            }

            guard range.location > 0 else { return "" }
            let windowLength = min(range.location, 256)
            var queryRange = CFRange(location: range.location - windowLength, length: windowLength)
            guard let queryValue = AXValueCreate(.cfRange, &queryRange) else { return nil }
            guard let stringForRange = copyParameterizedAttribute(
                element,
                name: "AXStringForRange",
                parameter: queryValue
            ) as? String else { return nil }
            return stringForRange
        }

        if Thread.isMainThread {
            return work()
        }
        return DispatchQueue.main.sync { work() }
    }

    private func focusedElement() -> AXUIElement? {
        if let frontmost = NSWorkspace.shared.frontmostApplication {
            let appElement = AXUIElementCreateApplication(frontmost.processIdentifier)
            if let element = copyFocusedElement(from: appElement) {
                return element
            }
        }

        let systemWide = AXUIElementCreateSystemWide()
        return copyFocusedElement(from: systemWide)
    }

    private func copyFocusedElement(from root: AXUIElement) -> AXUIElement? {
        guard let value = copyAttribute(root, name: kAXFocusedUIElementAttribute) else { return nil }
        guard CFGetTypeID(value) == AXUIElementGetTypeID() else { return nil }
        return (value as! AXUIElement)
    }

    private func copyAttribute(_ element: AXUIElement, name: String) -> CFTypeRef? {
        var value: CFTypeRef?
        let result = AXUIElementCopyAttributeValue(element, name as CFString, &value)
        return result == .success ? value : nil
    }

    private func copyParameterizedAttribute(_ element: AXUIElement, name: String, parameter: CFTypeRef) -> CFTypeRef? {
        var value: CFTypeRef?
        let result = AXUIElementCopyParameterizedAttributeValue(
            element,
            name as CFString,
            parameter,
            &value
        )
        return result == .success ? value : nil
    }

    private func copyAXValue(from element: AXUIElement, name: String) -> AXValue? {
        guard let value = copyAttribute(element, name: name) else { return nil }
        guard CFGetTypeID(value) == AXValueGetTypeID() else { return nil }
        return (value as! AXValue)
    }


}

private struct SelectionRangeContext {
    let range: CFRange
    let valueLength: Int?
    let selectedTextLength: Int?

    var isGhostSuggestion: Bool {
        guard range.length > 0 else { return false }

        if let valueLength {
            let rangeEnd = Int(range.location + range.length)
            guard rangeEnd == valueLength else { return false }
        }

        if let selectedTextLength, selectedTextLength != Int(range.length) {
            return false
        }

        return true
    }
}

private struct CursorToneContext {
    let word: String
    let trailingBoundary: String
}

extension CGEvent {
    func keyboardGetUnicodeString() -> String? {
        var length = 0
        var chars = [UniChar](repeating: 0, count: 4)
        keyboardGetUnicodeString(
            maxStringLength: 4,
            actualStringLength: &length,
            unicodeString: &chars
        )
        return length > 0 ? String(utf16CodeUnits: chars, count: length) : nil
    }
}
