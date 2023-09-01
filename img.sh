#!/usr/bin/env bash

set -euo pipefail

nix build .#serverImage --max-jobs 0

gzip --decompress --stdout --force <./result >result.tar

docker load <result.tar

docker push ghcr.io/pbar1/splitwise-sync-discord:latest
