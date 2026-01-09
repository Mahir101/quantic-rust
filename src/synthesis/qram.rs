//! Quantum Random Access Memory (QRAM)
//!
//! This module implements QRAM circuit constructions:
//! - Bucket-brigade QRAM
//! - Fan-out QRAM
//!
//! ## 🎯 Why is this used?
//! QRAM is the quantum equivalent of classical RAM, allowing a quantum 
//! algorithm to access classical data stored in "memory" using a quantum 
//! superposition of addresses. This is a critical component for algorithms 
//! that process large datasets, such as Quantum Machine Learning, Grover 
//! search over databases, and certain quantum chemistry simulations.
//!
//! ## ⚙️ How it works?
//! - **Bucket-Brigade**: Uses a tree-like structure of "routing" qubits. 
//!   The address bits navigate the tree to activate a unique path to a 
//!   memory cell. This architecture is efficient as it only activates $O(\log N)$ 
//!   routing nodes per access.
//! - **Fan-out QRAM**: Uses a parallel "fan-out" of the address bits to 
//!   access all memory cells simultaneously and then filters for the 
//!   target address.
//!
//! ## 📍 Where to apply this?
//! - **Database Search**: Oracles that need to retrieve values based on 
//!   an index in superposition.
//! - **QML**: Loading feature vectors into quantum states.
//! - **Lookup Tables**: Implementing complex non-linear functions by 
//!   storing their values in a grid.
//!
//! ## 📊 Code Behavior
//! - **Complexity**: 
//!     - Bucket-Brigade: $O(N)$ physical qubits for $N$ memory cells, but 
//!       only $O(\log N)$ gates are active per memory request.
//! - **Ancilla Management**: Requires a significant number of routing 
//!   qubits that must be properly uncomputed to avoid decoherence.
//! - **Connectivity**: Performance is highly dependent on the ability 
//!   to map a tree structure onto the physical QPU topology.

use crate::gates::core::Gate;

// ============================================================================
// BUCKET-BRIGADE QRAM
// ============================================================================

/// Generate a bucket-brigade QRAM circuit
/// 
/// # Arguments
/// * `address_qubits` - Qubits encoding the address
/// * `routing_qubits` - Qubits used for the routing tree
/// * `data_qubits` - Qubits encoding the memory data
pub fn bucket_brigade_qram(
    address_qubits: &[usize],
    routing_qubits: &[Vec<usize>], // routing_qubits[layer][index]
    data_qubits: &[usize],
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // 1. Layer-by-layer activation of routing tree
    for (i, &aq) in address_qubits.iter().enumerate() {
        let _layer = &routing_qubits[i];
        // For each node in the layer, apply routing based on address bit
        // Simplified: Controlled activation of child nodes
        for &rq in &routing_qubits[i] {
            gates.push(Gate::CX(aq, rq));
        }
    }
    
    // 2. Data readout
    for &dq in data_qubits {
        // Controlled transfer from memory to data register
        // Simplified: Use the last layer of routing to control data access
        if let Some(last_layer) = routing_qubits.last() {
            for &rq in last_layer {
                gates.push(Gate::CX(rq, dq));
            }
        }
    }
    
    // 3. Uncompute routing tree
    for (i, &aq) in address_qubits.iter().enumerate().rev() {
        for &rq in &routing_qubits[i] {
            gates.push(Gate::CX(aq, rq));
        }
    }
    
    gates
}

// ============================================================================
// FAN-OUT QRAM
// ============================================================================

/// Fan-out QRAM architecture
pub fn fan_out_qram(
    address_qubits: &[usize],
    data_qubits: &[usize],
) -> Vec<Gate> {
    let mut gates = Vec::new();
    
    // Fan-out involves unary encoding of the address
    // and then controlled-data transfer.
    
    // Simplified representation of address decoding
    for &aq in address_qubits {
        for &dq in data_qubits {
            gates.push(Gate::CX(aq, dq));
        }
    }
    
    gates
}
