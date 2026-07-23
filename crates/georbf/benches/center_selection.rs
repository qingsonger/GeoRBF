//! Deterministic production center-selection benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{CenterSelectionOptions, CenterSelectionProblem, CenterSelectionStrategy, Point};

const MEMORY_LIMIT: usize = 512 * 1024 * 1024;

fn fixture(count: usize) -> Result<CenterSelectionProblem<1>, Box<dyn Error>> {
    let mut locations = Vec::new();
    let mut gram = Vec::new();
    let mut targets = Vec::new();
    locations.try_reserve_exact(count)?;
    gram.try_reserve_exact(count.checked_mul(count).ok_or("Gram shape overflow")?)?;
    targets.try_reserve_exact(count)?;
    for index in 0..count {
        let coordinate = f64::from(u32::try_from(index)?);
        locations.push(Point::try_new([coordinate])?);
        targets.push((coordinate * 0.173).sin() + 0.25 * (coordinate * 0.071).cos());
    }
    for row in 0..count {
        for column in 0..count {
            let separation = f64::from(u32::try_from(row)?) - f64::from(u32::try_from(column)?);
            gram.push((-0.5 * separation * separation).exp());
        }
    }
    Ok(CenterSelectionProblem::try_from_row_major(
        locations, gram, targets,
    )?)
}

fn options(strategy: CenterSelectionStrategy) -> Result<CenterSelectionOptions, Box<dyn Error>> {
    Ok(CenterSelectionOptions::new(
        strategy,
        NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
    ))
}

fn run(
    problem: &CenterSelectionProblem<1>,
    options: &CenterSelectionOptions,
    iterations: u32,
) -> Result<(f64, f64), Box<dyn Error>> {
    let mut checksum = 0.0_f64;
    let started = Instant::now();
    for _ in 0..iterations {
        let selection = black_box(problem).try_select(black_box(options))?;
        checksum += selection
            .indices()
            .iter()
            .map(|&index| f64::from(u32::try_from(index).unwrap_or(u32::MAX)))
            .sum::<f64>();
        checksum += selection.diagnostics().rank.condition_estimate;
    }
    let nanoseconds = started.elapsed().as_secs_f64() * 1.0e9 / f64::from(iterations);
    Ok((nanoseconds, checksum))
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let candidates = if smoke { 48 } else { 160 };
    let selected = if smoke { 12 } else { 48 };
    let iterations = if smoke { 1 } else { 3 };
    let problem = fixture(candidates)?;
    let selected_count = NonZeroUsize::new(selected).ok_or("selected count")?;
    let strategies = [
        ("all-representers", CenterSelectionStrategy::AllRepresenters),
        (
            "user-provided",
            CenterSelectionStrategy::UserProvided((0..selected).collect()),
        ),
        (
            "farthest-point",
            CenterSelectionStrategy::FarthestPoint {
                count: selected_count,
                seed: 0x5eed,
            },
        ),
        (
            "residual-greedy",
            CenterSelectionStrategy::ResidualGreedy {
                count: selected_count,
                seed: 0x5eed,
            },
        ),
        (
            "power-greedy",
            CenterSelectionStrategy::PowerGreedy {
                count: selected_count,
                seed: 0x5eed,
            },
        ),
    ];

    println!("strategy,candidates,selected,nanoseconds,checksum");
    for (name, strategy) in strategies {
        let requested = if matches!(strategy, CenterSelectionStrategy::AllRepresenters) {
            candidates
        } else {
            selected
        };
        let (nanoseconds, checksum) = run(&problem, &options(strategy)?, iterations)?;
        println!("{name},{candidates},{requested},{nanoseconds:.2},{checksum:.17e}");
    }
    Ok(())
}
