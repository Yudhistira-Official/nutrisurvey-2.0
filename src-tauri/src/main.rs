#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix EGL_BAD_PARAMETER on Wayland/AMD — force WebKit software rendering
    #[cfg(target_os = "linux")]
    {
        // Prefer X11/XWayland backend to avoid EGL issues on Wayland
        if std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("GDK_BACKEND").is_err() {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        // Disable WebKit hardware acceleration compositing
        if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }
    nutrisurvey_lib::run();
}
