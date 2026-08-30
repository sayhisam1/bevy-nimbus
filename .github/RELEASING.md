# Publishing a release

The release workflow publishes every Nimbus workspace crate in dependency
order. It skips versions already present on crates.io, so a failed run can be
restarted safely.

1. Give every workspace package the same new version and update `Cargo.lock`.
2. Ensure CI passes on `main`, including every package-size check.
3. Create a GitHub environment named `crates-io`.
4. Add its `CARGO_REGISTRY_TOKEN` secret and protect it with required reviewers.
5. Publish a GitHub release tagged `v<workspace-version>`.

The workflow can also be started manually with the exact workspace version.
