{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  # https://discourse.nixos.org/t/cross-compiling-docker-images-with-flakes/25716/2
  outputs = { self, nixpkgs }:
    let
      pkgsLinux = nixpkgs.legacyPackages.x86_64-linux;

      user = "pbar1";
      repo = "splitwise-sync";
      imageSource = "https://github.com/${user}/${repo}";
      imageName = "ghcr.io/${user}/${repo}";

      # FIXME: I think this only worked because "server" was the only member of the
      # Cargo workspace
      server = pkgsLinux.rustPlatform.buildRustPackage {
        pname = "server";
        version = "0.0.0";
        src = ./.;
        cargoSha256 = "sha256-0aDqhezM6ZqxMqogBtVmODXjPaCj7+kgrEGZ2kmgi+M=";
      };

      # FIXME: Had to run this:
      # gzip --decompress --stdout --force < ./result > result.tar
      serverImage = pkgsLinux.dockerTools.buildLayeredImage {
        name = "${imageName}-discord";
        tag = "latest";
        config = {
          Cmd = [ "${server}/bin/server" ];
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
