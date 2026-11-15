//! Rust-native fail-closed API-only integration contract. Never handles secrets.
use std::{fs, path::Path, process::ExitCode};
const REQUIRED: &[&str] = &[
    "api- and sdk-only",
    "official sdk",
    "shared-auth is the sole cross-product authentication and assurance boundary",
    "audience `cliptown-api`",
    "authorized party `memebank-api`",
    "cliptown:memebank:read",
    "cliptown:memebank:write",
    "cliptown:memebank:delete",
    "never requires both mobile apps to be installed",
    "native clipboard export remains a separate user feature",
    "ciphertext-only payload boundary",
    "does not discover a local application",
];
const FORBIDDEN: &[&str] = &[
    "future direct local bridge", "local user-mediated handoff",
    "integrate through explicit clipboard writes",
    "integration transport is the clipboard",
    "memebank://", "cliptown://", "canopenurl", "canlaunchurl",
];
const CONTRACT_REQUIRED: &[&str] = &[
    "must not", "probe whether memebank or cliptown is installed",
    "use local ipc, loopback ports", "fall back to clipboard handoff",
    "memebank never calls a 3fa backend", "write and delete operations",
    "idempotency-key/body mismatch", "no mobile applications installed",
    "contract preview rather than a deployable integration",
];

fn validate_text(readme: &str, contract: &str) -> Result<(), String> {
    let combined = format!("{}\n{}", readme.to_lowercase(), contract.to_lowercase());
    let contract = contract.to_lowercase();
    for phrase in REQUIRED {
        if !combined.contains(phrase) {
            return Err(format!("missing required API-only invariant: {phrase}"));
        }
    }
    for phrase in FORBIDDEN {
        if combined.contains(phrase) {
            return Err(format!("forbidden local-app coupling: {phrase}"));
        }
    }
    for phrase in CONTRACT_REQUIRED {
        if !contract.contains(phrase) {
            return Err(format!("missing integration contract invariant: {phrase}"));
        }
    }
    Ok(())
}

fn validate_repository(root: &Path) -> Result<(), String> {
    let readme = fs::read_to_string(root.join("README.md"))
        .map_err(|_| "cannot read README.md")?;
    let contract = fs::read_to_string(root.join("docs/memebank-integration.md"))
        .map_err(|_| "cannot read docs/memebank-integration.md")?;
    validate_text(&readme, &contract)
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let root = arguments.next().unwrap_or_else(|| ".".to_owned());
    if arguments.next().is_some() {
        eprintln!("usage: cliptown-memebank-boundary [repository-root]");
        return ExitCode::from(2);
    }
    match validate_repository(Path::new(&root)) {
        Ok(()) => {
            println!("MemeBank API/SDK-only boundary validated by Rust");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("MemeBank boundary: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_fixture() -> (String, String) {
        (REQUIRED.join(" "), CONTRACT_REQUIRED.join(" "))
    }

    #[test]
    fn complete_contract_is_accepted() {
        let (readme, contract) = valid_fixture();
        assert!(validate_text(&readme, &contract).is_ok());
    }

    #[test]
    fn missing_required_api_only_invariant_is_rejected() {
        let (readme, contract) = valid_fixture();
        let readme = readme.replace("official sdk", "");
        assert!(validate_text(&readme, &contract).is_err());
    }

    #[test]
    fn local_app_deep_links_and_clipboard_fallback_are_rejected() {
        for marker in FORBIDDEN {
            let (readme, contract) = valid_fixture();
            assert!(validate_text(&format!("{readme} {marker}"), &contract).is_err());
        }
    }

    #[test]
    fn contract_only_requirement_cannot_be_satisfied_by_readme() {
        let (readme, contract) = valid_fixture();
        let phrase = "no mobile applications installed";
        assert!(validate_text(
            &format!("{readme} {phrase}"),
            &contract.replace(phrase, ""),
        ).is_err());
    }

    #[test]
    fn mixed_case_inputs_are_checked_equivalently() {
        let (readme, contract) = valid_fixture();
        assert!(validate_text(&readme.to_uppercase(), &contract.to_uppercase()).is_ok());
    }
}
