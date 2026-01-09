//! Quantum Random Access Memory (QRAM)
//!
//! This module implements QRAM circuit constructions:
//! - Bucket-brigade QRAM
//! - Fan-out QRAM

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
