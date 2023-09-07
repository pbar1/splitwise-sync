{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-filter.url = "github:numtide/nix-filter";
  };

  # https://discourse.nixos.org/t/cross-compiling-docker-images-with-flakes/25716/2
  outputs = { self, nixpkgs, nix-filter }:
    let
      pkgsLinux = nixpkgs.legacyPackages.x86_64-linux;
      filter = nix-filter.lib;

      user = "pbar1";
      repo = "splitwise-sync"; # NOTE: Should match what's in Cargo.toml
      imageSource = "https://github.com/${user}/${repo}";
      imageName = "ghcr.io/${user}/${repo}";

      # FIXME: I think this only worked because "splitwise-sync" was the only
      # member of the Cargo workspace
      server = pkgsLinux.rustPlatform.buildRustPackage {
        pname = repo;
        version = "0.0.0";
        src =  filter {
          root = ./.;
          include = [
            "Cargo.lock"
            "Cargo.toml"
            "server"
          ];
        };
        cargoSha256 = "sha256-dtYPICESudz6Sc/hsihGfyHrTxs8eFpiMYN6f19cw58=";
      };

      # FIXME: Had to run this:
      # gzip --decompress --stdout --force < ./result > result.tar
      serverImage = pkgsLinux.dockerTools.buildLayeredImage {
        name = "${imageName}";
        tag = "latest";
        contents = [ pkgsLinux.dockerTools.caCertificates ];
        config = {
          Entrypoint = [ "${server}/bin/${repo}" ];
          Labels = {
            "org.opencontainers.image.authors" = user;
            "org.opencontainers.image.source" = imageSource;
          };
        };
      };
    in
    {
      packages.aarch64-darwin = {
        inherit server serverImage;
      };
    };
}
