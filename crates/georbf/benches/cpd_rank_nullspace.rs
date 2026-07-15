//! Deterministic CPD rank and null-space construction workload.

use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

use georbf::{
    CenterRepresenter, CpdNullSpace, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, Point, PolynomialSpace,
};

fn value_center<const D: usize>(
    coordinates: [f64; D],
    identifier: u64,
) -> Result<CenterRepresenter<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let point = Point::try_new(coordinates)?;
    let atom = FunctionalAtom::value(point, FunctionalProvenance::new(identifier));
    let term = FunctionalTerm::try_new(1.0, atom)?;
    Ok(CenterRepresenter::new(FunctionalExpr::try_new([term])?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut centers = Vec::new();
    for index in 0_u32..64 {
        let phase = f64::from(index);
        centers.push(value_center(
            [phase.sin(), phase.cos(), (phase * 0.37).sin()],
            u64::from(index),
        )?);
    }
    let space = PolynomialSpace::<3>::try_new(2)?;
    let started = Instant::now();
    let iterations = if std::env::args().any(|argument| argument == "--smoke") {
        1
    } else {
        100
    };
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let system = black_box(CpdNullSpace::try_from_centers(
            black_box(&centers),
            black_box(&space),
        )?);
        checksum += system.basis().values().iter().sum::<f64>();
    }
    println!(
        "cpd-rank-nullspace,rows=64,columns=4,iterations={iterations},elapsed_ns={},checksum={checksum:.17e}",
        started.elapsed().as_nanos()
    );
    Ok(())
}
