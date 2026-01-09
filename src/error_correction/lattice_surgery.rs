//! Lattice Surgery for Fault-Tolerant Quantum Computing
//!
//! This module implements operations for lattice surgery:
//! - Merge operations (MZZ, MXX)
//! - Split operations
//! - Multi-patch protocols

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
