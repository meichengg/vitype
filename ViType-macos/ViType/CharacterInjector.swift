//
//  CharacterInjector.swift
//  ViType
//
//  Created by Tran Dat on 24/12/25.
//

import Cocoa
import Carbon

// MARK: - Event Marker
// Used to identify events injected by ViType - prevents re-processing by event tap
// This is critical for avoiding infinite loops
let kViTypeEventMarker: Int64 = 0x11EE22DD

// MARK: - Injection Types

enum InjectionMethod {
    case fast           // Default: backspace + text with minimal delays (High performance)
    case slow           // Terminals/IDEs: backspace + text with higher delays
    case extraSlow      // TUI apps (Claude CLI, etc.): higher delays for slow event-loop apps
    case passthrough    // Bypass processing
    
    var delays: (backspace: UInt32, wait: UInt32, text: UInt32) {
        switch self {
        case .fast:         return (200, 500, 200)          // Ultra fast: 0.2ms BS, 0.5ms Wait, 0.2ms Text
        case .slow:         return (10000, 20000, 10000)    // 10ms/20ms/10ms for Terminals
        case .extraSlow:    return (15000, 5000, 15000)     // 15ms/5ms/15ms for TUI apps (gap < 1 frame @30fps)
        case .passthrough:  return (0, 0, 0)
        }
    }
}

enum TextSendingMethod {
    case chunked      // Send multiple chars per CGEvent (faster, default)
    case oneByOne     // Send one char at a time
}

class CharacterInjector {
    
    // MARK: - Properties
    
    private var eventSource: CGEventSource?
    
    /// Semaphore to ensure injection completes before next keystroke is processed
    /// This prevents race conditions where backspace arrives before previous injection is rendered
    private let injectionSemaphore = DispatchSemaphore(value: 1)
    
    // MARK: - Initialization
    
    init() {
        // Use .privateState to isolate injected events from system event state
        eventSource = CGEventSource(stateID: .privateState)
    }
    
    /// Detect injection method based on current app
    private func detectInjectionMethod() -> (method: InjectionMethod, textMethod: TextSendingMethod) {
        // Fast terminals (GPU-accelerated)
        let fastTerminals: Set<String> = [
            "io.alacritty", "com.mitchellh.ghostty", "net.kovidgoyal.kitty",
            "com.github.wez.wezterm", "com.raphaelamorim.rio"
        ]
        
        // Medium/Slow terminals
        let slowTerminals: Set<String> = [
            "com.apple.Terminal", "com.googlecode.iterm2", "dev.warp.Warp-Stable",
            "co.zeit.hyper", "org.tabby", "com.termius-dmg.mac",
            "com.microsoft.VSCode", "com.microsoft.VSCodeInsiders", "com.visualstudio.code.oss"
        ]
        
        guard let bundleID = NSWorkspace.shared.frontmostApplication?.bundleIdentifier else {
            return (.fast, .chunked)
        }
        
        // Terminal-like apps -> slow
        if fastTerminals.contains(bundleID) || slowTerminals.contains(bundleID) {
            return (.slow, .oneByOne)
        }
        
        // JetBrains IDEs -> Slow method + Chunked
        if bundleID.hasPrefix("com.jetbrains.") {
            return (.slow, .chunked)
        }
        
        // Default -> Fast method + Chunked
        return (.fast, .chunked)
    }
    
    // MARK: - Synchronized Injection
    
    /// Inject text replacement synchronously - backspaces + new text in one atomic operation
    func injectSync(backspaceCount: Int, text: String, proxy: CGEventTapProxy?) {
        // Acquire semaphore for entire injection operation
        injectionSemaphore.wait()
        defer { injectionSemaphore.signal() }
        
        // Create NEW event source for each injection to ensure independent state
        eventSource = CGEventSource(stateID: .privateState)
        
        let (method, textMethod) = detectInjectionMethod()
        
        let delays = method.delays
        
        // For slow method (terminals), use direct post for better reliability
        let useDirectPost = (method == .slow)
        
        // Step 1: Send backspaces
        if backspaceCount > 0 {
            for _ in 0..<backspaceCount {
                sendKeyPress(0x33, proxy: proxy, useDirectPost: useDirectPost) // 0x33 is Delete (Backspace)
                usleep(delays.backspace)
            }
            usleep(delays.wait)
        }
        
        // Step 2: Send text
        if !text.isEmpty {
            switch textMethod {
            case .oneByOne:
                sendTextOneByOne(text, delay: delays.text, proxy: proxy, useDirectPost: useDirectPost)
            case .chunked:
                sendTextChunked(text, delay: delays.text, proxy: proxy, useDirectPost: useDirectPost)
            }
        }
        
        // Settle time
        let settleTime: UInt32 = (method == .slow) ? 20000 : 5000
        usleep(settleTime)
    }
    
    // MARK: - Internal Sending Methods
    
    private func sendKeyPress(_ keyCode: CGKeyCode, proxy: CGEventTapProxy?, useDirectPost: Bool) {
        guard let source = eventSource else { return }
        
        guard let keyDown = CGEvent(keyboardEventSource: source, virtualKey: keyCode, keyDown: true),
              let keyUp = CGEvent(keyboardEventSource: source, virtualKey: keyCode, keyDown: false) else { return }
        
        keyDown.setIntegerValueField(.eventSourceUserData, value: kViTypeEventMarker)
        keyUp.setIntegerValueField(.eventSourceUserData, value: kViTypeEventMarker)
        
        if useDirectPost || proxy == nil {
            keyDown.post(tap: .cghidEventTap)
            keyUp.post(tap: .cghidEventTap)
        } else {
            keyDown.tapPostEvent(proxy!)
            keyUp.tapPostEvent(proxy!)
        }
    }
    
    private func sendTextChunked(_ text: String, delay: UInt32, proxy: CGEventTapProxy?, useDirectPost: Bool) {
        guard let source = eventSource else { return }
        
        // Split text by special chars
        var segments: [(type: SegmentType, content: String)] = []
        var currentSegment = ""
        
        for char in text {
            if char == "\n" || char == "\r" {
                if !currentSegment.isEmpty {
                    segments.append((.text, currentSegment))
                    currentSegment = ""
                }
                segments.append((.newline, ""))
            } else if char == "\t" {
                if !currentSegment.isEmpty {
                    segments.append((.text, currentSegment))
                    currentSegment = ""
                }
                segments.append((.tab, ""))
            } else {
                currentSegment.append(char)
            }
        }
        if !currentSegment.isEmpty {
            segments.append((.text, currentSegment))
        }
        
        for (index, segment) in segments.enumerated() {
            switch segment.type {
            case .newline:
                sendKeyPress(0x24, proxy: proxy, useDirectPost: useDirectPost) // Return
            case .tab:
                sendKeyPress(0x30, proxy: proxy, useDirectPost: useDirectPost) // Tab
            case .text:
                let utf16 = Array(segment.content.utf16)
                var offset = 0
                let chunkSize = 20
                
                while offset < utf16.count {
                    let end = min(offset + chunkSize, utf16.count)
                    var chunk = Array(utf16[offset..<end])
                    
                    if let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
                       let keyUp = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) {
                        
                        keyDown.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: &chunk)
                        keyUp.keyboardSetUnicodeString(stringLength: chunk.count, unicodeString: &chunk)
                        
                        keyDown.setIntegerValueField(.eventSourceUserData, value: kViTypeEventMarker)
                        keyUp.setIntegerValueField(.eventSourceUserData, value: kViTypeEventMarker)
                        
                        if useDirectPost || proxy == nil {
                            keyDown.post(tap: .cghidEventTap)
                            keyUp.post(tap: .cghidEventTap)
                        } else {
                            keyDown.tapPostEvent(proxy!)
                            keyUp.tapPostEvent(proxy!)
                        }
                    }
                    
                    offset = end
                    if delay > 0 && offset < utf16.count {
                        usleep(delay)
                    }
                }
            }
            
            if delay > 0 && index < segments.count - 1 {
                usleep(delay)
            }
        }
    }
    
    private func sendTextOneByOne(_ text: String, delay: UInt32, proxy: CGEventTapProxy?, useDirectPost: Bool) {
        guard let source = eventSource else { return }
        
        for (index, char) in text.enumerated() {
            if char == "\n" || char == "\r" {
                sendKeyPress(0x24, proxy: proxy, useDirectPost: useDirectPost)
                continue
            }
            if char == "\t" {
                sendKeyPress(0x30, proxy: proxy, useDirectPost: useDirectPost)
                continue
            }
            
            var utf16 = Array(String(char).utf16)
            if let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
               let keyUp = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) {
                
                keyDown.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: &utf16)
                keyUp.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: &utf16)
                
                keyDown.setIntegerValueField(.eventSourceUserData, value: kViTypeEventMarker)
                keyUp.setIntegerValueField(.eventSourceUserData, value: kViTypeEventMarker)
                
                if useDirectPost || proxy == nil {
                    keyDown.post(tap: .cghidEventTap)
                    keyUp.post(tap: .cghidEventTap)
                } else {
                    keyDown.tapPostEvent(proxy!)
                    keyUp.tapPostEvent(proxy!)
                }
            }
            
            if delay > 0 && index < text.count - 1 {
                usleep(delay)
            }
        }
    }
    
    private enum SegmentType {
        case text, newline, tab
    }
}
