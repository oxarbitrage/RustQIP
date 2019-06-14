use std::fmt;
use std::rc::Rc;

use num::complex::Complex;

use super::pipeline::*;
use super::state_ops::*;

/// Possible relations to a parent qubit
pub enum Parent {
    Owned(Vec<Qubit>, Option<QubitOp>),
    Shared(Rc<Qubit>),
}

/// A qubit object, possible representing multiple physical qubit indices.
pub struct Qubit {
    pub indices: Vec<u64>,
    pub parent: Option<Parent>,
    pub id: u64,
}

impl Qubit {
    fn new(id: u64, indices: Vec<u64>) -> Qubit {
        Qubit {
            indices,
            parent: None,
            id,
        }
    }

    /// Merge qubits to for a new qubit object.
    pub fn merge_with_fn(id: u64, qubits: Vec<Qubit>, operator: Option<QubitOp>) -> Qubit {
        let mut all_indices = Vec::new();

        for q in qubits.iter() {
            all_indices.extend(q.indices.iter());
        }
        all_indices.sort();

        Qubit {
            indices: all_indices,
            parent: Some(Parent::Owned(qubits, operator)),
            id,
        }
    }

    /// Split a qubit in two, with one having the indices in `selected_indices`
    pub fn split(ida: u64, idb: u64, q: Qubit, selected_indices: Vec<u64>) -> (Qubit, Qubit) {
        let remaining = q.indices.clone()
            .into_iter()
            .filter(|x| !selected_indices.contains(x))
            .collect();
        let shared_parent = Rc::new(q);

        (Qubit {
            indices: selected_indices,
            parent: Some(Parent::Shared(shared_parent.clone())),
            id: ida,
        }, Qubit {
            indices: remaining,
            parent: Some(Parent::Shared(shared_parent.clone())),
            id: idb,
        })
    }
}

impl std::cmp::Eq for Qubit {}

impl std::cmp::PartialEq for Qubit {
    fn eq(&self, other: &Qubit) -> bool {
        self.id == other.id
    }
}

impl std::cmp::Ord for Qubit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl std::cmp::PartialOrd for Qubit {
    fn partial_cmp(&self, other: &Qubit) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for Qubit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let int_strings = self.indices.iter()
            .map(|x| x.clone().to_string())
            .collect::<Vec<String>>();

        write!(f, "Qubit[{}]", int_strings.join(", "))
    }
}

/// A builder which supports non-unitary operations
pub trait NonUnitaryBuilder {
    // Things like measure go here
    fn measure(&mut self, q: Qubit) -> MeasuredResultReference;
}

/// A builder which support unitary operations
pub trait UnitaryBuilder {
    // Things like X, Y, Z, NOT, H, SWAP, ... go here

    /// Build a builder which uses `q` as context.
    fn with_context(&mut self, q: Qubit) -> ConditionalContextBuilder;

    /// Build a generic matrix op, apply to `q`
    fn mat(&mut self, q: Qubit, mat: &[Complex<f64>]) -> Qubit;

    /// Build a matrix op from real numbers, apply to `q`
    fn real_mat(&mut self, q: Qubit, mat: &[f64]) -> Qubit {
        self.mat(q, from_reals(mat).as_slice())
    }

    /// Apply NOT to `q`
    fn not(&mut self, q: Qubit) -> Qubit {
        self.x(q)
    }

    /// Apply X to `q`
    fn x(&mut self, q: Qubit) -> Qubit {
        self.real_mat(q, &[0.0, 1.0, 1.0, 0.0])
    }

    /// Apply Y to `q`
    fn y(&mut self, q: Qubit) -> Qubit {
        self.mat(q, from_tuples(&[(0.0, 0.0), (0.0, -1.0), (0.0, 0.0), (0.0, 1.0)])
            .as_slice())
    }

    /// Apply Z to `q`
    fn z(&mut self, q: Qubit) -> Qubit {
        self.real_mat(q, &[1.0, 0.0, 0.0, -1.0])
    }

    /// Apply H to `q`
    fn hadamard(&mut self, q: Qubit) -> Qubit {
        let inv_sqrt = 1.0 / 2.0_f64.sqrt();
        self.real_mat(q, &[1.0 * inv_sqrt, 1.0 * inv_sqrt, 1.0 * inv_sqrt, -1.0 * inv_sqrt])
    }

    /// Apply SWAP to `qa` and `qb`
    fn swap(&mut self, qa: Qubit, qb: Qubit) -> (Qubit, Qubit) {
        let op = self.make_swap_op(&qa, &qb);
        let qa_indices = qa.indices.clone();
        let q = self.merge_with_fn(vec![qa, qb], Some(op));
        self.split(q, qa_indices)
    }

    /// Merge the qubits in `qs` into a single qubit.
    fn merge(&mut self, qs: Vec<Qubit>) -> Qubit {
        self.merge_with_fn(qs, None)
    }

    /// Split the qubit `q` into two, one which `selected_indices` and one with the remaining.
    fn split(&mut self, q: Qubit, selected_indices: Vec<u64>) -> (Qubit, Qubit);

    /// Build a generic matrix op.
    fn make_mat_op(&self, q: &Qubit, data: Vec<Complex<f64>>) -> QubitOp {
        QubitOp::MatrixOp(q.indices.clone(), data)
    }

    /// Build a swap op.
    fn make_swap_op(&self, qa: &Qubit, qb: &Qubit) -> QubitOp {
        QubitOp::SwapOp(qa.indices.clone(), qb.indices.clone())
    }

    /// Merge qubits using a generic state processing function.
    fn merge_with_fn(&mut self, qs: Vec<Qubit>, operator: Option<QubitOp>) -> Qubit;

}

/// A basic builder for unitary and non-unitary ops.
pub struct OpBuilder {
    qubit_index: u64,
    op_id: u64,
}

impl OpBuilder {
    pub fn new() -> OpBuilder {
        OpBuilder {
            qubit_index: 0,
            op_id: 0,
        }
    }

    /// Build a new qubit with `n` indices
    pub fn qubit(&mut self, n: u64) -> Qubit {
        let base_index = self.qubit_index;
        self.qubit_index = self.qubit_index + n;

        Qubit::new(self.get_op_id(), (base_index..self.qubit_index).collect())
    }

    fn get_op_id(&mut self) -> u64 {
        let tmp = self.op_id;
        self.op_id += 1;
        tmp
    }
}

impl NonUnitaryBuilder for OpBuilder {
    fn measure(&mut self, q: Qubit) -> MeasuredResultReference {
        unimplemented!()
    }
}

impl UnitaryBuilder for OpBuilder {
    fn with_context(&mut self, q: Qubit) -> ConditionalContextBuilder {
        ConditionalContextBuilder {
            parent_builder: self,
            conditioned_qubit: Some(q),
        }
    }

    fn mat(&mut self, q: Qubit, mat: &[Complex<f64>]) -> Qubit {
        let op = self.make_mat_op(&q, mat.to_vec());
        self.merge_with_fn(vec![q], Some(op))
    }

    fn split(&mut self, q: Qubit, selected_indices: Vec<u64>) -> (Qubit, Qubit) {
        Qubit::split(self.get_op_id(), self.get_op_id(), q, selected_indices)
    }

    fn merge_with_fn(&mut self, qs: Vec<Qubit>, op: Option<QubitOp>) -> Qubit {
        Qubit::merge_with_fn(self.get_op_id(), qs, op)
    }
}

/// An op builder which depends on the value of a given qubit (COPs)
pub struct ConditionalContextBuilder<'a> {
    parent_builder: &'a mut UnitaryBuilder,
    conditioned_qubit: Option<Qubit>,
}

impl<'a> ConditionalContextBuilder<'a> {
    /// Release the qubit used to build this builder
    pub fn release_qubit(self: Self) -> Qubit {
        match self.conditioned_qubit {
            Some(q) => q,
            None => panic!("Conditional context builder failed to populate qubit.")
        }
    }

    fn get_conditional_qubit(&mut self) -> Qubit {
        self.conditioned_qubit.take().unwrap()
    }

    fn set_conditional_qubit(&mut self, cq: Qubit) {
        self.conditioned_qubit = Some(cq);
    }
}

impl<'a> UnitaryBuilder for ConditionalContextBuilder<'a> {
    fn with_context(&mut self, q: Qubit) -> ConditionalContextBuilder {
        ConditionalContextBuilder {
            parent_builder: self,
            conditioned_qubit: Some(q),
        }
    }

    fn mat(&mut self, q: Qubit, mat: &[Complex<f64>]) -> Qubit {
        let op = self.make_mat_op(&q, mat.to_vec());
        let cq = self.get_conditional_qubit();
        let cq_indices = cq.indices.clone();
        let q = self.merge_with_fn(vec![cq, q], Some(op));
        let (cq, q) = self.split(q, cq_indices);

        self.set_conditional_qubit(cq);
        q
    }

    fn split(&mut self, q: Qubit, selected_indices: Vec<u64>) -> (Qubit, Qubit) {
        self.parent_builder.split(q, selected_indices)
    }

    fn make_mat_op(&self, q: &Qubit, data: Vec<Complex<f64>>) -> QubitOp {
        match &self.conditioned_qubit {
            Some(cq) => QubitOp::ControlOp(cq.indices.clone(),
                                           Box::new(self.parent_builder.make_mat_op(q, data))),
            None => panic!("Conditional context builder failed to populate qubit.")
        }
    }

    fn make_swap_op(&self, qa: &Qubit, qb: &Qubit) -> QubitOp {
        match &self.conditioned_qubit {
            Some(cq) => QubitOp::ControlOp(cq.indices.clone(),
                                           Box::new(self.parent_builder.make_swap_op(qa, qb))),
            None => panic!("Conditional context builder failed to populate qubit.")
        }
    }

    fn merge_with_fn(&mut self, qs: Vec<Qubit>, op: Option<QubitOp>) -> Qubit {
        self.parent_builder.merge_with_fn(qs, op)
    }
}