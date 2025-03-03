mod utils;

use qip::pipeline::MeasurementHandle;
use qip::qubits::RegisterHandle;
use qip::*;
use utils::assert_almost_eq;

fn setup_cswap_circuit(
    vec_n: u64,
) -> Result<(Register, RegisterHandle, RegisterHandle, MeasurementHandle), CircuitError> {
    // Setup inputs
    let mut b = OpBuilder::new();
    let q = b.qubit();
    let ra = b.register(vec_n)?;
    let rb = b.register(vec_n)?;

    // We will want to feed in some inputs later.
    let ha = ra.handle();
    let hb = rb.handle();

    // Define circuit
    let q = b.hadamard(q);

    let (q, _, _) = b.cswap(q, ra, rb)?;
    let q = b.hadamard(q);

    let (q, m1) = b.measure(q);

    Ok((q, ha, hb, m1))
}

#[test]
fn test_cswap_aligned() -> Result<(), CircuitError> {
    // Setup inputs
    let (q, ha, hb, m1) = setup_cswap_circuit(3)?;

    // Run circuit
    let (_, measured) = run_local_with_init::<f64>(
        &q,
        &[ha.make_init_from_index(0)?, hb.make_init_from_index(0)?],
    )?;

    let (m, p) = measured.get_measurement(&m1).unwrap();
    assert_eq!(m, 0);
    assert_almost_eq(p, 1.0, 10);

    Ok(())
}

#[test]
fn test_cswap_orthogonal() -> Result<(), CircuitError> {
    // Setup inputs
    let (q, ha, hb, m1) = setup_cswap_circuit(3)?;

    // Run circuit
    let (_, measured) = run_local_with_init::<f64>(
        &q,
        &[ha.make_init_from_index(0)?, hb.make_init_from_index(1)?],
    )?;

    let (m, p) = measured.get_measurement(&m1).unwrap();
    assert!(m == 0 || m == 1);
    assert_almost_eq(p, 0.5, 10);

    Ok(())
}

/// sin((2 * pi * i * w / L) + d) normalized
fn sin_wave(n: u64, freq: f64, d: f64) -> Vec<Complex<f64>> {
    let len = 1 << n;
    let state: Vec<_> = (0..len)
        .map(|i| -> Complex<f64> {
            let v = (d + (std::f64::consts::PI * freq * i as f64 / len as f64)).sin();
            Complex { re: v, im: 0.0 }
        })
        .collect();
    let mag: f64 = state.iter().map(|c| c.norm_sqr()).sum();
    let mag = mag.sqrt();
    state.iter().map(|c| -> Complex<f64> { c / mag }).collect()
}

#[test]
fn test_cswap_waves() -> Result<(), CircuitError> {
    // Setup inputs
    let (q, ha, hb, m1) = setup_cswap_circuit(3)?;

    let s1 = sin_wave(3, 1.0, 0.0);
    let s2 = sin_wave(3, 2.0, 0.0);

    // Run circuit
    let (_, measured) = run_local_with_init::<f64>(
        &q,
        &[ha.make_init_from_state(s1)?, hb.make_init_from_state(s2)?],
    )?;

    let (m, p) = measured.get_measurement(&m1).unwrap();
    assert!(m == 0 || m == 1);
    assert_almost_eq(p, 0.5, 3);

    Ok(())
}
