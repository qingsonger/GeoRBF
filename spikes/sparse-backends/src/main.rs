use std::hint::black_box;
use std::time::{Duration, Instant};

#[cfg(not(any(feature = "faer-backend", feature = "sprs-backend")))]
compile_error!("enable at least one sparse factorization backend feature");

#[cfg(not(any(feature = "kiddo-index", feature = "rstar-index")))]
compile_error!("enable at least one spatial index feature");

#[cfg(feature = "faer-backend")]
use faer::prelude::Solve;

const DIMENSION: usize = 3;
const SUPPORT_RADIUS: f64 = 1.75;
const ACCEPTED_BACKWARD_ERROR: f64 = 1.0e-10;
const BENCHMARK_HEADER: &str =
    "kind,phase,candidate,points,nonzeros_or_pairs,iterations,nanoseconds,checksum,residual";
const INDEX_BENCHMARK_PHASE: &str = "query_filter_canonicalize_checksum_end_to_end";
const SOLVER_BENCHMARK_PHASE: &str = "construct_factor_solve_review_checksum_end_to_end";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpatialIndex {
    #[cfg(feature = "kiddo-index")]
    Kiddo,
    #[cfg(feature = "rstar-index")]
    Rstar,
}

impl SpatialIndex {
    const ALL: &'static [Self] = &[
        #[cfg(feature = "kiddo-index")]
        Self::Kiddo,
        #[cfg(feature = "rstar-index")]
        Self::Rstar,
    ];

    const fn label(self) -> &'static str {
        match self {
            #[cfg(feature = "kiddo-index")]
            Self::Kiddo => "kiddo-5.3.2",
            #[cfg(feature = "rstar-index")]
            Self::Rstar => "rstar-0.13.0",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SparseBackend {
    #[cfg(feature = "faer-backend")]
    Faer,
    #[cfg(feature = "sprs-backend")]
    Sprs,
}

impl SparseBackend {
    const ALL: &'static [Self] = &[
        #[cfg(feature = "faer-backend")]
        Self::Faer,
        #[cfg(feature = "sprs-backend")]
        Self::Sprs,
    ];

    const fn label(self) -> &'static str {
        match self {
            #[cfg(feature = "faer-backend")]
            Self::Faer => "faer-0.24.4",
            #[cfg(feature = "sprs-backend")]
            Self::Sprs => "sprs-0.11.4+sprs-ldl-0.10.0",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NeighborPair {
    row: usize,
    column: usize,
    distance_squared: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SparseEntry {
    row: usize,
    column: usize,
    value: f64,
}

#[derive(Clone, Debug)]
struct SparseCase {
    dimension: usize,
    entries: Vec<SparseEntry>,
    right_hand_side: Vec<f64>,
    truth: Vec<f64>,
}

impl SparseCase {
    fn new(
        dimension: usize,
        entries: Vec<SparseEntry>,
        right_hand_side: Vec<f64>,
        truth: Vec<f64>,
    ) -> Result<Self, String> {
        if dimension == 0 {
            return Err("sparse system must be nonempty".to_owned());
        }
        if right_hand_side.len() != dimension || truth.len() != dimension {
            return Err("sparse system vectors do not match its dimension".to_owned());
        }
        if right_hand_side
            .iter()
            .chain(&truth)
            .any(|value| !value.is_finite())
        {
            return Err("sparse system vectors must be finite".to_owned());
        }
        if entries.is_empty() {
            return Err("sparse system must contain stored entries".to_owned());
        }
        let mut previous = None;
        for entry in &entries {
            if entry.row >= dimension || entry.column >= dimension {
                return Err("sparse entry index is out of bounds".to_owned());
            }
            if !entry.value.is_finite() || entry.value == 0.0 {
                return Err("stored sparse entries must be finite and nonzero".to_owned());
            }
            let key = (entry.row, entry.column);
            if previous.is_some_and(|previous| previous >= key) {
                return Err("sparse entries must be strictly row-major ordered".to_owned());
            }
            previous = Some(key);
        }
        Ok(Self {
            dimension,
            entries,
            right_hand_side,
            truth,
        })
    }

    fn from_points(points: &[[f64; DIMENSION]], radius: f64) -> Result<Self, String> {
        let pairs = brute_force_pairs(points, radius)?;
        let entries = assemble_wendland_c2(&pairs, radius)?;
        let dimension = points.len();
        let truth = truth_vector(dimension)?;
        let right_hand_side = sparse_matrix_vector_product(dimension, &entries, &truth)?;
        Self::new(dimension, entries, right_hand_side, truth)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SolveReport {
    solution: Vec<f64>,
    stored_nonzeros: usize,
    residual_infinity: f64,
    backward_error: f64,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct CscInspection {
    rows: usize,
    columns: usize,
    column_pointers: Vec<usize>,
    row_indices: Vec<usize>,
    values: Vec<f64>,
    product: Vec<f64>,
    solution: Vec<f64>,
}

fn validate_points_and_radius(points: &[[f64; DIMENSION]], radius: f64) -> Result<f64, String> {
    if points.is_empty() {
        return Err("point collection must be nonempty".to_owned());
    }
    if points.iter().flatten().any(|value| !value.is_finite()) {
        return Err("point coordinates must be finite".to_owned());
    }
    if !radius.is_finite() || radius <= 0.0 {
        return Err("support radius must be positive and finite".to_owned());
    }
    let radius_squared = radius * radius;
    if !radius_squared.is_finite() || radius_squared == 0.0 {
        return Err("squared support radius must be representable".to_owned());
    }
    Ok(radius_squared)
}

fn squared_distance(left: &[f64; DIMENSION], right: &[f64; DIMENSION]) -> Result<f64, String> {
    let mut scale = 0.0_f64;
    let mut sum = 1.0_f64;
    for axis in 0..DIMENSION {
        let difference = left[axis] - right[axis];
        if !difference.is_finite() {
            return Err("point separation is nonrepresentable".to_owned());
        }
        let absolute = difference.abs();
        if absolute != 0.0 {
            if scale < absolute {
                let ratio = scale / absolute;
                sum = ratio.mul_add(ratio * sum, 1.0);
                scale = absolute;
            } else {
                let ratio = absolute / scale;
                sum = ratio.mul_add(ratio, sum);
            }
        }
    }
    let distance_squared = if scale == 0.0 {
        0.0
    } else {
        (scale * scale) * sum
    };
    if !distance_squared.is_finite() {
        return Err("squared point separation is nonrepresentable".to_owned());
    }
    Ok(distance_squared)
}

fn brute_force_pairs(
    points: &[[f64; DIMENSION]],
    radius: f64,
) -> Result<Vec<NeighborPair>, String> {
    let radius_squared = validate_points_and_radius(points, radius)?;
    let mut pairs = Vec::new();
    for row in 0..points.len() {
        for column in row..points.len() {
            let distance_squared = squared_distance(&points[row], &points[column])?;
            if distance_squared < radius_squared {
                pairs.push(NeighborPair {
                    row,
                    column,
                    distance_squared,
                });
            }
        }
    }
    Ok(pairs)
}

fn indexed_pairs(
    points: &[[f64; DIMENSION]],
    radius: f64,
    index: SpatialIndex,
) -> Result<Vec<NeighborPair>, String> {
    let radius_squared = validate_points_and_radius(points, radius)?;
    let mut pairs = match index {
        #[cfg(feature = "kiddo-index")]
        SpatialIndex::Kiddo => kiddo_pairs(points, radius_squared)?,
        #[cfg(feature = "rstar-index")]
        SpatialIndex::Rstar => rstar_pairs(points, radius_squared)?,
    };
    pairs.sort_by_key(|pair| (pair.row, pair.column));
    pairs.dedup_by_key(|pair| (pair.row, pair.column));
    Ok(pairs)
}

#[cfg(feature = "kiddo-index")]
fn kiddo_pairs(
    points: &[[f64; DIMENSION]],
    radius_squared: f64,
) -> Result<Vec<NeighborPair>, String> {
    use kiddo::{SquaredEuclidean, float::kdtree::KdTree};

    let mut tree: KdTree<f64, u64, DIMENSION, 128, u32> = KdTree::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        let item =
            u64::try_from(index).map_err(|_| "point index does not fit in u64".to_owned())?;
        tree.add(point, item);
    }
    let mut pairs = Vec::new();
    for (row, point) in points.iter().enumerate() {
        for neighbor in tree.within::<SquaredEuclidean>(point, radius_squared) {
            let column = usize::try_from(neighbor.item)
                .map_err(|_| "kiddo item index does not fit in usize".to_owned())?;
            if column >= row {
                let distance_squared = squared_distance(point, &points[column])?;
                if distance_squared < radius_squared {
                    pairs.push(NeighborPair {
                        row,
                        column,
                        distance_squared,
                    });
                }
            }
        }
    }
    Ok(pairs)
}

#[cfg(feature = "rstar-index")]
fn rstar_pairs(
    points: &[[f64; DIMENSION]],
    radius_squared: f64,
) -> Result<Vec<NeighborPair>, String> {
    use rstar::{RTree, primitives::GeomWithData};

    let items = points
        .iter()
        .copied()
        .enumerate()
        .map(|(index, point)| GeomWithData::new(point, index))
        .collect();
    let tree = RTree::bulk_load(items);
    let mut pairs = Vec::new();
    for (row, point) in points.iter().enumerate() {
        for neighbor in tree.locate_within_distance(*point, radius_squared) {
            let column = neighbor.data;
            if column >= row {
                let distance_squared = squared_distance(point, &points[column])?;
                if distance_squared < radius_squared {
                    pairs.push(NeighborPair {
                        row,
                        column,
                        distance_squared,
                    });
                }
            }
        }
    }
    Ok(pairs)
}

fn wendland_c2(distance_squared: f64, radius: f64) -> Result<f64, String> {
    if !distance_squared.is_finite() || distance_squared < 0.0 {
        return Err("kernel distance must be finite and nonnegative".to_owned());
    }
    let distance = distance_squared.sqrt();
    if !distance.is_finite() {
        return Err("kernel distance is nonrepresentable".to_owned());
    }
    let normalized = distance / radius;
    if !normalized.is_finite() {
        return Err("normalized kernel distance is nonrepresentable".to_owned());
    }
    if normalized >= 1.0 {
        return Ok(0.0);
    }
    let taper = 1.0 - normalized;
    let taper_squared = taper * taper;
    let value = taper_squared * taper_squared * normalized.mul_add(4.0, 1.0);
    if !value.is_finite() || value <= 0.0 {
        return Err("interior Wendland C2 value is not positive and finite".to_owned());
    }
    Ok(value)
}

fn assemble_wendland_c2(pairs: &[NeighborPair], radius: f64) -> Result<Vec<SparseEntry>, String> {
    if pairs.is_empty() {
        return Err("neighbor pair collection must be nonempty".to_owned());
    }
    let mut entries = Vec::with_capacity(pairs.len().saturating_mul(2));
    for pair in pairs {
        let value = wendland_c2(pair.distance_squared, radius)?;
        if value == 0.0 {
            continue;
        }
        entries.push(SparseEntry {
            row: pair.row,
            column: pair.column,
            value,
        });
        if pair.row != pair.column {
            entries.push(SparseEntry {
                row: pair.column,
                column: pair.row,
                value,
            });
        }
    }
    entries.sort_by_key(|entry| (entry.row, entry.column));
    if entries
        .windows(2)
        .any(|window| (window[0].row, window[0].column) >= (window[1].row, window[1].column))
    {
        return Err("assembled sparse entries are duplicated or unordered".to_owned());
    }
    Ok(entries)
}

fn truth_vector(dimension: usize) -> Result<Vec<f64>, String> {
    (0..dimension)
        .map(|index| {
            let ordinal = u32::try_from(index + 1)
                .map_err(|_| "system dimension exceeds deterministic fixture range".to_owned())?;
            let value = f64::from(ordinal).mul_add(0.125, 0.25).sin();
            if value.is_finite() {
                Ok(value)
            } else {
                Err("truth vector is nonfinite".to_owned())
            }
        })
        .collect()
}

fn sparse_matrix_vector_product(
    dimension: usize,
    entries: &[SparseEntry],
    vector: &[f64],
) -> Result<Vec<f64>, String> {
    if vector.len() != dimension {
        return Err("matrix-vector shape mismatch".to_owned());
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err("matrix-vector input must be finite".to_owned());
    }
    let mut product = vec![0.0; dimension];
    for entry in entries {
        if entry.row >= dimension || entry.column >= dimension || !entry.value.is_finite() {
            return Err("invalid sparse entry in matrix-vector product".to_owned());
        }
        product[entry.row] = entry
            .value
            .mul_add(vector[entry.column], product[entry.row]);
        if !product[entry.row].is_finite() {
            return Err("matrix-vector product is nonrepresentable".to_owned());
        }
    }
    Ok(product)
}

fn infinity_norm(values: &[f64], label: &str) -> Result<f64, String> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(format!("{label} contains a nonfinite entry"));
    }
    let norm = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !norm.is_finite() {
        return Err(format!("{label} infinity norm is nonfinite"));
    }
    Ok(norm)
}

fn matrix_infinity_norm(case: &SparseCase) -> Result<f64, String> {
    let mut row_sums = vec![0.0_f64; case.dimension];
    for entry in &case.entries {
        row_sums[entry.row] += entry.value.abs();
        if !row_sums[entry.row].is_finite() {
            return Err("matrix infinity norm has a nonfinite row sum".to_owned());
        }
    }
    infinity_norm(&row_sums, "matrix row sums")
}

fn residual_metrics(case: &SparseCase, solution: &[f64]) -> Result<(f64, f64), String> {
    if solution.len() != case.dimension {
        return Err("solution shape does not match sparse system".to_owned());
    }
    let product = sparse_matrix_vector_product(case.dimension, &case.entries, solution)?;
    let residual = case
        .right_hand_side
        .iter()
        .zip(product)
        .map(|(right_hand_side, value)| right_hand_side - value)
        .collect::<Vec<_>>();
    let residual_infinity = infinity_norm(&residual, "residual")?;
    let denominator = matrix_infinity_norm(case)?.mul_add(
        infinity_norm(solution, "solution")?,
        infinity_norm(&case.right_hand_side, "right-hand side")?,
    );
    if !denominator.is_finite() {
        return Err("backward-error denominator is nonfinite".to_owned());
    }
    let backward_error = if denominator == 0.0 {
        residual_infinity
    } else {
        residual_infinity / denominator
    };
    if !backward_error.is_finite() {
        return Err("backward error is nonfinite".to_owned());
    }
    Ok((residual_infinity, backward_error))
}

fn solve(case: &SparseCase, backend: SparseBackend) -> Result<SolveReport, String> {
    let (solution, stored_nonzeros) = match backend {
        #[cfg(feature = "faer-backend")]
        SparseBackend::Faer => solve_faer(case)?,
        #[cfg(feature = "sprs-backend")]
        SparseBackend::Sprs => solve_sprs(case)?,
    };
    let (residual_infinity, backward_error) = residual_metrics(case, &solution)?;
    if backward_error > ACCEPTED_BACKWARD_ERROR {
        return Err(format!(
            "{} failed original-unit residual review: {backward_error:.17e}",
            backend.label()
        ));
    }
    for (index, (&actual, &expected)) in solution.iter().zip(&case.truth).enumerate() {
        if (actual - expected).abs() > 1.0e-8 * expected.abs().max(1.0) {
            return Err(format!(
                "{} failed analytic truth at solution[{index}]",
                backend.label()
            ));
        }
    }
    Ok(SolveReport {
        solution,
        stored_nonzeros,
        residual_infinity,
        backward_error,
    })
}

#[cfg(feature = "faer-backend")]
fn solve_faer(case: &SparseCase) -> Result<(Vec<f64>, usize), String> {
    use faer::{
        Side,
        sparse::{SparseColMat, Triplet},
    };

    let triplets = case
        .entries
        .iter()
        .map(|entry| Triplet::new(entry.row, entry.column, entry.value))
        .collect::<Vec<_>>();
    let matrix = SparseColMat::<usize, f64>::try_new_from_triplets(
        case.dimension,
        case.dimension,
        &triplets,
    )
    .map_err(|error| format!("faer CSC construction failed: {error:?}"))?;
    let stored_nonzeros = matrix.compute_nnz();
    let factor = matrix
        .sp_cholesky(Side::Lower)
        .map_err(|error| format!("faer sparse Cholesky rejected system: {error:?}"))?;
    let right_hand_side = faer::Col::from_fn(case.dimension, |row| case.right_hand_side[row]);
    let solution = factor.solve(&right_hand_side);
    Ok((
        (0..case.dimension).map(|index| solution[index]).collect(),
        stored_nonzeros,
    ))
}

#[cfg(feature = "sprs-backend")]
fn solve_sprs(case: &SparseCase) -> Result<(Vec<f64>, usize), String> {
    let mut triplets =
        sprs::TriMat::with_capacity((case.dimension, case.dimension), case.entries.len());
    for entry in &case.entries {
        triplets.add_triplet(entry.row, entry.column, entry.value);
    }
    let matrix: sprs::CsMat<f64> = triplets.to_csc();
    let stored_nonzeros = matrix.nnz();
    let factor = sprs_ldl::LdlNumeric::new(matrix.view())
        .map_err(|error| format!("sprs LDL rejected system: {error:?}"))?;
    Ok((factor.solve(&case.right_hand_side), stored_nonzeros))
}

#[cfg(all(test, feature = "faer-backend"))]
fn inspect_faer_csc(case: &SparseCase, vector: &[f64]) -> Result<CscInspection, String> {
    use faer::{
        Side,
        sparse::{SparseColMat, Triplet},
    };

    let triplets = case
        .entries
        .iter()
        .map(|entry| Triplet::new(entry.row, entry.column, entry.value))
        .collect::<Vec<_>>();
    let matrix = SparseColMat::<usize, f64>::try_new_from_triplets(
        case.dimension,
        case.dimension,
        &triplets,
    )
    .map_err(|error| format!("faer CSC construction failed: {error:?}"))?;
    let column_pointers = matrix.col_ptr().to_vec();
    let row_indices = matrix.row_idx().to_vec();
    let values = matrix.val().to_vec();
    let mut product = vec![0.0; matrix.nrows()];
    for column in 0..matrix.ncols() {
        for position in column_pointers[column]..column_pointers[column + 1] {
            let row = row_indices[position];
            product[row] = values[position].mul_add(vector[column], product[row]);
        }
    }
    let factor = matrix
        .sp_cholesky(Side::Lower)
        .map_err(|error| format!("faer sparse Cholesky rejected system: {error:?}"))?;
    let right_hand_side = faer::Col::from_fn(case.dimension, |row| case.right_hand_side[row]);
    let solution = factor.solve(&right_hand_side);
    Ok(CscInspection {
        rows: matrix.nrows(),
        columns: matrix.ncols(),
        column_pointers,
        row_indices,
        values,
        product,
        solution: (0..case.dimension).map(|index| solution[index]).collect(),
    })
}

#[cfg(all(test, feature = "sprs-backend"))]
fn inspect_sprs_csc(case: &SparseCase, vector: &[f64]) -> Result<CscInspection, String> {
    let mut triplets =
        sprs::TriMat::with_capacity((case.dimension, case.dimension), case.entries.len());
    for entry in &case.entries {
        triplets.add_triplet(entry.row, entry.column, entry.value);
    }
    let matrix: sprs::CsMat<f64> = triplets.to_csc();
    let column_pointers = matrix.indptr().raw_storage().to_vec();
    let row_indices = matrix.indices().to_vec();
    let values = matrix.data().to_vec();
    let mut product = vec![0.0; matrix.rows()];
    for column in 0..matrix.cols() {
        for position in column_pointers[column]..column_pointers[column + 1] {
            let row = row_indices[position];
            product[row] = values[position].mul_add(vector[column], product[row]);
        }
    }
    let factor = sprs_ldl::LdlNumeric::new(matrix.view())
        .map_err(|error| format!("sprs LDL rejected system: {error:?}"))?;
    Ok(CscInspection {
        rows: matrix.rows(),
        columns: matrix.cols(),
        column_pointers,
        row_indices,
        values,
        product,
        solution: factor.solve(&case.right_hand_side),
    })
}

#[cfg(test)]
fn inspect_backend_csc(
    case: &SparseCase,
    vector: &[f64],
    backend: SparseBackend,
) -> Result<CscInspection, String> {
    match backend {
        #[cfg(feature = "faer-backend")]
        SparseBackend::Faer => inspect_faer_csc(case, vector),
        #[cfg(feature = "sprs-backend")]
        SparseBackend::Sprs => inspect_sprs_csc(case, vector),
    }
}

fn grid_points(side: usize) -> Result<Vec<[f64; DIMENSION]>, String> {
    if side == 0 {
        return Err("grid side must be nonzero".to_owned());
    }
    let capacity = side
        .checked_mul(side)
        .and_then(|value| value.checked_mul(side))
        .ok_or_else(|| "grid size overflows usize".to_owned())?;
    let mut points = Vec::with_capacity(capacity);
    for z in 0..side {
        for y in 0..side {
            for x in 0..side {
                let x = u32::try_from(x)
                    .map_err(|_| "grid coordinate exceeds fixture range".to_owned())?;
                let y = u32::try_from(y)
                    .map_err(|_| "grid coordinate exceeds fixture range".to_owned())?;
                let z = u32::try_from(z)
                    .map_err(|_| "grid coordinate exceeds fixture range".to_owned())?;
                points.push([f64::from(x), f64::from(y), f64::from(z)]);
            }
        }
    }
    Ok(points)
}

fn pair_checksum(pairs: &[NeighborPair]) -> Result<f64, String> {
    let mut checksum = 0.0;
    for pair in pairs {
        let row = u32::try_from(pair.row + 1)
            .map_err(|_| "pair row exceeds checksum range".to_owned())?;
        let column = u32::try_from(pair.column + 1)
            .map_err(|_| "pair column exceeds checksum range".to_owned())?;
        checksum +=
            f64::from(row).mul_add(0.5, f64::from(column).mul_add(0.25, pair.distance_squared));
    }
    if !checksum.is_finite() {
        return Err("pair checksum is nonfinite".to_owned());
    }
    Ok(checksum)
}

fn time_index(
    points: &[[f64; DIMENSION]],
    index: SpatialIndex,
    iterations: usize,
) -> Result<(Duration, usize, f64), String> {
    let started = Instant::now();
    let mut last_pairs = None;
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let pairs = black_box(indexed_pairs(
            black_box(points),
            black_box(SUPPORT_RADIUS),
            index,
        ))?;
        checksum += pair_checksum(&pairs)?;
        last_pairs = Some(pairs);
    }
    let elapsed = started.elapsed();
    let pair_count = last_pairs.map_or(0, |pairs| pairs.len());
    if !checksum.is_finite() {
        return Err("index benchmark checksum is nonfinite".to_owned());
    }
    Ok((elapsed, pair_count, checksum))
}

fn time_backend(
    case: &SparseCase,
    backend: SparseBackend,
    iterations: usize,
) -> Result<(Duration, SolveReport, f64), String> {
    let started = Instant::now();
    let mut last_report = None;
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let report = black_box(solve(black_box(case), backend))?;
        checksum += report.solution.iter().sum::<f64>();
        checksum += report.residual_infinity;
        last_report = Some(report);
    }
    let elapsed = started.elapsed();
    let report =
        last_report.ok_or_else(|| "benchmark iteration count must be nonzero".to_owned())?;
    if !checksum.is_finite() {
        return Err("solver benchmark checksum is nonfinite".to_owned());
    }
    Ok((elapsed, report, checksum))
}

fn run_benchmark(smoke: bool) -> Result<(), String> {
    let (sides, iterations): (&[usize], usize) = if smoke {
        (&[4, 6], 1)
    } else {
        (&[6, 8, 10], 3)
    };
    println!("{BENCHMARK_HEADER}");
    for &side in sides {
        let points = grid_points(side)?;
        for &index in SpatialIndex::ALL {
            let (elapsed, pair_count, checksum) = time_index(&points, index, iterations)?;
            println!(
                "index,{INDEX_BENCHMARK_PHASE},{},{},{pair_count},{iterations},{},{checksum:.17e},0",
                index.label(),
                points.len(),
                elapsed.as_nanos()
            );
        }
        let case = SparseCase::from_points(&points, SUPPORT_RADIUS)?;
        for &backend in SparseBackend::ALL {
            let (elapsed, report, checksum) = time_backend(&case, backend, iterations)?;
            println!(
                "solve,{SOLVER_BENCHMARK_PHASE},{},{},{},{iterations},{},{checksum:.17e},{:.17e}",
                backend.label(),
                points.len(),
                report.stored_nonzeros,
                elapsed.as_nanos(),
                report.residual_infinity
            );
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let smoke = match arguments.next().as_deref() {
        None => false,
        Some("--smoke") => true,
        Some(argument) => return Err(format!("unknown argument: {argument}")),
    };
    if let Some(argument) = arguments.next() {
        return Err(format!("unexpected extra argument: {argument}"));
    }
    run_benchmark(smoke)
}

#[cfg(test)]
mod sparse_spike_cases {
    use super::*;

    fn assert_solution_close(actual: &[f64], expected: &[f64], tolerance: f64, label: &str) {
        assert_eq!(actual.len(), expected.len(), "{label} solution length");
        for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            assert!(
                (actual - expected).abs() <= tolerance * expected.abs().max(1.0),
                "{label} solution[{index}] expected {expected:.17e}, got {actual:.17e}"
            );
        }
    }

    #[test]
    fn spatial_indices_match_brute_force_and_strict_support_boundary() -> Result<(), String> {
        let points = vec![
            [0.0, 0.0, 0.0],
            [SUPPORT_RADIUS, 0.0, 0.0],
            [SUPPORT_RADIUS * 0.5, 0.0, 0.0],
            [0.0, 1.0, 1.0],
        ];
        let truth = brute_force_pairs(&points, SUPPORT_RADIUS)?;
        assert!(!truth.contains(&NeighborPair {
            row: 0,
            column: 1,
            distance_squared: SUPPORT_RADIUS * SUPPORT_RADIUS,
        }));
        for &index in SpatialIndex::ALL {
            assert_eq!(
                indexed_pairs(&points, SUPPORT_RADIUS, index)?,
                truth,
                "{} neighborhood",
                index.label()
            );
        }
        Ok(())
    }

    #[test]
    fn compact_assembly_is_symmetric_ordered_and_local() -> Result<(), String> {
        let points = grid_points(6)?;
        let case = SparseCase::from_points(&points, SUPPORT_RADIUS)?;
        assert!(case.entries.len() <= 27 * case.dimension);
        for entry in &case.entries {
            let transposed = case
                .entries
                .binary_search_by_key(&(entry.column, entry.row), |candidate| {
                    (candidate.row, candidate.column)
                });
            let index = transposed.map_err(|_| "missing symmetric entry".to_owned())?;
            assert_eq!(case.entries[index].value.to_bits(), entry.value.to_bits());
        }
        let product = sparse_matrix_vector_product(case.dimension, &case.entries, &case.truth)?;
        assert_eq!(product, case.right_hand_side);
        Ok(())
    }

    #[test]
    fn hand_derived_wendland_csc_truth_agrees_for_every_backend() -> Result<(), String> {
        let off_diagonal = 3.0 / 16.0;
        let points = [
            [0.0, 0.0, 0.0],
            [SUPPORT_RADIUS * 0.5, 0.0, 0.0],
            [SUPPORT_RADIUS, 0.0, 0.0],
        ];
        let pairs = brute_force_pairs(&points, SUPPORT_RADIUS)?;
        let entries = assemble_wendland_c2(&pairs, SUPPORT_RADIUS)?;
        let truth = vec![1.0, 2.0, 3.0];
        let expected_product = vec![11.0 / 8.0, 11.0 / 4.0, 27.0 / 8.0];
        let case = SparseCase::new(3, entries, expected_product.clone(), truth.clone())?;
        let expected_column_pointers = vec![0, 2, 5, 7];
        let expected_row_indices = vec![0, 1, 0, 1, 2, 1, 2];
        let expected_values = vec![
            1.0,
            off_diagonal,
            off_diagonal,
            1.0,
            off_diagonal,
            off_diagonal,
            1.0,
        ];

        for &backend in SparseBackend::ALL {
            let inspection = inspect_backend_csc(&case, &truth, backend)?;
            assert_eq!((inspection.rows, inspection.columns), (3, 3));
            assert_eq!(
                inspection.column_pointers,
                expected_column_pointers,
                "{} column pointers",
                backend.label()
            );
            assert_eq!(
                inspection.row_indices,
                expected_row_indices,
                "{} row indices",
                backend.label()
            );
            assert_eq!(
                inspection.values,
                expected_values,
                "{} stored values and symmetry",
                backend.label()
            );
            assert!(
                inspection
                    .column_pointers
                    .windows(2)
                    .all(|window| window[0] < window[1]),
                "{} monotone column pointers",
                backend.label()
            );
            for column in 0..inspection.columns {
                let start = inspection.column_pointers[column];
                let end = inspection.column_pointers[column + 1];
                assert!(
                    inspection.row_indices[start..end]
                        .windows(2)
                        .all(|window| window[0] < window[1]),
                    "{} sorted unique rows in column {column}",
                    backend.label()
                );
                for position in start..end {
                    let row = inspection.row_indices[position];
                    let transpose_start = inspection.column_pointers[row];
                    let transpose_end = inspection.column_pointers[row + 1];
                    let transpose_offset = inspection.row_indices[transpose_start..transpose_end]
                        .binary_search(&column)
                        .map_err(|_| {
                            format!("{} missing transpose of ({row}, {column})", backend.label())
                        })?;
                    assert_eq!(
                        inspection.values[position].to_bits(),
                        inspection.values[transpose_start + transpose_offset].to_bits(),
                        "{} symmetry at ({row}, {column})",
                        backend.label()
                    );
                }
            }
            assert_eq!(
                inspection.product,
                expected_product,
                "{} candidate-storage matrix-vector product",
                backend.label()
            );
            assert_solution_close(&inspection.solution, &truth, 1.0e-12, backend.label());
        }
        Ok(())
    }

    #[test]
    fn analytic_sparse_spd_truth_agrees_for_every_backend() -> Result<(), String> {
        let points = grid_points(4)?;
        let case = SparseCase::from_points(&points, SUPPORT_RADIUS)?;
        for &backend in SparseBackend::ALL {
            let report = solve(&case, backend)?;
            assert_eq!(report.stored_nonzeros, case.entries.len());
            assert!(report.backward_error <= ACCEPTED_BACKWARD_ERROR);
            assert_solution_close(&report.solution, &case.truth, 1.0e-8, backend.label());
        }
        Ok(())
    }

    #[test]
    fn singular_system_is_not_reported_as_success() -> Result<(), String> {
        let entries = vec![
            SparseEntry {
                row: 0,
                column: 0,
                value: 1.0,
            },
            SparseEntry {
                row: 0,
                column: 1,
                value: 1.0,
            },
            SparseEntry {
                row: 1,
                column: 0,
                value: 1.0,
            },
            SparseEntry {
                row: 1,
                column: 1,
                value: 1.0,
            },
        ];
        let case = SparseCase::new(2, entries, vec![1.0, 0.0], vec![0.0, 0.0])?;
        for &backend in SparseBackend::ALL {
            assert!(
                solve(&case, backend).is_err(),
                "{} accepted a singular inconsistent system",
                backend.label()
            );
        }
        Ok(())
    }

    #[test]
    fn nonfinite_and_malformed_inputs_fail_before_dispatch() {
        assert!(brute_force_pairs(&[], SUPPORT_RADIUS).is_err());
        assert!(brute_force_pairs(&[[f64::NAN, 0.0, 0.0]], SUPPORT_RADIUS).is_err());
        assert!(brute_force_pairs(&[[0.0, 0.0, 0.0]], 0.0).is_err());
        assert!(SparseCase::new(0, Vec::new(), Vec::new(), Vec::new()).is_err());
        assert!(
            SparseCase::new(
                1,
                vec![SparseEntry {
                    row: 0,
                    column: 0,
                    value: f64::INFINITY,
                }],
                vec![0.0],
                vec![0.0]
            )
            .is_err()
        );
    }

    #[test]
    fn repeated_index_and_solver_reports_are_deterministic() -> Result<(), String> {
        let points = grid_points(5)?;
        for &index in SpatialIndex::ALL {
            assert_eq!(
                indexed_pairs(&points, SUPPORT_RADIUS, index)?,
                indexed_pairs(&points, SUPPORT_RADIUS, index)?,
                "{} pairs",
                index.label()
            );
        }
        let case = SparseCase::from_points(&points, SUPPORT_RADIUS)?;
        for &backend in SparseBackend::ALL {
            assert_eq!(
                solve(&case, backend)?,
                solve(&case, backend)?,
                "{} report",
                backend.label()
            );
        }
        Ok(())
    }

    #[test]
    fn candidate_indices_match_on_scaling_fixture() -> Result<(), String> {
        let points = grid_points(8)?;
        let truth = brute_force_pairs(&points, SUPPORT_RADIUS)?;
        assert!(truth.len() <= 14 * points.len());
        for &index in SpatialIndex::ALL {
            assert_eq!(
                indexed_pairs(&points, SUPPORT_RADIUS, index)?,
                truth,
                "{} scaling pairs",
                index.label()
            );
        }
        Ok(())
    }

    #[test]
    fn benchmark_output_schema_names_explicit_end_to_end_phases() {
        assert_eq!(
            BENCHMARK_HEADER,
            "kind,phase,candidate,points,nonzeros_or_pairs,iterations,nanoseconds,checksum,residual"
        );
        assert!(INDEX_BENCHMARK_PHASE.ends_with("_end_to_end"));
        assert!(SOLVER_BENCHMARK_PHASE.ends_with("_end_to_end"));
        assert!(SOLVER_BENCHMARK_PHASE.contains("construct_factor_solve_review"));
    }

    #[cfg(feature = "kiddo-index")]
    #[test]
    #[should_panic(expected = "Too many items with the same position on one axis")]
    fn kiddo_default_bucket_panics_on_valid_axis_aligned_grid() {
        let mut tree: kiddo::KdTree<f64, DIMENSION> = kiddo::KdTree::new();
        let mut item = 0_u64;
        for z in 0_u32..10 {
            for y in 0_u32..10 {
                for x in 0_u32..10 {
                    tree.add(&[f64::from(x), f64::from(y), f64::from(z)], item);
                    item += 1;
                }
            }
        }
    }
}
