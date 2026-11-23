use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
use wry::{http, WebView, WebViewBuilder};

#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("zorness-android"),
    );
}

#[cfg(not(target_os = "android"))]
fn init_logging() {
    env_logger::init();
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("attempt to unwind out of Rust with error: {error:?}");
            std::process::abort()
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn start_app_inner() {
    stop_unwind(main);
}

#[no_mangle]
#[inline(never)]
#[cfg(any(target_os = "android", target_os = "ios"))]
pub extern "C" fn start_app() {
    #[cfg(target_os = "android")]
    {
        tao::android_binding!(
            app_zorness,
            zorness_android,
            Rust,
            wry::android_setup,
            start_app_inner
        );
        wry::android_binding!(app_zorness, zorness_android);
    }

    #[cfg(target_os = "ios")]
    start_app_inner();
}

pub fn main() {
    init_logging();
    let event_loop = EventLoop::new();
    let mut webview = None;
    let mut window = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                let app_window = build_window(event_loop).expect("create Zorness window");
                webview.replace(build_webview(&app_window).expect("create Harness WebView"));
                window.replace(app_window);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested { .. },
                ..
            } => {
                webview.take();
                window.take();
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn build_window(event_loop: &EventLoopWindowTarget<()>) -> anyhow::Result<Window> {
    Ok(WindowBuilder::new()
        .with_title("Zorness")
        .build(event_loop)?)
}

fn build_webview(window: &Window) -> anyhow::Result<WebView> {
    let builder = WebViewBuilder::new()
        .with_url("zorness://app/index.html")
        .with_devtools(cfg!(debug_assertions))
        .with_custom_protocol("zorness".into(), move |_webview_id, _request| {
            http::Response::builder()
                .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(http::header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                .header(
                    http::header::CONTENT_SECURITY_POLICY,
                    "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; connect-src 'none'; base-uri 'none'; form-action 'none'",
                )
                .body(CONNECTION_PAGE.as_bytes().into())
                .unwrap_or_default()
        });

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let webview = builder.build(window)?;

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        builder.build_gtk(window.default_vbox().expect("GTK window has a vbox"))?
    };

    Ok(webview)
}

const CONNECTION_PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
  <meta name="theme-color" content="#111318">
    <title>Zorness · Connect to Harness</title>
  <style>
    :root { color-scheme: dark; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
    * { box-sizing: border-box; }
    body { margin: 0; min-height: 100vh; background: #111318; color: #f1f3f6; display: grid; place-items: center; padding: 24px; }
    main { width: min(100%, 520px); }
    .mark { color: #9ca8bb; font-size: 12px; font-weight: 700; letter-spacing: .14em; text-transform: uppercase; }
    h1 { margin: 18px 0 10px; font-size: clamp(30px, 8vw, 42px); letter-spacing: -.035em; }
    p { color: #aeb5c2; line-height: 1.55; }
    label { display: block; margin: 28px 0 8px; color: #dce1e9; font-size: 14px; font-weight: 600; }
    input { width: 100%; min-height: 54px; border: 1px solid #343a46; border-radius: 12px; padding: 0 15px; background: #1a1e26; color: #f6f7fa; font: inherit; outline: none; }
    input:focus { border-color: #94a9d1; box-shadow: 0 0 0 3px #94a9d129; }
    button { width: 100%; min-height: 52px; margin-top: 14px; border: 0; border-radius: 12px; background: #dce5f7; color: #171b22; font: inherit; font-weight: 700; }
    button:active { transform: translateY(1px); }
    #error { min-height: 1.4em; margin: 12px 0 0; color: #ff9f9f; font-size: 14px; }
    .note { margin-top: 28px; padding-top: 18px; border-top: 1px solid #2b303a; font-size: 13px; }
  </style>
</head>
<body>
  <main>
    <div class="mark">Zorness · Android</div>
    <h1>Connect to Harness</h1>
    <p>Paste the authenticated Web entry URL printed by a reachable <code>dsh web</code> Host. The Host exchanges its process launch token for a browser session, then serves the actual Harness interface and its configured AI providers.</p>
    <form id="connect-form">
      <label for="host-url">Harness Web entry URL</label>
      <input id="host-url" type="url" inputmode="url" autocomplete="url" autocapitalize="off" spellcheck="false" placeholder="https://harness.example/?token=…" required>
      <button type="submit">Open Harness</button>
      <p id="error" role="alert" aria-live="polite"></p>
    </form>
    <p class="note">Treat this URL like a password: do not share or log it. Use HTTPS with a certificate trusted by Android; the Host redirects to a clean URL after issuing its session cookie. A Host bound to another device’s loopback address is not reachable from this phone.</p>
  </main>
  <script>
    const form = document.getElementById('connect-form');
    const field = document.getElementById('host-url');
    const error = document.getElementById('error');
    form.addEventListener('submit', event => {
      event.preventDefault();
      error.textContent = '';
      try {
        const target = new URL(field.value.trim());
        if (target.protocol !== 'https:') {
          throw new Error('Use an HTTPS URL with a certificate trusted by Android.');
        }
        if (target.username || target.password) {
          throw new Error('Do not put a username or password in the URL.');
        }
        window.location.assign(target.href);
      } catch (reason) {
        error.textContent = reason instanceof Error ? reason.message : 'Enter a valid Harness Web URL.';
      }
    });
  </script>
</body>
</html>"##;
