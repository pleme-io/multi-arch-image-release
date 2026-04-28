//! `pleme-io/multi-arch-image-release` — combine per-arch OCI manifests
//! into a single multi-arch image tag.
//!
//! Lifts forge's `commands/image_release.rs` regctl combine step into a
//! standalone action. Inputs: a base tag + a comma-separated list of
//! per-arch source tags (e.g. `<repo>:amd64-<sha>,<repo>:arm64-<sha>`);
//! output: the canonical multi-arch tag (`<repo>:<sha>`) and the
//! resulting manifest digest.
//!
//! Implementation uses `regctl image manifest combine` — same tool the
//! forge command uses, present in the substrate's image-release toolchain.

use std::process::{Command, Stdio};

use pleme_actions_shared::{ActionError, Input, Output, StepSummary};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Inputs {
    /// Final multi-arch tag, e.g. `ghcr.io/pleme-io/my-tool:abc123` or
    /// `ghcr.io/pleme-io/my-tool:v1.2.3`.
    target_tag: String,
    /// Comma-separated source tags, one per architecture. Each must
    /// already exist in the registry (built + pushed by upstream steps).
    source_tags: String,
    /// Optional additional tags to alias to the same digest after combine.
    /// Common pattern: combine into `:<sha>`, then alias `:latest` and
    /// `:v1.2.3`. Comma-separated.
    #[serde(default)]
    additional_tags: Option<String>,
}

fn main() {
    pleme_actions_shared::log::init();
    if let Err(e) = run() {
        e.emit_to_stdout();
        if e.is_fatal() {
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), ActionError> {
    let inputs = Input::<Inputs>::from_env()?;
    let sources: Vec<&str> = inputs.source_tags.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if sources.is_empty() {
        return Err(ActionError::error("input `source-tags` must list at least one tag"));
    }

    // Combine via regctl
    let mut args: Vec<String> = vec![
        "image".into(),
        "manifest".into(),
        "combine".into(),
        "--ref-tag".into(),
        inputs.target_tag.clone(),
    ];
    for src in &sources {
        args.push((*src).to_string());
    }
    run_command("regctl", &args)?;

    // Capture digest of the combined tag
    let digest = run_capture("regctl", &["image", "digest", &inputs.target_tag])?;
    let digest = digest.trim().to_string();

    // Alias additional tags
    let aliases: Vec<&str> = inputs
        .additional_tags
        .as_deref()
        .map(|s| s.split(',').map(str::trim).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default();
    for alias in &aliases {
        run_command(
            "regctl",
            &[
                "image".to_string(),
                "copy".to_string(),
                inputs.target_tag.clone(),
                (*alias).to_string(),
            ],
        )?;
    }

    let output = Output::from_runner_env()?;
    output.set("target-tag", &inputs.target_tag)?;
    output.set("digest", &digest)?;
    output.set("alias-count", aliases.len().to_string())?;

    let mut summary = StepSummary::from_runner_env()?;
    let mut rows = vec![
        vec!["target-tag".to_string(), inputs.target_tag.clone()],
        vec!["digest".into(), digest.clone()],
        vec!["sources".into(), sources.join(", ")],
    ];
    if !aliases.is_empty() {
        rows.push(vec!["aliases".into(), aliases.join(", ")]);
    }
    summary
        .heading(2, "multi-arch-image-release")
        .table(&["Field", "Value"], rows);
    summary.commit()?;

    Ok(())
}

fn run_command(program: &str, args: &[String]) -> Result<(), ActionError> {
    let status = Command::new(program)
        .args(args.iter().map(String::as_str))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| ActionError::error(format!("failed to spawn `{program}`: {e}")))?;
    if !status.success() {
        return Err(ActionError::error(format!(
            "`{program}` exited with status {status}"
        )));
    }
    Ok(())
}

fn run_capture(program: &str, args: &[&str]) -> Result<String, ActionError> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| ActionError::error(format!("failed to spawn `{program}`: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(ActionError::error(format!(
            "`{program}` exited with status {} (stderr: {})",
            output.status,
            stderr.trim()
        )));
    }
    Ok(stdout.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_tags_is_error() {
        // Build the inputs struct via deserialize from a fake env
        let inputs = Inputs {
            target_tag: "ghcr.io/x/y:tag".into(),
            source_tags: "  ,,  ".into(),
            additional_tags: None,
        };
        // We can't easily call run() here without mocking the registry;
        // smoke-test the parsing logic by reproducing it inline.
        let sources: Vec<&str> = inputs.source_tags.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        assert!(sources.is_empty());
    }

    #[test]
    fn source_tags_split_correctly() {
        let s = "ghcr.io/x/y:amd64-abc, ghcr.io/x/y:arm64-abc";
        let sources: Vec<&str> = s.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        assert_eq!(sources, vec!["ghcr.io/x/y:amd64-abc", "ghcr.io/x/y:arm64-abc"]);
    }

    #[test]
    fn additional_tags_handle_trailing_commas() {
        let s = "ghcr.io/x/y:latest,ghcr.io/x/y:v1.2.3,";
        let aliases: Vec<&str> = s.split(',').map(str::trim).filter(|t| !t.is_empty()).collect();
        assert_eq!(aliases, vec!["ghcr.io/x/y:latest", "ghcr.io/x/y:v1.2.3"]);
    }
}
