
{ lib
, stdenv
, stdenvNoCC
, rustPlatform
, cargo-tauri
, bun
, nodejs
, openssl
, pkg-config
, glib-networking
, webkitgtk_4_1
, wrapGAppsHook4
, gst_all_1
, gsettings-desktop-schemas
, glib
}:

let
  bunDeps = stdenvNoCC.mkDerivation {
    pname = "clips-bun-deps";
    version = "0.1.0";
    nativeBuildInputs = [ bun nodejs ];
    src = lib.fileset.toSource {
      root = ./../../.;
      fileset = lib.fileset.unions [
        ./../../package.json
        ./../../bun.lock
      ];
    };

    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      bun install \
        --frozen-lockfile \
        --ignore-scripts \
        --backend copyfile \
        --force
      cp -r node_modules/ $out
    '';

    outputHashMode = "recursive";
    outputHash = "sha256-/+e4012GxzLeUGayDJt8rL/LVYW3qtjOJ7QeKzhVCs0="; 
  };

in rustPlatform.buildRustPackage (finalAttrs: {
  pname    = "clips";
  version  = "0.1.0";
  src      = ./../../.;

  cargoHash = "sha256-7HK3mNHTpC3Wn5fRZ9gl09vPpE5PrjRY+AuDQClNErM=";

  nativeBuildInputs = with lib; [
    cargo-tauri.hook
    bun
    nodejs
    pkg-config
    glib
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    gst_all_1.gst-plugins-ugly
    gst_all_1.gst-libav
    gsettings-desktop-schemas

  ] ++ lib.optionals stdenv.hostPlatform.isLinux [
    wrapGAppsHook4
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    glib-networking
    openssl
    webkitgtk_4_1
  ];

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;

  configurePhase = ''
    cp -r ${bunDeps} node_modules
    chmod -R +w node_modules
    patchShebangs node_modules/{*,.*}
  '';

  meta = with lib; {
    description = "Clips: A Tauri + Svelte + Bun desktop application";
    license     = licenses.mit;
    platforms   = platforms.linux;
    maintainers = with maintainers; [ 0xmiki ];
     mainProgram = "clips";
  };
})
