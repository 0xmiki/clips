{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "clips-tauri-desktop-env";

  buildInputs = with pkgs; [
    at-spi2-atk
    atkmm
    cairo
    harfbuzz
    bun
    nodejs
    rustup         
    cargo
    cargo-tauri
    pkg-config
    openssl
    gtk3
    libayatana-appindicator
    gobject-introspection
    librsvg
    glib
    glib.dev
    dbus
    libsoup_3
    pango
    webkitgtk_4_1
    gdk-pixbuf
    ffmpeg
    yt-dlp
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    gst_all_1.gst-plugins-ugly
    gst_all_1.gst-libav
    gsettings-desktop-schemas
  ];

  shellHook = ''
    export PATH="$HOME/.bun/bin:$PATH"
    export BUN_INSTALL="$HOME/.bun"
    export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share:${pkgs.gtk3}/share:$XDG_DATA_DIRS"
    # for wayland
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
    # use this for x11
    # export WEBKIT_DISABLE_COMPOSITING_MODE=1
    # Set Rust to use the stable toolchain by default
    rustup default stable

    echo "🧪 Bun + Tauri + Svelte dev environment ready"
  '';
}
