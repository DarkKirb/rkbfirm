{
  description = "rust-template";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixpkgs-unstable;
    flake-utils.url = github:numtide/flake-utils;

    rust-overlay = {
      url = github:oxalica/rust-overlay;
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    cargo2nix = {
      url = github:cargo2nix/cargo2nix;
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, cargo2nix, ... } @ inputs: flake-utils.lib.eachSystem [ "aarch64-linux" "x86_64-linux" ] (system:
    let
      overlays = [
        cargo2nix.overlays.default
        (import rust-overlay)
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      rustPkgs = pkgs.rustBuilder.makePackageSet {
        packageFun = import ./Cargo.nix;
        rustChannel = "nightly";
        target = "thumbv6m-none-eabi";
        packageOverrides = pkgs: pkgs.rustBuilder.overrides.all;
      };
    in
    rec {
      devShells.default = with pkgs; mkShell {
        buildInputs = [
          (rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" ];
            targets = ["thumbv6m-none-eabi"];
          })
          cargo2nix.packages.${system}.cargo2nix
          elf2uf2-rs
          cargo-embed
          llvmPackages_latest.bintools
        ];
      };
      packages = rec { 
        rkbfirm-source = pkgs.releaseTools.sourceTarball {
          name = "rkbfirm-source";
          src = self;
          officialRelease = true;
          version = self.lastModifiedDate;
          nativeBuildInputs = [ pkgs.zstd ];
          distPhase = ''
            releaseName=rkb1-src-$version
            mkdir -p $out/tarballs
            mkdir ../$releaseName
            cp -prd . ../$releaseName
            (cd .. && tar -cf- $releaseName | zstd --ultra -22 > $out/tarballs/$releaseName.tar.zst) || false
          '';
        };
        rkbfirm-crate = (rustPkgs.workspace.rkbfirm { }).overrideAttrs(old: {
          configureCargo = "true";
        });
        rkbfirm = pkgs.stdenvNoCC.mkDerivation {
          pname = "rkbfirm";
          src = self;
          version = self.lastModifiedDate;
          nativeBuildInputs = with pkgs; [
            (elf2uf2-rs.overrideAttrs (old: {
              patches = [
                ./elf2uf2.patch
              ];
            }))
            zstd
          ];
          buildInputs = [rkbfirm-crate];
          buildPhase = ''
            elf2uf2-rs ${rkbfirm-crate}/bin/rkbfirm rkbfirm.uf2 --verbose
          '';
          installPhase = ''
            mkdir $out
            zstd --ultra -22 < rkbfirm.uf2 > $out/rkbfirm.uf2.zst
            mkdir $out/nix-support
            echo "file binary-dist $out/rkbfirm.uf2.zst" > $out/nix-support/hydra-build-products
            echo "$pname-$version" > $out/nix-support/hydra-release-name
          '';
        };
        default = rkbfirm;
      };
      nixosModules.default = import ./nixos {
        inherit inputs system;
      };
      hydraJobs = packages;
    });
}
