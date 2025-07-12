{ lib, stdenv, fetchurl }:
# Change the version to the latest one and also update the hash accordingly 
let
  version = "2025.06.30";
  hash = "sha256:e2d5598b3936ff991d57099291be5bf60c5af2ef6ad4dd8d6800bb7b10ca50a7";
in
stdenv.mkDerivation (finalAttrs: {
  pname = "yt-dlp";
  inherit version;

  src = fetchurl {
    url = "https://github.com/yt-dlp/yt-dlp/releases/download/${version}/yt-dlp_linux";
    sha256 = hash;
  };
  dontUnpack = true;
  installPhase = ''
    mkdir -p $out/bin
    cp $src $out/bin/yt-dlp
    chmod +x $out/bin/yt-dlp
  '';

  meta = {
    description = "Statically linked yt-dlp binary";
    homepage = "https://github.com/yt-dlp/yt-dlp";
    license = lib.licenses.unlicense;
    mainProgram = "yt-dlp";
    maintainers = [];
    platforms = [ "x86_64-linux" "aarch64-linux" ];
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
  };
})
