# Rustiq 🦀⚛️

[![Rust](https://github.com/smartiel/rustiq/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/smartiel/rustiq/actions/workflows/rust.yml)
[![PyPI version](https://badge.fury.io/py/rustiq.svg)](https://badge.fury.io/py/rustiq)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Rustiq** is a high-performance Rust library with Python bindings designed for advanced quantum circuit synthesis, optimization, and analysis. It provides a comprehensive suite of tools ranging from foundational gates and arithmetic to cutting-edge error mitigation and circuit cutting techniques.

---

## 🚀 Key Features

*   **Advanced Synthesis**: Pauli network synthesis, Clifford isometry synthesis, and state preparation (Dicke, GHZ, W-states).
*   **Algorithms Register**: Full implementations of QFT, HHL, Quantum Walks, and Hamiltonian simulations (Trotter-Suzuki, LCU).
*   **Circuit Optimization**: Multi-stage optimization pipeline including gate merging, commutation analysis, T-count minimization, and ZX-calculus rules.
*   **Error Correction & Fault Tolerance**: Support for Surface codes, Lattice Surgery, QLDPC codes, and various QEC decoders (MWPM, BP).
*   **Variational Framework**: Tools for VQE/QAOA, including Hardware Efficient Ansatz (HEA), UCCSD, and Parameter-Shift gradients.
*   **Error Mitigation**: zero-noise extrapolation (ZNE), probabilistic error cancellation (PEC), and Clifford data regression (CDR).
*   **Quantum Circuit Cutting**: Distributed quantum computing utilities for wire and gate cutting (wire/gate cutting, entanglement forging).

---

## 🛠 Installation

### Python Interface
Rustiq is primarily used as a Python library. You can install it directly via pip:

```bash
pip install git+https://github.com/smartiel/rustiq.git@main
```

*Note: Requires a [Rust toolchain](https://rustup.rs/) installed on your system for compilation.*

### Rust Crate
For high-performance Rust applications, add Rustiq to your `Cargo.toml`:

```toml
[dependencies]
rustiq = { git = "https://github.com/smartiel/rustiq" }
```

---

## 📖 Module Overview

### 1. Quantum Algorithms (`rustiq::algorithms`)
A collection of core quantum routines and higher-level algorithms.
*   **QFT/IQFT**: Standard and approximate Quantum Fourier Transforms.
*   **Arithmetic**: Draper adders, ripple-carry adders (VBE, Cuccaro), and modular multipliers.
*   **Quantum Walks**: Discrete-time (DTQW) and Continuous-time (CTQW) walks on various graphs.
*   **Linear Systems**: HHL algorithm structure and Quantum Singular Value Transformation (QSVT) steps.
*   **Simulations**: Higher-order Trotter-Suzuki formulas and Linear Combination of Unitaries (LCU).

### 2. Synthesis & State Prep (`rustiq::synthesis`)
Techniques for converting mathematical operators into hardware-level circuits.
*   **State Preparation**: Isometry-based preparation, Grover-Rudolph, and amplitude encoding.
*   **QRAM**: Architectures including Bucket-Brigade and Fan-out QRAM.
*   **Decompositions**: Quantum Shannon Decomposition (QSD) and two-level unitary decomposition.
*   **Standard States**: Optimized synthesis for GHZ, W, and Dicke states.

### 3. Circuit Optimization (`rustiq::optimization`)
Powerful routines to reduce gate counts and circuit depth.
*   **Pipeline**: Automated optimization including gate cancellation, rotation merging, and peephole optimization.
*   **Resource Minimization**: Targeted T-count and CNOT-depth reduction.
*   **ZX-Calculus**: Advanced optimization using spider fusion and pivot rules inspired by the ZX-calculus.

### 4. Error Correction (`rustiq::error_correction`)
Foundations for fault-tolerant quantum computing.
*   **Codes**: Implementation of Bit-flip, Shor (9-qubit), Steane (7-qubit), and Surface codes.
*   **Lattice Surgery**: Logical Z-merge/split operations for topological qubits.
*   **QLDPC**: Syndrome extraction for Hypergraph Product codes.
*   **Decoders**: Conceptual logic for Minimum Weight Perfect Matching (MWPM) and Belief Propagation.

### 5. Variational Methods (`rustiq::variational`)
Tools for NISQ algorithms like VQE and QAOA.
*   **Ansatz Library**: Hardware Efficient Ansatz (HEA) with configurable entangling patterns, and fermionic UCCSD.
*   **QAOA**: Specialized layers for MaxCut, weighted MaxCut, and General Ising problems.
*   **Gradients**: Automated Parameter-Shift rule implementations.
*   **Metric Tests**: SWAP test and Hadamard test for state overlap and expectation values.

### 6. Error Mitigation & Cutting (`rustiq::error_mitigation`/`cutting`)
Techniques for improving results on noisy hardware.
*   **Mitigation**: Zero Noise Extrapolation (ZNE), PEC (Quasi-probability), symmetry verification, and virtual distillation.
*   **Circuit Cutting**: Wire cutting and gate cutting (CNOT/CZ/SWAP) to execute large circuits on smaller QPUs.

---

## 💻 Example Usage (Python)

```python
import rustiq
from rustiq import Metric

# Synthesize a Pauli Network
paulis = ["XYZI", "IZYX", "XIYI"]
circuit = rustiq.pauli_network_synthesis(
    paulis, 
    metric=Metric.DEPTH, 
    preserve_order=True
)

# Analyze the circuit
stats = rustiq.analyze_circuit(circuit)
print(f"Gate Count: {stats['count']}, T-depth: {stats['t_depth']}")
```

---

## 🔬 References

If you use Rustiq in your research, please cite the following works:

**Pauli Network Synthesis:**
```bibtex
@misc{debrugière2024faster,
      title={Faster and shorter synthesis of Hamiltonian simulation circuits}, 
      author={Timothée Goubault de Brugière and Simon Martiel},
      year={2024},
      eprint={2404.03280},
      archivePrefix={arXiv},
      primaryClass={quant-ph}
}
```

**Clifford & Isometry Synthesis:**
```bibtex
@misc{debrugière2022graphstate,
      title={A graph-state based synthesis framework for Clifford isometries}, 
      author={Timothée Goubault de Brugière and Simon Martiel and Christophe Vuillot},
      year={2022},
      eprint={2212.06928},
      archivePrefix={arXiv},
      primaryClass={quant-ph}
}
```

---

## 🛡 License
This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.