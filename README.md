# pleme-io/multi-arch-image-release

Combine per-arch OCI manifests into a single multi-arch image tag using `regctl`. Optional aliases for `:latest` / `:vX.Y.Z` etc.

```yaml
- uses: pleme-io/multi-arch-image-release@v1
  with:
    target-tag: ghcr.io/pleme-io/my-tool:${{ github.sha }}
    source-tags: |
      ghcr.io/pleme-io/my-tool:amd64-${{ github.sha }},
      ghcr.io/pleme-io/my-tool:arm64-${{ github.sha }}
    additional-tags: |
      ghcr.io/pleme-io/my-tool:latest,
      ghcr.io/pleme-io/my-tool:${{ github.ref_name }}
```

## Inputs

| Name | Required | Description |
|---|---|---|
| `target-tag` | yes | Final multi-arch tag |
| `source-tags` | yes | Comma-separated per-arch source tags |
| `additional-tags` | no | Comma-separated alias tags |

## Outputs

| Name | Description |
|---|---|
| `target-tag` | Echoed for downstream steps |
| `digest` | Multi-arch manifest digest |
| `alias-count` | Number of aliases applied |

## Prerequisites

`regctl` must be on `PATH` in the runner — typically installed by the substrate toolchain or via:

```yaml
- uses: regclient/actions/regctl-installer@main
```
