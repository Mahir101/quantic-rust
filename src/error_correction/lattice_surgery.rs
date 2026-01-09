//! Lattice Surgery for Fault-Tolerant Quantum Computing
//!
//! This module implements operations for lattice surgery:
//! - Merge operations (MZZ, MXX)
//! - Split operations
//! - Multi-patch protocols
//!
//! ## 🎯 Why is this used?
//! In a topological quantum computer (like the Surface Code), physical movement 
//! of qubits is often replaced by "Lattice Surgery". This technique allows for 
//! logical entanglement and multi-qubit measurements between logical qubits 
//! by dynamically changing the boundaries of the code patches. It is the 
//! primary method for performing CNOT gates and measuring stabilizer operators 
//! in a fault-tolerant way.
//!
//! ## ⚙️ How it works?
//! - **Merge Operation**: Combines two independent logical patches into a 
//!   single, larger patch by measuring the joint boundary stabilizers. 
//!   For example, a Merge-Z measures the $Z \otimes Z$ parity between patches.
//! - **Split Operation**: Reverses a merge by measuring a transversal 
//!   set of operators along the split boundary, effectively re-establishing 
//!   two independent logical qubits.
//!
//! ## 📍 Where to apply this?
//! - **Logical Gate Implementation**: Performing logical CNOT or CZ gates 
//!   between Surface Code patches.
//! - **Resource Management**: Resizing or relocating logical qubits on a 
//!   2D qubit array.
//! - **Magic State Injection**: Measuring an injected state into a surface 
//!   code register.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: $O(B)$ where $B$ is the number of qubits along the 
//!   boundary (typically proportional to the code distance $d$).
//! - **Hardware Requirement**: Directly requires a 2D nearest-neighbor 
//!   connected qubit architecture.
//! - **Probabilistic Nature**: The outcomes of boundary measurements must 
//!   be tracked for Pauli frame updates.

use crate::gates::core::Gate;

/// Lattice surgery merge-Z (MZZ) operation
pub fn lattice_surgery_merge_z(
    patch1_qubits: &[usize],
    patch2_qubits: &[usize],
    ancilla_qubits: &[usize],
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Entangle boundary qubits with ancilla
    for (&p1, &a) in patch1_qubits.iter().zip(ancilla_qubits.iter()) {
        gates.push(Gate::CX(p1, a));
    }
    for (&p2, &a) in patch2_qubits.iter().zip(ancilla_qubits.iter()) {
        gates.push(Gate::CX(p2, a));
    }
    
    gates
}

/// Lattice surgery split-Z operation
pub fn lattice_surgery_split_z(
    patch_qubits: &[usize],
) -> Vec<Gate> {
    let mut gates = Vec::new();
    // In Split-Z, we perform H on the boundary and measure
    for &q in patch_qubits {
        gates.push(Gate::H(q));
    }
    gates
}
