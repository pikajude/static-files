{ pkgs ? import <nixpkgs> {}
, keyFile ? "${builtins.toString ./.}/secret.key"
, pgUser ? "pikajude"
, pw ? ./example.password }:

with pkgs;

let
  nodePkgs = callPackage ./generated/node-composition.nix {};

in (makeRustPlatform rustNightlyBin).buildRustPackage rec {
  name = "jude-rs-${version}";
  version = "0.1.0";
  src = builtins.filterSource (name: type: !
    (type == "directory" && (baseNameOf (toString name) == "target")) ||
    (baseNameOf (toString name) == "secret.password")
    ) ./.;

  depsSha256 = "1h3w2v12mvq22qf02vqazr7a36132yx12203rb6gbql1263hbdgx";

  buildInputs = [ cmake pkgconfig libsodium sass nodePkgs.cssnano-cli ];

  prePatch = ''
    cat ${pw} > secret.password
  '';

  KEY_FILE = keyFile;
  PGUSER = pgUser;
}
