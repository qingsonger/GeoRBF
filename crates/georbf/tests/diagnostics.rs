//! Independent tests for structured errors and source-aware diagnostics.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::error::Error;
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use georbf::{
    CancellationDiagnostic, CapabilityDiagnostic, ConditioningDiagnostic, ContrastDiagnostic,
    DiagnosticPath, DiagnosticTextField, DiagnosticValueError, ErrorCategory, ErrorCode,
    GaugeDiagnostic, GeoRbfError, InfeasibilityDiagnostic, InputDiagnostic, LevelId,
    MemoryDiagnostic, ObservationId, RankDiagnostic, SemanticProvenance, SourceLocation,
    VersionDiagnostic,
};

type TestResult = Result<(), Box<dyn Error>>;

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
    let expected = [
        (ErrorCode::InvalidInput, ErrorCategory::Input, 1001),
        (
            ErrorCode::CapabilityUnavailable,
            ErrorCategory::Capability,
            2001,
        ),
        (ErrorCode::RankRejected, ErrorCategory::Rank, 3001),
        (ErrorCode::MissingGauge, ErrorCategory::Gauge, 4001),
        (ErrorCode::MissingContrast, ErrorCategory::Contrast, 5001),
        (ErrorCode::Infeasible, ErrorCategory::Infeasibility, 6001),
        (ErrorCode::IllConditioned, ErrorCategory::Conditioning, 7001),
        (ErrorCode::MemoryUnavailable, ErrorCategory::Memory, 8001),
        (ErrorCode::Cancelled, ErrorCategory::Cancellation, 9001),
        (ErrorCode::VersionMismatch, ErrorCategory::Version, 10_001),
    ];

    for (error, (code, category, number)) in errors.iter().zip(expected) {
        assert_eq!(error.code(), code);
        assert_eq!(error.code().category(), category);
        assert_eq!(error.code().number(), number);
        assert!(error.code().identifier().starts_with("GEORBF-E"));
        assert!(error.to_string().starts_with(error.code().identifier()));
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
