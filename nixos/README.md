# Running **Clips** on NixOS

This guide helps you get the **Clips** video clipping app working smoothly on NixOS — including properly bundling `yt-dlp` for use inside an AppImage.

---

## 📦 1. Build `yt-dlp` as a Static Binary

The version of `yt-dlp` in the Nix package registry depends on a Python interpreter and **does not work** when called from within an AppImage. To fix this:

- **Build a static binary yourself** using the provided Nix files in this repo.
- This ensures compatibility, better performance, and access to the latest updates.

> ⚠️ If you had `yt-dlp` installed from Nixpkgs previously, uninstall it with:

```sh
nix-env -e yt-dlp
```

Removing it from your `configuration.nix` is **not enough**.

---

## 🔧 2. Build the `Clips` Derivation

### Steps

1. Clone this repo:

   ```sh
   git clone https://github.com/your-user/clips
   cd clips
   ```

2. Enter the dev shell:

   ```sh
   nix-shell
   ```

3. Install JS deps:

   ```sh
   bun install
   ```

4. (Optional) Test the build:

   ```sh
   NIXPKGS_ALLOW_UNFREE=1 nix-build release.nix
   ```

5. Add it to your system config:

### Example `configuration.nix` Snippet

```nix
{ config, pkgs, ... }:

let
  yt-dlp = pkgs.callPackage /path/to/clips/nixos/yt-dlp/default.nix {};
  clips  = pkgs.callPackage /path/to/clips/nixos/clips/default.nix {};
in {
  environment.systemPackages = with pkgs; [
    yt-dlp
    clips
  ];
}
```

---

## 🐛 Troubleshooting: Blank or White Screen

Depending on your display server, you may need to set environment variables:

- **Wayland**:

  ```sh
  export WEBKIT_DISABLE_DMABUF_RENDERER=1
  ```

- **X11**:

  ```sh
  export WEBKIT_DISABLE_COMPOSITING_MODE=1
  ```

- TODO: Make the `default.nix` fetch the repo instead of requiring `git clone`
