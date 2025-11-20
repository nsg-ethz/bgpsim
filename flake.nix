{
  description = "A Network Control-Plane Simulator";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05"; };

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      rustSrc = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = name: type:
          !builtins.elem (baseNameOf name) [ "./target" ".git" ];
      };
      manifestPath = "${toString rustSrc}/Cargo.toml";
      manifest = builtins.fromTOML (builtins.readFile manifestPath);
    in {
      devShells."${system}".default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rust-analyzer
          rustfmt
          rustc
          trunk
          tailwindcss_3
          llvmPackages.bintools
        ];
      };

    };
}
