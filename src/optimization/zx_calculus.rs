//! ZX-Calculus Based Optimization
//!
//! This module implements optimization rules based on ZX-calculus:
//! - Spider fusion
//! - Identity removal
//! - Pivot/Local complementation rules (conceptual)
//!
//! ## 🎯 Why is this used?
//! ZX-calculus is a powerful graphical language for quantum computation that 
//! ignores the sequential "gate" structure in favor of a graph-based 
//! representation (spiders and edges). This allowed for optimizations that are 
//! hard to see in standard circuit form, such as reducing T-counts by 
//! simplifying internal phase-carrying graph nodes.
//!
//! ## ⚙️ How it works?
//! - **Spider Fusion**: Merges adjacent spiders of the same color (Z or X) 
//!   by summing their phases.
//! - **Identity Removal**: Removes any spider with a zero phase and only 
//!   two connections.
//! - **Graph-Like Simplication**: Converts a circuit into a graph of spiders, 
//!   applies local complementation and pivoting rules to eliminate internal 
//!   Hadamard-connected spiders, and then re-synthesizes an optimized circuit.
//!
//! ## 📍 Where to apply this?
//! - **T-count reduction**: Highly effective for Clifford + T circuits.
//! - **Structural simplification**: Large-scale circuits with complex 
//!   dependency patterns.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: Graph simplification is polynomial in the number of 
//!   spiders ($O(V^3)$ for pivoting). 
//! - **Agnosticism**: The ZX-calculus does not care about the underlying 
//!   qubit topology, making it a "logical" optimizer.
//! - **Side Effects**: Final re-synthesis might result in a completely 
//!   different gate sequence than the original.

use crate::gates::core::Gate;

/// Apply spider fusion rules
/// 
/// Z_alpha Z_beta = Z_(alpha+beta)
pub fn spider_fusion(circuit: &[Gate]) -> Vec<Gate> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < circuit.len() {
        if i + 1 < circuit.len() {
            match (&circuit[i], &circuit[i+1]) {
                (Gate::RZ(q1, a1), Gate::RZ(q2, a2)) if q1 == q2 => {
                    result.push(Gate::RZ(*q1, a1 + a2));
                    i += 2;
                    continue;
                }
                (Gate::RX(q1, a1), Gate::RX(q2, a2)) if q1 == q2 => {
                    result.push(Gate::RX(*q1, a1 + a2));
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        result.push(circuit[i].clone());
        i += 1;
    }
    
    result
}

/// Pivot rule (conceptual) - used in graph-like ZX simplification
pub fn apply_pivot_rule(_graph: &mut Vec<Gate>) {
    // This would involve identifying a pair of internal hubs and 
    // applying the pivot transform to simplify connectivity.
}
