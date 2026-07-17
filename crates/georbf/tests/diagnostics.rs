//! Independent tests for structured errors and source-aware diagnostics.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use georbf::{
    CancellationDiagnostic, CapabilityDiagnostic, ConditioningDiagnostic, ContrastDiagnostic,
    DiagnosticPath, DiagnosticPathError, DiagnosticTextField, DiagnosticValueError, ErrorCategory,
    ErrorCode, GaugeDiagnostic, GeoRbfError, InfeasibilityDiagnostic, InputDiagnostic, LevelId,
    MemoryDiagnostic, ObservationId, RankDiagnostic, SemanticProvenance, SourceLocation,
    VersionDiagnostic,
};

type TestResult = Result<(), Box<dyn Error>>;
type ErrorContract = (ErrorCode, ErrorCategory, u32, &'static str, &'static str);

const EXPECTED_ERROR_CONTRACTS: [ErrorContract; 10] = [
    (
        ErrorCode::InvalidInput,
        ErrorCategory::Input,
        1001,
        "GEORBF-E1001",
        "GEORBF-E1001 invalid input at inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=42 | group=section-a: kernel.length: must be finite and positive",
    ),
    (
        ErrorCode::CapabilityUnavailable,
        ErrorCategory::Capability,
        2001,
        "GEORBF-E2001",
        "GEORBF-E2001 capability unavailable at inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=42 | group=section-a: center Hessian",
    ),
    (
        ErrorCode::RankRejected,
        ErrorCategory::Rank,
        3001,
        "GEORBF-E3001",
        "GEORBF-E3001 rank rejected at inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=42 | group=section-a: estimated rank 4 is below required rank 5 for 8x5 system",
    ),
    (
        ErrorCode::MissingGauge,
        ErrorCategory::Gauge,
        4001,
        "GEORBF-E4001",
        "GEORBF-E4001 missing gauge at inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=42 | group=section-a: 2 connected component(s) remain ungauged",
    ),
    (
        ErrorCode::MissingContrast,
        ErrorCategory::Contrast,
        5001,
        "GEORBF-E5001",
        "GEORBF-E5001 missing contrast at inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=42 | group=section-a: levels 3 and 4 have no usable contrast",
    ),
    (
        ErrorCode::Infeasible,
        ErrorCategory::Infeasibility,
        6001,
        "GEORBF-E6001",
        "GEORBF-E6001 infeasible hard constraints: two hard equalities prescribe different values [sources: inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=42 | group=section-a, field=fields.stratigraphy | level=9]",
    ),
    (
        ErrorCode::IllConditioned,
        ErrorCategory::Conditioning,
        7001,
        "GEORBF-E7001",
        "GEORBF-E7001 condition limit exceeded: estimate 1000000000000 exceeds limit 10000000000",
    ),
    (
        ErrorCode::MemoryUnavailable,
        ErrorCategory::Memory,
        8001,
        "GEORBF-E8001",
        "GEORBF-E8001 memory unavailable: requested 4096 bytes exceeds limit 2048",
    ),
    (
        ErrorCode::Cancelled,
        ErrorCategory::Cancellation,
        9001,
        "GEORBF-E9001",
        "GEORBF-E9001 operation cancelled: dense factorization",
    ),
    (
        ErrorCode::VersionMismatch,
        ErrorCategory::Version,
        10_001,
        "GEORBF-E10001",
        "GEORBF-E10001 version mismatch: project schema expected 1.0, found 2.0",
    ),
];

fn provenance(identifier: u64) -> Result<SemanticProvenance, georbf::ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "inputs/project.yaml".to_owned(),
            NonZeroUsize::new(27).expect("positive source line"),
        )?,
        "m".to_owned(),
        "fields.stratigraphy.observations[4]".to_owned(),
        Some("section-a".to_owned()),
    )
}

#[test]
fn error_variants_have_stable_codes_and_structured_evidence() -> TestResult {
    let source = DiagnosticPath::try_observation(&provenance(42)?)?;
    let other_source =
        DiagnosticPath::try_level("fields.stratigraphy".to_owned(), LevelId::new(9))?;
    let errors = [
        GeoRbfError::Input {
            source: Some(source.clone()),
            diagnostic: InputDiagnostic::try_new(
                "kernel.length".to_owned(),
                "must be finite and positive".to_owned(),
            )?,
        },
        GeoRbfError::Capability {
            source: Some(source.clone()),
            diagnostic: CapabilityDiagnostic::try_new("center Hessian".to_owned())?,
        },
        GeoRbfError::Rank {
            source: Some(source.clone()),
            diagnostic: RankDiagnostic::try_new(8, 5, 4, 5)?,
        },
        GeoRbfError::Gauge {
            source: Some(source.clone()),
            diagnostic: GaugeDiagnostic::try_new(2)?,
        },
        GeoRbfError::Contrast {
            source: Some(source.clone()),
            diagnostic: ContrastDiagnostic::try_new(LevelId::new(3), LevelId::new(4))?,
        },
        GeoRbfError::Infeasible {
            diagnostic: InfeasibilityDiagnostic::try_new(
                vec![source.clone(), other_source],
                "two hard equalities prescribe different values".to_owned(),
            )?,
        },
        GeoRbfError::Conditioning {
            source: None,
            diagnostic: ConditioningDiagnostic::try_new(1.0e12, 1.0e10)?,
        },
        GeoRbfError::Memory {
            source: None,
            diagnostic: MemoryDiagnostic::try_limit_exceeded(4096, 2048)?,
        },
        GeoRbfError::Cancelled {
            source: None,
            diagnostic: CancellationDiagnostic::try_new("dense factorization".to_owned())?,
        },
        GeoRbfError::Version {
            source: None,
            diagnostic: VersionDiagnostic::try_new(
                "project schema".to_owned(),
                "1.0".to_owned(),
                "2.0".to_owned(),
            )?,
        },
    ];
    let mut identifiers = HashSet::new();
    for (error, (code, category, number, identifier, display)) in
        errors.iter().zip(EXPECTED_ERROR_CONTRACTS)
    {
        assert_eq!(error.code(), code);
        assert_eq!(error.code().category(), category);
        assert_eq!(error.code().number(), number);
        assert_eq!(error.code().identifier(), identifier);
        assert_eq!(error.to_string(), display);
        assert!(
            identifiers.insert(identifier),
            "duplicate identifier {identifier}"
        );
    }
    assert_eq!(
        errors[5]
            .primary_source()
            .and_then(DiagnosticPath::observation_id),
        Some(ObservationId::new(42))
    );
    assert_eq!(errors[5].related_sources().len(), 1);
    assert!(errors[0].to_string().contains("inputs/project.yaml:27"));
    Ok(())
}

#[test]
fn source_paths_preserve_optional_identifiers_independently() -> TestResult {
    struct Case {
        line: usize,
        field_path: &'static str,
        observation_id: Option<ObservationId>,
        level_id: Option<LevelId>,
        constraint_group: Option<&'static str>,
        display: &'static str,
    }

    let cases = [
        Case {
            line: 12,
            field_path: "kernel.length",
            observation_id: None,
            level_id: None,
            constraint_group: None,
            display: "inputs/project.yaml:12 | field=kernel.length",
        },
        Case {
            line: 18,
            field_path: "fields.stratigraphy.levels[17]",
            observation_id: None,
            level_id: Some(LevelId::new(17)),
            constraint_group: Some("section-a"),
            display: "inputs/project.yaml:18 | field=fields.stratigraphy.levels[17] | level=17 | group=section-a",
        },
    ];

    for case in cases {
        let source = SourceLocation::try_new(
            "inputs/project.yaml".to_owned(),
            NonZeroUsize::new(case.line).expect("positive source line"),
        )?;
        let path = DiagnosticPath::try_source(
            &source,
            case.field_path,
            case.observation_id,
            case.level_id,
            case.constraint_group,
        )?;

        assert_eq!(path.source_path(), Some("inputs/project.yaml"));
        assert_eq!(path.source_line().map(NonZeroUsize::get), Some(case.line));
        assert_eq!(path.field_path(), Some(case.field_path));
        assert_eq!(path.observation_id(), case.observation_id);
        assert_eq!(path.level_id(), case.level_id);
        assert_eq!(path.constraint_group(), case.constraint_group);
        assert_eq!(path.to_string(), case.display);
    }

    let source = SourceLocation::try_new(
        "inputs/project.yaml".to_owned(),
        NonZeroUsize::new(1).expect("positive source line"),
    )?;
    assert!(matches!(
        DiagnosticPath::try_source(&source, " ", None, None, None),
        Err(DiagnosticPathError::EmptyFieldPath)
    ));
    assert!(matches!(
        DiagnosticPath::try_source(&source, "kernel.length", None, None, Some(" ")),
        Err(DiagnosticPathError::EmptyConstraintGroup)
    ));
    Ok(())
}

#[test]
fn observation_level_paths_preserve_source_identifiers() -> TestResult {
    let path = DiagnosticPath::try_observation_at_level(&provenance(812)?, LevelId::new(17))?;
    assert_eq!(path.source_path(), Some("inputs/project.yaml"));
    assert_eq!(path.source_line().map(NonZeroUsize::get), Some(27));
    assert_eq!(
        path.field_path(),
        Some("fields.stratigraphy.observations[4]")
    );
    assert_eq!(path.observation_id(), Some(ObservationId::new(812)));
    assert_eq!(path.level_id(), Some(LevelId::new(17)));
    assert_eq!(path.constraint_group(), Some("section-a"));
    assert_eq!(
        path.to_string(),
        "inputs/project.yaml:27 | field=fields.stratigraphy.observations[4] | observation=812 | level=17 | group=section-a"
    );

    let level = DiagnosticPath::try_level(
        "fields.stratigraphy.levels[17]".to_owned(),
        LevelId::new(17),
    )?;
    assert_eq!(level.observation_id(), None);
    assert_eq!(level.level_id(), Some(LevelId::new(17)));
    assert_eq!(
        level.to_string(),
        "field=fields.stratigraphy.levels[17] | level=17"
    );
    assert_eq!(DiagnosticPath::global().to_string(), "<global>");
    Ok(())
}

#[test]
fn invalid_diagnostic_evidence_is_rejected_without_partial_success() {
    assert!(matches!(
        InputDiagnostic::try_new(" ".to_owned(), "reason".to_owned()),
        Err(DiagnosticValueError::EmptyText {
            field: DiagnosticTextField::InputField
        })
    ));
    assert!(matches!(
        RankDiagnostic::try_new(2, 2, 2, 2),
        Err(DiagnosticValueError::InvalidRank { .. })
    ));
    assert!(matches!(
        ConditioningDiagnostic::try_new(f64::INFINITY, 1.0e10),
        Err(DiagnosticValueError::InvalidConditioning { .. })
    ));
    assert!(matches!(
        MemoryDiagnostic::try_limit_exceeded(1024, 2048),
        Err(DiagnosticValueError::MemoryWithinLimit { .. })
    ));
    assert!(matches!(
        VersionDiagnostic::try_new("model".to_owned(), "1".to_owned(), "1".to_owned()),
        Err(DiagnosticValueError::IdenticalVersions)
    ));
}

#[test]
fn diagnostic_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<DiagnosticPath>();
    assert_send_sync::<GeoRbfError>();
}

#[test]
fn no_core_output() -> TestResult {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_no_output_macros(&source_root)?;
    Ok(())
}

fn assert_no_output_macros(path: &Path) -> TestResult {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            assert_no_output_macros(&path)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path)?;
            for forbidden in ["print!", "println!", "eprint!", "eprintln!", "dbg!"] {
                assert!(
                    !source.contains(forbidden),
                    "{} contains forbidden core output macro {forbidden}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}
