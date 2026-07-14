//! Repository maintenance commands.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const REQUIRED_FIELDS: [&str; 17] = [
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
    "benchmark",
    "status",
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
    let arguments = args.by_ref().collect::<Vec<_>>();
    match arguments.as_slice() {
        [area, action] if area == "requirements" && action == "check" => check_requirements(),
        [area, action] if area == "requirements" && action == "next" => next_requirement(),
        [area, action, requirement_id]
            if area == "requirements" && action == "show" =>
        {
            show_requirement(requirement_id)
        }
        [area, action, requirement_id]
            if area == "requirements" && action == "deps" =>
        {
            show_dependency_closure(requirement_id)
        }
        _ => Err(
            "usage:\n  cargo xtask requirements check\n  cargo xtask requirements next\n  cargo xtask requirements show <REQ-ID>\n  cargo xtask requirements deps <REQ-ID>"
                .to_owned(),
        ),
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask must be located directly under the workspace root".to_owned())
}

fn check_requirements() -> Result<String, String> {
    let (root, requirements) = load_validated_requirements()?;
    reject_production_placeholders(&root)?;
    reject_publishable_prerelease_packages(&root)?;
    Ok(format!(
        "requirements check passed: {} v1 requirements",
        requirements.len()
    ))
}

fn load_validated_requirements() -> Result<(PathBuf, Vec<Requirement>), String> {
    let root = workspace_root()?;
    let registry_path = root.join("requirements/v1.yaml");
    let source = fs::read_to_string(&registry_path)
        .map_err(|error| format!("failed to read {}: {error}", registry_path.display()))?;
    validate_registry_header(&source)?;
    let requirements = parse_requirements(&source)?;
    validate_requirements(&requirements)?;
    Ok((root, requirements))
}

fn next_requirement() -> Result<String, String> {
    let (_, requirements) = load_validated_requirements()?;
    let requirement = next_eligible_requirement(&requirements)
        .ok_or_else(|| "no eligible unfinished v1 requirement was found".to_owned())?;
    Ok(format_requirement_summary(requirement, &requirements))
}

fn show_requirement(requirement_id: &str) -> Result<String, String> {
    let (_, requirements) = load_validated_requirements()?;
    let requirement = find_requirement(&requirements, requirement_id)?;
    Ok(format_requirement_summary(requirement, &requirements))
}

fn show_dependency_closure(requirement_id: &str) -> Result<String, String> {
    let (_, requirements) = load_validated_requirements()?;
    find_requirement(&requirements, requirement_id)?;
    let closure = dependency_closure_ids(requirement_id, &requirements)?;
    if closure.is_empty() {
        return Ok(format!("{requirement_id} dependency closure: none"));
    }

    let mut lines = vec![format!("{requirement_id} dependency closure:")];
    for dependency_id in closure {
        let dependency = find_requirement(&requirements, &dependency_id)?;
        lines.push(format!(
            "- {} [{}] {}",
            dependency.id(),
            dependency.value("status"),
            dependency.value("title")
        ));
    }
    Ok(lines.join("\n"))
}

fn find_requirement<'a>(
    requirements: &'a [Requirement],
    requirement_id: &str,
) -> Result<&'a Requirement, String> {
    requirements
        .iter()
        .find(|requirement| requirement.id() == requirement_id)
        .ok_or_else(|| format!("unknown requirement `{requirement_id}`"))
}

fn format_requirement_summary(requirement: &Requirement, requirements: &[Requirement]) -> String {
    let status_by_id = requirements
        .iter()
        .map(|candidate| (candidate.id(), candidate.value("status")))
        .collect::<BTreeMap<_, _>>();
    let dependencies = list_items(requirement.value("dependencies"))
        .map(|dependency| {
            let status = status_by_id.get(dependency).copied().unwrap_or("unknown");
            format!("{dependency} [{status}]")
        })
        .collect::<Vec<_>>();
    let issue = display_optional_identifier(requirement.value("issue"));
    let pull_request = display_optional_identifier(requirement.value("pull_request"));

    [
        format!(
            "{} | {} | {} | {}",
            requirement.id(),
            requirement.value("status"),
            requirement.value("milestone"),
            requirement.value("priority")
        ),
        format!("title: {}", requirement.value("title")),
        format!("description: {}", requirement.value("description")),
        format!("issue: {issue}; pull request: {pull_request}"),
        format!(
            "dependencies: {}",
            if dependencies.is_empty() {
                "none".to_owned()
            } else {
                dependencies.join(", ")
            }
        ),
        format!(
            "tests: {}",
            list_items(requirement.value("tests"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "docs: {}",
            list_items(requirement.value("docs"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "interfaces: rust={}; cli={}; c={}; cpp={}; python={}; benchmark={}",
            requirement.value("rust_api"),
            requirement.value("cli"),
            requirement.value("c_api"),
            requirement.value("cpp_api"),
            requirement.value("python_api"),
            requirement.value("benchmark")
        ),
    ]
    .join("\n")
}

fn display_optional_identifier(value: &str) -> &str {
    if is_null(value) { "none" } else { value }
}

fn next_eligible_requirement(requirements: &[Requirement]) -> Option<&Requirement> {
    let status_by_id = requirements
        .iter()
        .map(|requirement| (requirement.id(), requirement.value("status")))
        .collect::<BTreeMap<_, _>>();

    let ordering_key = |index: usize, requirement: &Requirement| {
        (
            milestone_rank(requirement.value("milestone")),
            priority_rank(requirement.value("priority")),
            index,
        )
    };

    let active = requirements
        .iter()
        .enumerate()
        .filter(|(_, requirement)| {
            let rank = status_rank_of(requirement.value("status"));
            rank >= status_rank_of("in_progress") && rank < status_rank_of("integrated")
        })
        .min_by_key(|(index, requirement)| ordering_key(*index, requirement))
        .map(|(_, requirement)| requirement);
    if active.is_some() {
        return active;
    }

    requirements
        .iter()
        .enumerate()
        .filter(|(_, requirement)| {
            let rank = status_rank_of(requirement.value("status"));
            rank < status_rank_of("in_progress")
                && list_items(requirement.value("dependencies")).all(|dependency| {
                    status_by_id.get(dependency).is_some_and(|status| {
                        status_rank_of(status) >= status_rank_of("integrated")
                    })
                })
        })
        .min_by_key(|(index, requirement)| ordering_key(*index, requirement))
        .map(|(_, requirement)| requirement)
}

fn milestone_rank(milestone: &str) -> u32 {
    milestone
        .strip_prefix('M')
        .and_then(|value| value.split_once('-').map(|(number, _)| number))
        .and_then(|number| number.parse::<u32>().ok())
        .unwrap_or(u32::MAX)
}

fn priority_rank(priority: &str) -> u8 {
    match priority {
        "P0" => 0,
        "P1" => 1,
        "P2" => 2,
        _ => u8::MAX,
    }
}

fn dependency_closure_ids(
    requirement_id: &str,
    requirements: &[Requirement],
) -> Result<Vec<String>, String> {
    let by_id = requirements
        .iter()
        .map(|requirement| (requirement.id(), requirement))
        .collect::<BTreeMap<_, _>>();
    let mut visited = BTreeSet::new();
    let mut closure = Vec::new();
    visit_dependencies(requirement_id, &by_id, &mut visited, &mut closure)?;
    Ok(closure)
}

fn visit_dependencies(
    requirement_id: &str,
    by_id: &BTreeMap<&str, &Requirement>,
    visited: &mut BTreeSet<String>,
    closure: &mut Vec<String>,
) -> Result<(), String> {
    let requirement = by_id
        .get(requirement_id)
        .ok_or_else(|| format!("unknown requirement `{requirement_id}`"))?;
    for dependency in list_items(requirement.value("dependencies")) {
        if visited.insert(dependency.to_owned()) {
            visit_dependencies(dependency, by_id, visited, closure)?;
            closure.push(dependency.to_owned());
        }
    }
    Ok(())
}

fn validate_registry_header(source: &str) -> Result<(), String> {
    const HEADERS: [&str; 4] = [
        "schema_version: 1",
        "target_version: \"1.0.0\"",
        "source_of_truth: true",
        "requirements:",
    ];
    for required_line in HEADERS {
        let count = source.lines().filter(|line| *line == required_line).count();
        if count != 1 {
            return Err(format!(
                "requirements registry must contain exactly one top-level `{required_line}`"
            ));
        }
    }
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if line == "requirements:" {
            break;
        }
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !HEADERS.contains(&line) {
            return Err(format!(
                "line {}: unknown or misindented registry header `{trimmed}`",
                index + 1
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
        if raw_line == "requirements:" {
            in_requirements = true;
            continue;
        }
        if !in_requirements {
            continue;
        }
        if let Some(rest) = raw_line.strip_prefix("  - id:") {
            if !valid_scalar_quotes(rest) {
                return Err(format!(
                    "line {line_number}: requirement id has mismatched quotes"
                ));
            }
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
        if !raw_line.starts_with("    ") {
            return Err(format!(
                "line {line_number}: requirement fields must use four-space indentation"
            ));
        }
        let Some(requirement) = current.as_mut() else {
            return Err(format!(
                "line {line_number}: requirement fields must follow `- id:`"
            ));
        };
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!("line {line_number}: expected `key: value`"));
        };
        if !valid_scalar_quotes(value) {
            return Err(format!("line {line_number}: `{key}` has mismatched quotes"));
        }
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

fn valid_scalar_quotes(value: &str) -> bool {
    let value = value.trim();
    match (value.chars().next(), value.chars().last()) {
        (Some(first @ ('"' | '\'')), Some(last)) => first == last && value.len() >= 2,
        _ => true,
    }
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
    for field in requirement.fields.keys() {
        if !REQUIRED_FIELDS.contains(&field.as_str()) {
            failures.push(format!("{id}: unknown field `{field}`"));
        }
    }
    for field in REQUIRED_FIELDS {
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
    if !valid_requirement_id(id) {
        failures.push(format!(
            "line {}: invalid requirement id `{id}`",
            requirement.line
        ));
    }
    for field in ["dependencies", "tests", "docs"] {
        let value = requirement.value(field);
        if !valid_inline_list(value) {
            failures.push(format!(
                "{id}: `{field}` must be a bracketed inline list without empty items"
            ));
        } else if matches!(field, "tests" | "docs") && is_empty_list(value) {
            failures.push(format!("{id}: `{field}` must not be empty"));
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
    for obligation in [
        "rust_api",
        "cli",
        "c_api",
        "cpp_api",
        "python_api",
        "benchmark",
    ] {
        let value = requirement.value(obligation);
        if value != "implemented" && !valid_not_applicable(value) {
            failures.push(format!(
                "{id}: integrated obligation `{obligation}` must be implemented or `N/A: reason`"
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

fn valid_requirement_id(value: &str) -> bool {
    let Some(body) = value.strip_prefix("REQ-") else {
        return false;
    };
    let Some((name, sequence)) = body.rsplit_once('-') else {
        return false;
    };
    name.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    }) && valid_positive_integer(sequence)
}

fn valid_inline_list(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    inner.trim().is_empty() || inner.split(',').all(|item| !item.trim().is_empty())
}

fn dependency_cycle_members(
    requirements: &[Requirement],
    known_ids: &BTreeSet<String>,
) -> Vec<String> {
    let graph = requirements
        .iter()
        .map(|requirement| {
            let dependencies = list_items(requirement.value("dependencies"))
                .filter(|dependency| known_ids.contains(*dependency))
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            (requirement.id().to_owned(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();

    graph
        .keys()
        .filter(|candidate| {
            let mut visited = BTreeSet::new();
            let mut pending = graph
                .get(*candidate)
                .into_iter()
                .flat_map(BTreeSet::iter)
                .cloned()
                .collect::<Vec<_>>();
            while let Some(current) = pending.pop() {
                if current == **candidate {
                    return true;
                }
                if visited.insert(current.clone()) {
                    pending.extend(
                        graph
                            .get(&current)
                            .into_iter()
                            .flat_map(BTreeSet::iter)
                            .cloned(),
                    );
                }
            }
            false
        })
        .cloned()
        .collect()
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

fn reject_publishable_prerelease_packages(root: &Path) -> Result<(), String> {
    for relative in [
        "crates/georbf/Cargo.toml",
        "crates/georbf-cli/Cargo.toml",
        "crates/georbf-ffi/Cargo.toml",
        "crates/georbf-python/Cargo.toml",
        "xtask/Cargo.toml",
    ] {
        let path = root.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        if !manifest_disables_publication(&source) {
            return Err(format!(
                "{}: prerelease workspace package must set `publish = false`",
                path.display()
            ));
        }
    }
    Ok(())
}

fn manifest_disables_publication(source: &str) -> bool {
    let mut in_package = false;
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package && line == "publish = false" {
            return true;
        }
    }
    false
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
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
    };

    use super::{
        Requirement, dependency_closure_ids, dependency_cycle_members, format_requirement_summary,
        list_items, manifest_disables_publication, next_eligible_requirement, parse_requirements,
        scalar, validate_registry_header, validate_requirements, workspace_root,
    };

    fn valid_requirement() -> Requirement {
        let fields = [
            ("id", "REQ-TEST-001"),
            ("title", "Test requirement"),
            ("description", "Exercise registry validation"),
            ("dependencies", "[]"),
            ("milestone", "M0-v0.0.1"),
            ("priority", "P0"),
            ("issue", "1"),
            ("pull_request", "2"),
            ("tests", "[unit]"),
            ("docs", "[test-doc]"),
            ("rust_api", "N/A: test-only requirement"),
            ("cli", "N/A: test-only requirement"),
            ("c_api", "N/A: test-only requirement"),
            ("cpp_api", "N/A: test-only requirement"),
            ("python_api", "N/A: test-only requirement"),
            ("benchmark", "N/A: test-only requirement"),
            ("status", "documented"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect::<BTreeMap<_, _>>();
        Requirement { line: 1, fields }
    }

    fn requirement_with(
        id: &str,
        status: &str,
        dependencies: &str,
        milestone: &str,
        priority: &str,
    ) -> Requirement {
        let mut requirement = valid_requirement();
        for (field, value) in [
            ("id", id),
            ("status", status),
            ("dependencies", dependencies),
            ("milestone", milestone),
            ("priority", priority),
        ] {
            requirement
                .fields
                .insert(field.to_owned(), value.to_owned());
        }
        requirement
    }

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
    fn requires_unique_top_level_registry_headers() {
        let source =
            "schema_version: 1\ntarget_version: \"1.0.0\"\nsource_of_truth: true\nrequirements:\n";
        assert!(validate_registry_header(source).is_ok());
        assert!(validate_registry_header(&format!("{source}source_of_truth: true\n")).is_err());
        let unknown = source.replace("requirements:\n", "owner: GeoRBF\nrequirements:\n");
        assert!(validate_registry_header(&unknown).is_err());
    }

    #[test]
    fn rejects_misindented_requirement_fields() {
        let source = "requirements:\n  - id: REQ-A-001\n  dependencies: []\n";
        assert!(parse_requirements(source).is_err());
    }

    #[test]
    fn rejects_mismatched_scalar_quotes() {
        let source = "requirements:\n  - id: \"REQ-A-001'\n    dependencies: []\n";
        assert!(parse_requirements(source).is_err());
    }

    #[test]
    fn prerelease_manifests_must_disable_publication() {
        assert!(manifest_disables_publication(
            "[package]\nname = \"georbf\"\npublish = false\n"
        ));
        assert!(!manifest_disables_publication(
            "[package]\nname = \"georbf\"\n"
        ));
        assert!(!manifest_disables_publication(
            "[package]\nname = \"georbf\"\n[package.metadata]\npublish = false\n"
        ));
    }

    #[test]
    fn rejects_unknown_fields_and_malformed_lists() {
        let mut requirement = valid_requirement();
        requirement
            .fields
            .insert("unknown".to_owned(), "value".to_owned());
        requirement
            .fields
            .insert("tests".to_owned(), "unit".to_owned());
        let result = validate_requirements(&[requirement]);
        assert!(result.as_ref().is_err_and(|message| {
            message.contains("unknown field") && message.contains("bracketed inline list")
        }));
    }

    #[test]
    fn integrated_requirements_cannot_defer_benchmarks() {
        let mut requirement = valid_requirement();
        requirement
            .fields
            .insert("status".to_owned(), "integrated".to_owned());
        requirement
            .fields
            .insert("benchmark".to_owned(), "planned".to_owned());
        let result = validate_requirements(&[requirement]);
        assert!(
            result
                .as_ref()
                .is_err_and(|message| { message.contains("integrated obligation `benchmark`") })
        );
    }

    #[test]
    fn detects_dependency_cycles() {
        let source = "requirements:\n  - id: REQ-A\n    dependencies: [REQ-B]\n  - id: REQ-B\n    dependencies: [REQ-A]\n  - id: REQ-C\n    dependencies: [REQ-A]\n";
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

    #[test]
    fn compact_summary_includes_dependency_status_and_required_context() {
        let requirements = [
            requirement_with("REQ-BASE-001", "integrated", "[]", "M0-v0.0.1", "P0"),
            requirement_with(
                "REQ-NEXT-001",
                "planned",
                "[REQ-BASE-001]",
                "M1-v0.1.0",
                "P0",
            ),
        ];
        let summary = format_requirement_summary(&requirements[1], &requirements);
        assert!(summary.contains("REQ-NEXT-001 | planned | M1-v0.1.0 | P0"));
        assert!(summary.contains("dependencies: REQ-BASE-001 [integrated]"));
        assert!(summary.contains("tests: unit"));
        assert!(summary.contains("docs: test-doc"));
    }

    #[test]
    fn next_requirement_prefers_active_work_then_eligible_milestone_priority() {
        let mut requirements = vec![
            requirement_with("REQ-BASE-001", "integrated", "[]", "M0-v0.0.1", "P0"),
            requirement_with(
                "REQ-LATER-001",
                "planned",
                "[REQ-BASE-001]",
                "M2-v0.2.0",
                "P1",
            ),
            requirement_with(
                "REQ-NEXT-001",
                "planned",
                "[REQ-BASE-001]",
                "M2-v0.2.0",
                "P0",
            ),
        ];
        assert_eq!(
            next_eligible_requirement(&requirements).map(Requirement::id),
            Some("REQ-NEXT-001")
        );

        requirements.push(requirement_with(
            "REQ-ACTIVE-001",
            "in_progress",
            "[REQ-BASE-001]",
            "M3-v0.3.0",
            "P1",
        ));
        assert_eq!(
            next_eligible_requirement(&requirements).map(Requirement::id),
            Some("REQ-ACTIVE-001")
        );
    }

    #[test]
    fn dependency_closure_is_transitive_and_dependencies_first() {
        let requirements = [
            requirement_with("REQ-BASE-001", "integrated", "[]", "M0-v0.0.1", "P0"),
            requirement_with(
                "REQ-MIDDLE-001",
                "integrated",
                "[REQ-BASE-001]",
                "M1-v0.1.0",
                "P0",
            ),
            requirement_with(
                "REQ-TOP-001",
                "planned",
                "[REQ-MIDDLE-001]",
                "M2-v0.2.0",
                "P0",
            ),
        ];
        assert_eq!(
            dependency_closure_ids("REQ-TOP-001", &requirements),
            Ok(vec!["REQ-BASE-001".to_owned(), "REQ-MIDDLE-001".to_owned()])
        );
    }

    #[test]
    fn workflow_docs_require_green_ready_head_ci_before_one_merge() {
        let root = workspace_root();
        assert!(
            root.is_ok(),
            "workspace root should resolve during tests: {root:?}"
        );
        let root = root.unwrap_or_default();
        let required_sequence = "mark the PR ready -> wait for the complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact ready head -> merge exactly once only when that CI is green -> record truthful integration state";

        for relative_path in ["AGENTS.md", "docs/CODEX_WORKFLOW.md"] {
            let source = fs::read_to_string(root.join(relative_path));
            assert!(source.is_ok(), "failed to read {relative_path}: {source:?}");
            let source = source.unwrap_or_default();
            let normalized = source.split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(
                normalized.contains(required_sequence),
                "{relative_path} must state the canonical ready-head CI sequence"
            );
            assert!(
                !normalized.contains("mark the PR ready, merge it"),
                "{relative_path} must not permit a merge before ready-head CI"
            );
        }
    }
}
