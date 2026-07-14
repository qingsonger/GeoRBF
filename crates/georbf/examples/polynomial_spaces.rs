//! Generate and evaluate the complete polynomial side space for CPD order 3.

use std::error::Error;

use georbf::{Point, PolynomialSpace};

fn main() -> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<2>::try_new(3)?;
    let point = Point::try_new([2.0, 3.0])?;
    let mut values = vec![0.0; space.term_count()];
    let mut gradients = vec![[0.0; 2]; space.term_count()];
    space.try_evaluate(point, &mut values, &mut gradients)?;

    println!(
        "CPD order {}: {} complete polynomial terms",
        space.cpd_order().get(),
        space.term_count()
    );
    for ((term, value), gradient) in space.terms().iter().zip(values).zip(gradients) {
        println!(
            "alpha={:?}, value={value}, gradient={gradient:?}",
            term.exponents()
        );
    }
    Ok(())
}
