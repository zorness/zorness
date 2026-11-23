# Zorness Android

This Android project hosts the real DeepSeek Harness Web client in Android's platform WebView through Rust with Wry and Tao. The WebView starts on a small local connection page, then opens the interface served by a reachable `dsh web` Host. This is a WebView client, not a standalone Android port of the Node.js Host.

The Android app has no Compose screen or XML layout Views. A minimal Kotlin `WryActivity` supplies the Android lifecycle and JNI entry point required by Wry, and hides the system bars for immersive display.

## Build

Install JDK 17 or later, Android SDK platform 36, Android NDK 25.0.8775105, and Rust stable. Install the Android Rust targets with:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Set `ANDROID_HOME` and `ANDROID_SDK_ROOT` to the SDK directory, `NDK_HOME` or `ANDROID_NDK_HOME` to an installed NDK directory, and `JAVA_HOME` to the JDK. Add Cargo's bin directory to `PATH` so Gradle can run `cargo`.

The Rust Gradle task runs `cargo build --lib --target <android-triple>` and stages the resulting shared library in the matching `app/src/main/jniLibs/<abi>/` directory. Build the ARM64 debug APK from this directory with:

```sh
./gradlew :app:assembleArm64Debug
```

Use `./gradlew :app:assembleDebug` to build the configured Android ABI variants. Gradle writes APKs under `app/build/outputs/apk/`.

## Connect to a Harness Host

Run `dsh web` on a machine reachable from the phone. The default loopback URL is only reachable on that same machine. For a phone on the same LAN, bind the Host to one concrete network-interface address, configure its trusted host, and use TLS with a certificate trusted by Android. Do not bind to a wildcard address. For public access, use an HTTPS reverse proxy and the Host's documented trusted-host configuration.

Copy the complete authenticated entry URL printed by `dsh web`, including its `?token=` query, into the Android connection page. Treat that URL like a password. The app accepts HTTPS only; the Host exchanges its process launch token for a browser-session cookie and redirects to the clean Web application URL.

The `dsh web` Host serves the real interface, boot injections, client plugins, authenticated HTTP APIs, and Remote WebSocket streams. Configure the AI provider on that Host. The Android app does not run a second provider client, store an AI key, or start the Harness Node.js runtime locally.

See the upstream [Web profile documentation](../apps/cli/reference/README.md#web-profile) for listener, TLS, public URL, and trusted-host options. Wry's Android integration notes are in the [Wry mobile guide](https://github.com/tauri-apps/wry/blob/dev/MOBILE.md).
