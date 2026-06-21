# Releases

Release artifacts are built by the `Release` GitHub Actions workflow.

## Triggering a Release

Create and push a version tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds Linux amd64 artifacts for:

- `ssctl`
- `lwd-exporter`

It uploads archived binaries and a `SHA256SUMS` file to the GitHub release.

## Manual Builds

The workflow can also be run manually from GitHub Actions with `workflow_dispatch`. Manual runs upload workflow artifacts but only tag runs publish a GitHub release.
