//! Assemble CPD polynomial actions and expand side-condition-safe weights.

use std::error::Error;

use georbf::{
    CenterRepresenter, CpdNullSpace, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, Point, PolynomialSpace,
};

fn value_center(x: f64, identifier: u64) -> Result<CenterRepresenter<1>, Box<dyn Error>> {
    let atom = FunctionalAtom::value(Point::try_new([x])?, FunctionalProvenance::new(identifier));
    let term = FunctionalTerm::try_new(1.0, atom)?;
    Ok(CenterRepresenter::new(FunctionalExpr::try_new([term])?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let centers = [
        value_center(-1.0, 100)?,
        value_center(0.0, 101)?,
        value_center(1.0, 102)?,
    ];
    let space = PolynomialSpace::<1>::try_new(2)?;
    let side_condition = CpdNullSpace::try_from_centers(&centers, &space)?;
    let weights = side_condition.try_expand_weights(&[2.0])?;

    println!(
        "Q is {}x{}, nullity={}, rank={}, Q^T w residual={:.3e}",
        side_condition.actions().rows(),
        side_condition.actions().columns(),
        side_condition.basis().columns(),
        side_condition.diagnostics().svd_rank,
        weights.side_condition_residual()
    );
    Ok(())
}
