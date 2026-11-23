# Zorness Android architecture

The root project description remains unchanged. Android uses Rust and Wry/Tao to create the application window and host Android's platform WebView. The WebView displays a local connection page, then navigates to the actual DeepSeek Harness Web client served by `dsh web`.

```text
Android app
  Rust + Tao + Wry
    local connection page -> Android WebView -> HTTPS dsh web Host
                                               Web client + boot injections
                                               authenticated APIs + Remote WebSocket
                                               configured AI provider
  Kotlin WryActivity + JNI entry point
```

## Responsibilities

- **Rust** owns the Android event loop and Wry WebView. Its local page accepts an HTTPS `dsh web` entry URL and does not bundle the upstream Web application or implement a second Harness Host.
- **TypeScript/JavaScript** remains the upstream Web client served by the Host. The Host supplies boot injections, plugin bundles, session APIs, and the Remote stream endpoint required by the real interface.
- **Kotlin** is limited to the Android Activity and the Android-specific full-screen behavior needed by Wry. It contains no Compose UI or XML layout Views.
- **Java** has no application logic in this integration.
- **AI connectivity** is provided through the connected `dsh web` Host. The Host owns provider configuration and credentials; the Android app does not collect or persist provider keys.

## Runtime requirements

The `dsh web` process and its configured provider must remain available while the phone uses the app. The default loopback listener is not reachable from a separate phone. A reachable Host must use an Android-trusted HTTPS certificate, a concrete reachable bind address, and the Host's trusted-host configuration.

The entry URL contains the running Host's process launch token. The WebView passes it to the Host, which establishes a browser-session cookie and redirects to the clean application URL. The connection page and the Android app do not save or log the launch URL.

The local HTML page is only a connection form. A standalone Android Host would require porting the Node.js profile launcher, Cordis plugin composition, authentication, HTTP APIs, WebSocket streams, and provider integrations; that is outside this WebView integration.

## Android build integration

The Android Gradle task invokes `cargo build --lib --target` for each configured Rust Android triple, then copies the generated `.so` into the matching Android ABI's `jniLibs` directory for APK packaging. The checked-in Kotlin Wry Activity and JNI bindings provide the Android lifecycle glue. Full-screen mode uses AndroidX `WindowInsetsControllerCompat` with transient system bars available by swipe.
