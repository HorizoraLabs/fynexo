# Fynexo Build Commands

## PC Platform Build
- Development Run: `cargo run`
- Release Build: `cargo build --release`

## Mobile (Android) Platform Build
- Build APK (Modern 64-bit Phones Only - Recommended, 4x faster & smaller): `cargo android apk build --release aarch64`
- Build Universal APK (All 4 architectures): `cargo android apk build --release`
- Build APK (Debug): `cargo android apk build aarch64`
- Run on Connected Phone / Emulator: `cargo android run`
- Open in Android Studio: `cargo android open`
