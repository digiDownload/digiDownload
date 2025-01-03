{
  description = "digi_download developement flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config
            openssl # reqwests dependency | could go rust only with rustls
          ];

          shellHook = ''
            hyprctl dispatch "exec [workspace 1 silent] burpsuite"

            mkdir /tmp/digi
            hyprctl dispatch "exec [workspace 4 silent] nautilus /tmp/digi"

            rust-rover
          '';
        };
      });
}
