{
  description = "pleme-io/multi-arch-image-release — combine per-arch OCI manifests into a single multi-arch tag";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crate2nix = { url = "github:nix-community/crate2nix"; inputs.nixpkgs.follows = "nixpkgs"; };
    flake-utils.url = "github:numtide/flake-utils";
    substrate = { url = "github:pleme-io/substrate"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs @ { self, nixpkgs, crate2nix, flake-utils, substrate, ... }:
    (import "${substrate}/lib/rust-action-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils;
    }) {
      toolName = "multi-arch-image-release";
      src = self;
      repo = "pleme-io/multi-arch-image-release";
      action = {
        description = "Combine N per-arch OCI manifests into a single multi-arch image tag using regctl. Optional additional-tags aliases the resulting digest under more tags (e.g. :latest + :vX.Y.Z). Lifts forge's image_release combine step.";
        inputs = [
          { name = "target-tag"; description = "Final multi-arch tag (full registry/repo:tag)"; required = true; }
          { name = "source-tags"; description = "Comma-separated source tags, one per architecture"; required = true; }
          { name = "additional-tags"; description = "Comma-separated additional tags to alias the digest under"; }
        ];
        outputs = [
          { name = "target-tag"; description = "Final multi-arch tag (echoed for downstream steps)"; }
          { name = "digest"; description = "Multi-arch manifest digest"; }
          { name = "alias-count"; description = "Number of aliases applied"; }
        ];
      };
    };
}
