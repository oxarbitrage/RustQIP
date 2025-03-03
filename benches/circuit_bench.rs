#![feature(test)]

use qip::boolean_circuits::arithmetic::exp_mod;
use qip::pipeline::run_with_init;
use qip::qubits::RegisterHandle;
use qip::sparse_state::SparseQuantumState;
use qip::*;

fn exp_mod_circuit(
    n: u64,
    k: u64,
) -> Result<
    (
        Register,
        RegisterHandle,
        RegisterHandle,
        RegisterHandle,
        RegisterHandle,
    ),
    CircuitError,
> {
    let mut b = OpBuilder::new();
    let (ra, ha) = b.register_and_handle(n + 1)?;
    let (rb, hb) = b.register_and_handle(k)?;
    let (rm, hm) = b.register_and_handle(n)?;
    let (rp, hp) = b.register_and_handle(n + 1)?;
    let re = b.register(n + 1)?;

    let (ra, rb, rm, rp, re) = exp_mod(&mut b, ra, rb, rm, rp, re)?;

    let r = b.merge(vec![ra, rb, rm, rp, re])?;
    Ok((r, ha, hb, hm, hp))
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    #[bench]
    fn bench_exp_mod_circuit_base(bencher: &mut Bencher) {
        let circuit_n = 2;
        let mod_k = 1;
        let (reg, ha, hb, hm, hp) = exp_mod_circuit(circuit_n, mod_k).unwrap();

        let a = 1;
        let b = 1;
        let m = 3;
        let p = 1;

        bencher.iter(|| {
            run_with_init::<f64, SparseQuantumState<f64>>(
                &reg,
                &[
                    ha.make_init_from_index(a).unwrap(),
                    hb.make_init_from_index(b).unwrap(),
                    hm.make_init_from_index(m).unwrap(),
                    hp.make_init_from_index(p).unwrap(),
                ],
            )
            .unwrap();
        });
    }

    #[bench]
    fn bench_exp_mod_circuit_rec(bencher: &mut Bencher) {
        let circuit_n = 2;
        let mod_k = 2;
        let (reg, ha, hb, hm, hp) = exp_mod_circuit(circuit_n, mod_k).unwrap();

        let a = 1;
        let b = 1;
        let m = 3;
        let p = 1;

        bencher.iter(|| {
            run_with_init::<f64, SparseQuantumState<f64>>(
                &reg,
                &[
                    ha.make_init_from_index(a).unwrap(),
                    hb.make_init_from_index(b).unwrap(),
                    hm.make_init_from_index(m).unwrap(),
                    hp.make_init_from_index(p).unwrap(),
                ],
            )
            .unwrap();
        });
    }
}
