# ViType

Bộ gõ tiếng Việt cho macOS, hỗ trợ cả hai kiểu gõ **Telex** và **VNI**.

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)
![macOS](https://img.shields.io/badge/macOS-14.6%2B-brightgreen.svg)

## Tổng quan

ViType là bộ gõ tiếng Việt nhẹ, được xây dựng với **Rust core** để xử lý chuyển đổi văn bản hiệu suất cao và giao diện **Swift/SwiftUI** để tích hợp tự nhiên với macOS. Ứng dụng chặn input bàn phím toàn hệ thống và chuyển đổi phím gõ thành ký tự tiếng Việt với dấu thanh chính xác.

## Tính năng

- **Hai kiểu gõ**: Telex (dùng chữ cái) và VNI (dùng số)
- **Chuyển đổi thời gian thực**: Chuyển đổi ký tự tiếng Việt ngay khi gõ
- **Chế độ đặt dấu**: Kiểu cũ (chuẩn chính tả) hoặc Kiểu mới (đặt dấu vào nguyên âm chính)
- **Tự động sửa dấu**: Tự động di chuyển dấu thanh khi thêm nguyên âm
- **Bỏ qua kiểm tra âm tiết**: Tùy chọn bỏ qua kiểm tra cụm nguyên âm hợp lệ
- **Loại trừ ứng dụng**: Tắt ViType cho các ứng dụng cụ thể
- **Tích hợp Menu Bar**: Truy cập nhanh cài đặt và bật/tắt bộ gõ
- **Khởi động cùng hệ thống**: Tùy chọn tự động khởi động
- **Tự động cập nhật**: Cơ chế cập nhật tích hợp qua Sparkle

## Cài đặt

1. Tải file DMG từ [Releases](https://github.com/ttdatt/vitype/releases) hoặc [build từ mã nguồn](#build-từ-mã-nguồn)
2. Kéo `ViType.app` vào thư mục Applications
3. Khởi chạy ViType
4. **Cấp quyền Accessibility**: Khi được yêu cầu, vào System Settings > Privacy & Security > Accessibility và bật ViType. Sau đó khởi động lại ứng dụng ViType

## Sử dụng

### Bật/Tắt bộ gõ

- **Phím tắt**: `Ctrl+Shift+Space` (mặc định)
- **Menu bar**: Click vào icon ViType và chọn "Bật/Tắt gõ tiếng Việt"

### Menu Bar

Icon trên menu bar hiển thị trạng thái hiện tại:
- **V**: Đang bật gõ tiếng Việt
- **E**: Chế độ tiếng Anh (ViType tắt)

### Cài đặt

Truy cập cài đặt qua icon menu bar:
- **Kiểu gõ**: Chọn giữa Telex và VNI
- **Chế độ đặt dấu**: Kiểu cũ hoặc Kiểu mới
- **Tự động sửa dấu**: Bật/tắt tự động di chuyển dấu thanh
- **Bỏ qua kiểm tra âm tiết**: Bỏ qua kiểm tra cụm nguyên âm hợp lệ
- **Loại trừ ứng dụng**: Tắt ViType cho các ứng dụng cụ thể

### Bảng tham khảo nhanh

| Thao tác | Telex | VNI |
|----------|-------|-----|
| Sắc (´) | `s` | `1` |
| Huyền (`) | `f` | `2` |
| Hỏi (ˀ) | `r` | `3` |
| Ngã (~) | `x` | `4` |
| Nặng (.) | `j` | `5` |
| Xóa dấu | `z` | `0` |
| â, ê, ô | `aa`, `ee`, `oo` | `a6`, `e6`, `o6` |
| ơ, ư | `ow`, `uw` | `o7`, `u7` |
| ă | `aw` | `a8` |
| đ | `dd` | `d9` |

**Ví dụ**:
| Từ | Telex | VNI |
|----|-------|-----|
| việt | `vieejt` | `vie6t5` |
| đẹp | `ddejp` | `d9e5p` |
| người | `nguwowif` | `ngu7o7i2` |

---

## Build từ mã nguồn

### Yêu cầu

- **macOS 14.6** trở lên
- **Xcode 16.2** trở lên với Command Line Tools
- **Rust toolchain** (stable) - cài đặt qua [rustup](https://rustup.rs/)

### 1. Clone Repository

```bash
git clone --recursive https://github.com/ttdatt/vitype.git
cd vitype
```

Nếu đã clone mà không có `--recursive`:

```bash
git submodule update --init --recursive
```

### 2. Build Rust Core

```bash
cd ViType-core
cargo build --release
```

### 3. Build ứng dụng macOS

Dùng Xcode:
```bash
xcodebuild -project ViType-macos/ViType.xcodeproj \
  -scheme ViType -configuration Release build
```

Hoặc mở `ViType-macos/ViType.xcodeproj` trong Xcode và build với Cmd+B.

### 4. Tạo DMG (Tùy chọn)

```bash
cd ViType-macos
bash ./scripts/build_dmg.sh
```

File DMG sẽ được tạo trong `ViType-macos/dist/`.

Để build có chữ ký và notarize:
```bash
bash ./scripts/build_dmg.sh --sign --notarize --apple-id "your@email.com"
```

---

## Kiến trúc

```
┌─────────────────────────────────────────────────────────┐
│                    ViType-macos                         │
│                  (Swift/SwiftUI)                        │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  Menu Bar   │  │   Cài đặt    │  │  CGEvent Tap  │  │
│  │  Manager    │  │    Views     │  │  (Bàn phím)   │  │
│  └─────────────┘  └──────────────┘  └───────┬───────┘  │
│                                             │          │
│  ┌──────────────────────────────────────────▼────────┐ │
│  │              KeyTransformer (FFI Bridge)          │ │
│  └──────────────────────────────────────────┬────────┘ │
└─────────────────────────────────────────────┼──────────┘
                                              │ C FFI
┌─────────────────────────────────────────────▼──────────┐
│                     ViType-core                        │
│                       (Rust)                           │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ VitypeEngine│  │  Telex/VNI   │  │   Diacritics  │  │
│  │  (lib.rs)   │  │    Rules     │  │    Helpers    │  │
│  └─────────────┘  └──────────────┘  └───────────────┘  │
└────────────────────────────────────────────────────────┘
```

**Tại sao tách riêng?**
- **Rust core**: Xử lý phím gõ hiệu suất cao, chuyển đổi Unicode và quy tắc bộ gõ được hưởng lợi từ tốc độ và an toàn bộ nhớ của Rust
- **Swift UI**: Trải nghiệm macOS native với SwiftUI cho cài đặt, menu bar và tích hợp hệ thống (Accessibility APIs, CGEvent)

## Cấu trúc dự án

```
ViType/
├── ViType-core/              # Rust core engine
│   ├── src/
│   │   ├── lib.rs            # Logic engine chính, đặt dấu thanh
│   │   ├── telex.rs          # Quy tắc Telex
│   │   ├── vni.rs            # Quy tắc VNI
│   │   ├── diacritics.rs     # Helper xử lý dấu
│   │   ├── common.rs         # Bảng ánh xạ nguyên âm/dấu thanh
│   │   ├── ffi.rs            # C FFI exports
│   │   └── tests/            # Unit tests
│   ├── Cargo.toml
│   ├── TELEX_RULES.md        # Tài liệu quy tắc Telex
│   └── VNI_RULES.md          # Tài liệu quy tắc VNI
│
├── ViType-macos/             # Ứng dụng macOS
│   ├── ViType/
│   │   ├── AppDelegate.swift         # CGEvent tap, xử lý bàn phím
│   │   ├── KeyTransformer.swift      # FFI bridge đến Rust core
│   │   ├── MenuBarManager.swift      # UI menu bar
│   │   ├── ContentView.swift         # Cửa sổ cài đặt chính
│   │   ├── GeneralSettingsView.swift # Cài đặt chung
│   │   ├── AdvancedSettingsView.swift# Cài đặt nâng cao
│   │   ├── AppExclusionView.swift    # Cài đặt loại trừ ứng dụng
│   │   └── vitype_core.h             # C header cho FFI
│   ├── ViType.xcodeproj/
│   └── scripts/
│       └── build_dmg.sh      # Script tạo DMG
│
├── AGENTS.md                 # Hướng dẫn repository
└── LICENSE                   # GPL-3.0
```

## Giấy phép

Dự án này được cấp phép theo **GNU General Public License v3.0** - xem file [LICENSE](LICENSE) để biết chi tiết.
