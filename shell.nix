{ pkgs ? import <nixpkgs> {} }:

with pkgs;

stdenv.mkDerivation {
  name = "jude-rs";
  buildInputs = [ sass nodePackages.bower pkgconfig libsodium ];
  KEY_FILE = "${builtins.toString ./.}/secret.key";
  PGUSER = "jude";
}
