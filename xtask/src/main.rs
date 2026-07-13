//! Repository maintenance commands.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const REQUIRED_FIELDS: [&str; 15] = [
    "id",
    "title",
    "description",
    "dependencies",
    "milestone",
    "priority",
    "issue",
    "pull_request",
    "tests",
    "docs",
    "rust_api",
    "cli",
    "c_api",
    "cpp_api",
    "python_api",
];

const VALID_STATUSES: [&str; 8] = [
    "planned",
    "specified",
    "in_progress",
    "implemented",
    "tested",
    "documented",
    "integrated",
    "released",
];

#[derive(Debug)]
struct Requirement {
    line: usize,
    fields: BTreeMap<String, String>,
}

impl Requirement {
    fn value(&self, field: &str) -> &str {
        self.fields.get(field).map_or("", String::as_str)
    }

    fn id(&self) -> &str {
        self.value("id")
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run(mut args: impl Iterator<Item = String>) -> Result<String, String> {
    match (args.next().as_deref(), args.next().as_deref(), args.next()) {
        (Some("requirements"), Some("check"), None) => check_requirements(),
        _ => Err("usage: cargo xtask requirements check".to_owned()),
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask must be located directly under the workspace root".to_owned())
}

fn check_requirements() -> Result<String, String> {
    let root = workspace_root()?;
    let registry_path = root.join("requirements/v1.yaml");
    let source = fs::read_to_string(&registry_path)
        .map_err(|error| format!("failed to read {}: {error}", registry_path.display()))?;
    validate_registry_header(&source)?;
    let requirements = parse_requirements(&source)?;
    validate_requirements(&requirements)?;
    reject_production_placeholders(&root)?;
    Ok(format!(
        "requirements check passed: {} v1 requirements",
        requirements.len()
    ))
}

fn validate_registry_header(source: &str) -> Result<(), String> {
    for required_line in ["schema_version: 1", "target_version: \"1.0.0\""] {
        if !source.lines().any(|line| line.trim() == required_line) {
            return Err(format!(
                "requirements registry must contain `{required_line}`"
            ));
        }
    }
    Ok(())
}

fn parse_requirements(source: &str) -> Result<Vec<Requirement>, String> {
    let mut requirements = Vec::new();
    let mut current: Option<Requirement> = None;
    let mut in_requirements = false;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "requirements:" {
            in_requirements = true;
            continue;
        }
        if !in_requirements {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- id:") {
            if let Some(requirement) = current.take() {
                requirements.push(requirement);
            }
            let mut fields = BTreeMap::new();
            fields.insert("id".to_owned(), scalar(rest));
            current = Some(Requirement {
                line: line_number,
                fields,
            });
            continue;
        }
        let Some(requirement) = current.as_mut() else {
            return Err(format!(
                "line {line_number}: requirement fields must follow `- id:`"
            ));
        };
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!("line {line_number}: expected `key: value`"));
        };
        if requirement
            .fields
            .insert(key.to_owned(), scalar(value))
            .is_some()
        {
            return Err(format!(
                "line {line_number}: duplicate field `{key}` in {}",
                requirement.id()
            ));
        }
    }
    if let Some(requirement) = current {
        requirements.push(requirement);
    }
    if requirements.is_empty() {
        return Err("requirements registry contains no entries".to_owned());
    }
    Ok(requirements)
}

fn scalar(value: &str) -> String {
    value.trim().trim_matches('"').trim_matches('\'').to_owned()
}

fn validate_requirements(requirements: &[Requirement]) -> Result<(), String> {
    let mut failures = Vec::new();
    let ids = collect_requirement_ids(requirements, &mut failures);

    for requirement in requirements {
        validate_requirement_fields(requirement, &mut failures);
    }
    validate_dependencies(requirements, &ids, &mut failures);

    let cyclic_ids = dependency_cycle_members(requirements, &ids);
    if !cyclic_ids.is_empty() {
        failures.push(format!(
            "dependency graph contains a cycle among: {}",
            cyclic_ids.join(", ")
        ));
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn collect_requirement_ids(
    requirements: &[Requirement],
    failures: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for requirement in requirements {
        let id = requirement.id();
        if id.is_empty() {
            failures.push(format!("line {}: empty requirement id", requirement.line));
        } else if !ids.insert(id.to_owned()) {
            failures.push(format!(
                "line {}: duplicate requirement id `{id}`",
                requirement.line
            ));
        }
    }
    ids
}

fn validate_requirement_fields(requirement: &Requirement, failures: &mut Vec<String>) {
    let id = requirement.id();
    for field in REQUIRED_FIELDS.into_iter().chain(["benchmark", "status"]) {
        match requirement.fields.get(field) {
            None => failures.push(format!("{id}: missing required field `{field}`")),
            Some(value)
                if !matches!(field, "dependencies" | "issue" | "pull_request")
                    && is_null(value) =>
            {
                failures.push(format!("{id}: required field `{field}` is empty"));
            }
            Some(_) => {}
        }
    }
    if !matches!(requirement.value("priority"), "P0" | "P1" | "P2") {
        failures.push(format!("{id}: priority must be one of P0, P1, or P2"));
    }
    for field in ["issue", "pull_request"] {
        let value = requirement.value(field);
        if !is_null(value) && !valid_positive_integer(value) {
            failures.push(format!(
                "{id}: `{field}` must be null or a positive integer"
            ));
        }
    }
    for field in [
        "rust_api",
        "cli",
        "c_api",
        "cpp_api",
        "python_api",
        "benchmark",
    ] {
        let value = requirement.value(field);
        if !matches!(value, "planned" | "implemented") && !valid_not_applicable(value) {
            failures.push(format!(
                "{id}: `{field}` must be planned, implemented, or `N/A: reason`"
            ));
        }
    }

    let status = requirement.value("status");
    let Some(status_rank) = VALID_STATUSES.iter().position(|value| *value == status) else {
        failures.push(format!("{id}: invalid status `{status}`"));
        return;
    };
    if status_rank >= status_rank_of("in_progress") && is_null(requirement.value("issue")) {
        failures.push(format!("{id}: `{status}` requires an associated issue"));
    }
    if status_rank >= status_rank_of("implemented") && is_null(requirement.value("pull_request")) {
        failures.push(format!(
            "{id}: `{status}` requires an associated pull request"
        ));
    }
    if status_rank >= status_rank_of("tested") && is_empty_list(requirement.value("tests")) {
        failures.push(format!("{id}: `{status}` requires tests"));
    }
    if status_rank >= status_rank_of("documented") && is_empty_list(requirement.value("docs")) {
        failures.push(format!("{id}: `{status}` requires documentation"));
    }
    if status_rank >= status_rank_of("integrated") {
        validate_integrated_requirement(requirement, failures);
    }
}

fn validate_integrated_requirement(requirement: &Requirement, failures: &mut Vec<String>) {
    let id = requirement.id();
    for interface in ["rust_api", "cli", "c_api", "cpp_api", "python_api"] {
        let value = requirement.value(interface);
        if value != "implemented" && !valid_not_applicable(value) {
            failures.push(format!(
                "{id}: integrated interface `{interface}` must be implemented or `N/A: reason`"
            ));
        }
    }
    for forbidden in ["experimental", "partial", "deferred"] {
        if requirement
            .fields
            .values()
            .any(|value| value.to_ascii_lowercase().contains(forbidden))
        {
            failures.push(format!(
                "{id}: integrated v1 requirement contains forbidden marker `{forbidden}`"
            ));
        }
    }
}

fn validate_dependencies(
    requirements: &[Requirement],
    ids: &BTreeSet<String>,
    failures: &mut Vec<String>,
) {
    let status_by_id = requirements
        .iter()
        .filter_map(|requirement| {
            VALID_STATUSES
                .iter()
                .position(|value| *value == requirement.value("status"))
                .map(|rank| (requirement.id(), rank))
        })
        .collect::<BTreeMap<_, _>>();

    for requirement in requirements {
        for dependency in list_items(requirement.value("dependencies")) {
            if dependency == requirement.id() {
                failures.push(format!(
                    "{}: requirement depends on itself",
                    requirement.id()
                ));
            } else if !ids.contains(dependency) {
                failures.push(format!(
                    "{}: unknown dependency `{dependency}`",
                    requirement.id()
                ));
            } else if status_by_id
                .get(requirement.id())
                .is_some_and(|rank| *rank >= status_rank_of("in_progress"))
                && status_by_id
                    .get(dependency)
                    .is_some_and(|rank| *rank < status_rank_of("integrated"))
            {
                failures.push(format!(
                    "{}: active requirement has non-integrated dependency `{dependency}`",
                    requirement.id()
                ));
            }
        }
    }
}

fn valid_positive_integer(value: &str) -> bool {
    value.parse::<u64>().is_ok_and(|identifier| identifier > 0)
}

fn dependency_cycle_members(
    requirements: &[Requirement],
    known_ids: &BTreeSet<String>,
) -> Vec<String> {
    let mut remaining = requirements
        .iter()
        .map(|requirement| {
            let dependencies = list_items(requirement.value("dependencies"))
                .filter(|dependency| known_ids.contains(*dependency))
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            (requirement.id().to_owned(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();

    loop {
        let ready = remaining
            .iter()
            .filter(|(_, dependencies)| dependencies.is_empty())
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            break;
        }
        for id in &ready {
            remaining.remove(id);
        }
        for dependencies in remaining.values_mut() {
            for id in &ready {
                dependencies.remove(id);
            }
        }
    }

    remaining.into_keys().collect()
}

fn status_rank_of(status: &str) -> usize {
    VALID_STATUSES
        .iter()
        .position(|value| *value == status)
        .unwrap_or(usize::MAX)
}

fn is_null(value: &str) -> bool {
    matches!(value, "" | "null" | "~")
}

fn is_empty_list(value: &str) -> bool {
    is_null(value) || value == "[]"
}

fn valid_not_applicable(value: &str) -> bool {
    value
        .strip_prefix("N/A:")
        .is_some_and(|reason| !reason.trim().is_empty())
}

fn list_items(value: &str) -> impl Iterator<Item = &str> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('"').trim_matches('\''))
        .filter(|item| !item.is_empty())
}

fn reject_production_placeholders(root: &Path) -> Result<(), String> {
    let mut failures = Vec::new();
    for relative in ["crates", "xtask"] {
        scan_rust_sources(&root.join(relative), &mut failures)?;
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn scan_rust_sources(path: &Path, failures: &mut Vec<String>) -> Result<(), String> {
    for entry in
        fs::read_dir(path).map_err(|error| format!("failed to scan {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            scan_rust_sources(&entry_path, failures)?;
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            let source = fs::read_to_string(&entry_path)
                .map_err(|error| format!("failed to read {}: {error}", entry_path.display()))?;
            for token in [
                concat!("TO", "DO"),
                concat!("FIX", "ME"),
                concat!("to", "do!("),
                concat!("un", "implemented!("),
            ] {
                if source.contains(token) {
                    failures.push(format!(
                        "{}: production source contains forbidden placeholder `{token}`",
                        entry_path.display()
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{dependency_cycle_members, list_items, parse_requirements, scalar};

    #[test]
    fn parses_scalar_quotes() {
        assert_eq!(scalar(" \"value\" "), "value");
    }

    #[test]
    fn parses_inline_dependency_lists() {
        assert_eq!(
            list_items("[REQ-A, REQ-B]").collect::<Vec<_>>(),
            ["REQ-A", "REQ-B"]
        );
    }

    #[test]
    fn rejects_registry_without_requirements() {
        assert!(parse_requirements("schema_version: 1\nrequirements:\n").is_err());
    }

    #[test]
    fn detects_dependency_cycles() {
        let source = "requirements:\n  - id: REQ-A\n    dependencies: [REQ-B]\n  - id: REQ-B\n    dependencies: [REQ-A]\n";
        let requirements = parse_requirements(source);
        assert!(requirements.is_ok());
        let requirements = requirements.unwrap_or_default();
        let ids = requirements
            .iter()
            .map(|requirement| requirement.id().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            dependency_cycle_members(&requirements, &ids),
            ["REQ-A", "REQ-B"]
        );
    }
}
