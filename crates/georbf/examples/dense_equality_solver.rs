//! Solve one analytic SPD equality system with explicit numerical policy.

use std::error::Error;

use georbf::{
    ConditionPolicy, DenseEqualitySystem, DenseFactorization, DenseSolveOptions, Regularization,
};

fn main() -> Result<(), Box<dyn Error>> {
    let system =
        DenseEqualitySystem::try_from_row_major(2, vec![4.0, 1.0, 1.0, 3.0], vec![6.0, 7.0])?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        4,
    )?;
    let solution = system.try_solve(options)?;

    println!("solution = {:?}", solution.values());
    println!(
        "condition = {:.6e}, original residual = {:.6e}",
        solution.diagnostics().effective_rank.condition_estimate,
        solution.diagnostics().final_residual.original_infinity
    );
    Ok(())
}
