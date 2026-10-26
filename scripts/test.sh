#!/usr/bin/env bash
# set -euo pipefail

# cargo build-sbf --manifest-path programs/reflex/Cargo.toml --sbf-out-dir target/deploy
# anchor deploy --commitment processed --no-idl
anchor test --skip-local-validator --skip-build --skip-deploy
