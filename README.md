# QForge

**An open-source quantum circuit compiler and optimizer, written in Rust.**

QForge takes OpenQASM 2.0 circuits through a multi-pass optimization pipeline, maps them to real hardware topologies using SABRE routing, estimates fidelity against IBM-style noise models, and decomposes to the IBM native gate set - ready to run on real quantum hardware.

```
cargo run --bin qforge -- compile circuits/ghz.qasm --native --target grid --qubits 9
```

```
QForge v0.1.0

  Parsing...        done  (0ms)
  Optimizing...     done  (0ms)
  Decomposing...    done  (IBM native: CX, RZ, SX, X)
  Routing...        done

  =============================================
  COMPILATION REPORT
  =============================================

  Input
    Qubits          5
    Gates           16
    2Q gates        4
    Depth           7

  Output (optimized)
    Gates           10      (-37.5%)
    2Q gates        4      (-0.0%)
    Depth           6      (-14.3%)

  Hardware (grid-3x3):
    SWAPs inserted  4
    SWAP cost (CX)  12
    Routed gates    22

  Selected: aggressive
    Est. fidelity:   61.74%
    Circuit time:    7168ns
    Readout error:   7.75%
    Decoherence:     30.67%
  =============================================
```

---

## What it solves

Quantum circuits cannot run directly on hardware. Before submission, a compiler must:

**Reduce gate count.** Quantum states decohere. Every unnecessary gate is another chance for error. A circuit with 500 gates might succeed 20% of the time. The same circuit optimized to 300 gates might succeed 60% of the time. Gate reduction is not a performance optimisation - it determines whether you get a useful answer or noise.

**Satisfy connectivity constraints.** On a real chip, only physically adjacent qubits can interact. If your circuit needs a CX gate between non-adjacent qubits, the compiler must insert SWAP gates to move the quantum state - at 3× the cost of a regular gate each time.

**Decompose to native gates.** IBM hardware runs four gates natively: CX, RZ, SX, X. A circuit written with H, T, S, and CNOT must be translated before it can execute.

QForge handles all three.

---

## Benchmark results

Ten circuits from the standard benchmark suite, compiled on Apple M-class equivalent hardware:

| Circuit | Input gates | Output gates | Reduction | Fidelity (est.) |
|---|---|---|---|---|
| bell | 4 | 4 | 0.0% | 96.12% |
| ghz | 16 | 10 | **37.5%** | 61.74% |
| qft3 | 10 | 10 | 0.0% | 79.71% |
| qft5 | 22 | 22 | 0.0% | 71.43% |
| grover3 | 24 | 20 | **16.7%** | 68.22% |
| bv5 | 18 | 14 | **22.2%** | 74.18% |
| teleport | 7 | 7 | 0.0% | 91.34% |
| random5 | 22 | 12 | **45.5%** | 73.61% |
| phase_kick | 20 | 10 | **50.0%** | 81.44% |
| adder3 | 12 | 12 | 0.0% | 79.83% |

QForge correctly leaves circuits with no redundancy unchanged (Bell, QFT, teleport) and removes redundancy aggressively where it exists (random5, phase_kick). The optimizer never touches gates it cannot prove are redundant.

**Hardware routing comparison on GHZ (5 qubits, 4 CX gates):**

| Topology | SWAPs inserted | CX equivalent cost |
|---|---|---|
| linear-5 | 12 | 36 |
| heavy-hex-7 | 8 | 24 |
| grid-3x3 | **4** | **12** |

**Statevector simulation - Bell state verification (4096 shots):**

```
  |11>    2073   50.61%  ██████████
  |00>    2023   49.39%  ██████████
```

Quantum entanglement verified in software. The 0.6% deviation from perfect 50/50 is statistical noise at 4096 shots.

---

## Quick start

```bash
git clone https://github.com/MA-V4/Q-Forge
cd Q-Forge
cargo build
```

**Compile and optimize:**
```bash
cargo run --bin qforge -- compile circuits/ghz.qasm
```

**Target real hardware (IBM-style grid topology):**
```bash
cargo run --bin qforge -- compile circuits/ghz.qasm --target grid --qubits 9
```

**Decompose to IBM native gate set (CX, RZ, SX, X):**
```bash
cargo run --bin qforge -- compile circuits/ghz.qasm --native
```

**Simulate with statevector:**
```bash
cargo run --bin qforge -- simulate circuits/bell.qasm --shots 4096
```

**Run the full benchmark suite:**
```bash
cargo run --bin qforge -- bench
# Outputs bench_results.json and bench_report.html
```

**Dump IR as JSON:**
```bash
cargo run --bin qforge -- ir circuits/bell.qasm
```

---

## CLI reference

```
qforge compile  <circuit.qasm> [--target linear|grid|heavy-hex]
                               [--qubits N]
                               [--native]

qforge simulate <circuit.qasm> [--shots 1024]

qforge ir       <circuit.qasm>

qforge bench    [--circuits circuits/]
                [--out results.json]
                [--html report.html]
```

---

## Architecture

Nine Rust crates, zero external quantum dependencies:

```
crates/
  qforge-ir/         Gate types, circuit representation, DAG depth
  qforge-parser/     OpenQASM 2.0 lexer and recursive descent parser
  qforge-optimizer/  Pass manager and eight optimization passes
  qforge-routing/    Hardware topology, qubit allocation, SABRE routing
  qforge-noise/      IBM-style noise model, fidelity estimator, cost function
  qforge-simulator/  Statevector simulator with ZYZ-decomposition gate fusion
  qforge-bench/      Benchmark runner, JSON output, self-contained HTML report
  qforge-server/     HTTP API (Phase 8)
  qforge-cli/        Command-line interface
```

---

## Optimization passes

QForge runs eight passes in a configurable pipeline, iterating to fixed point:

| Pass | What it does |
|---|---|
| `identity_elimination` | Removes Rz(0), Rx(0), Ry(0) and other provably identity gates |
| `gate_cancellation` | Cancels adjacent inverse pairs: H·H, X·X, CX·CX, Rz(a)·Rz(-a) |
| `rotation_merging` | Merges consecutive rotations: Rz(a)·Rz(b) → Rz(a+b) |
| `commutation_analysis` | Reorders gates that commute to expose non-adjacent cancellable pairs |
| `gate_fusion` | Multiplies 2×2 unitary matrices of consecutive single-qubit gates, collapses to U3 via ZYZ decomposition |
| `native_gate_decomposition` | Decomposes H, Y, Z, S, T, Rx, Ry, SWAP, CZ, CCX, U1/U2/U3 into IBM native set |

The commutation analysis pass is the key insight: gates on disjoint qubits always commute, diagonal gates commute past CX controls, X/Rx commute past CX targets. This lets the optimizer find and cancel non-adjacent inverse pairs that naive cancellation would miss - achieving 40% reduction on circuits with no obvious adjacency.

---

## Hardware topologies

Three built-in topologies, or load any coupling graph from JSON:

**Linear** - each qubit connects only to its neighbours. Most constrained.
```
0 - 1 - 2 - 3 - 4
```

**Grid** - every qubit connects to up to four neighbours.
```
0 - 1 - 2
|   |   |
3 - 4 - 5
|   |   |
6 - 7 - 8
```

**Heavy-hex** - IBM's topology. Deliberately limited connectivity to reduce crosstalk noise. You pay in SWAPs but gain in gate fidelity.

Custom JSON topology:
```json
{
  "name": "my-chip",
  "qubit_count": 5,
  "coupling_map": [
    {"source": 0, "target": 1},
    {"source": 1, "target": 2}
  ],
  "native_gates": ["cx", "rz", "sx", "x"]
}
```

---

## Noise model

QForge ships with an IBM Brisbane-like noise model for fidelity estimation:

| Parameter | Value |
|---|---|
| T1 relaxation | 150–230 µs (per qubit) |
| T2 dephasing | 90–150 µs (per qubit) |
| Single-qubit gate error | 0.02–0.05% |
| CX gate error | 0.5–1.3% |
| Readout error | 1–5% |
| CX gate duration | 533ns |
| SX gate duration | 35.5ns |
| RZ gate duration | 0ns (virtual) |

The fidelity estimator computes: F = ∏(1 − εᵢ) × ∏exp(−t/T₂ᵢ), where the product runs over all gate errors and the exponential term accounts for decoherence over the total circuit duration. This is a conservative lower bound on actual process fidelity.

Four compilation strategies are evaluated against the noise model and the highest-fidelity result is selected.

---

## Example circuits

Ten circuits included in `circuits/`:

```
bell.qasm          2-qubit Bell state - entanglement baseline
ghz.qasm           5-qubit GHZ with deliberate redundancy
qft3.qasm          3-qubit quantum Fourier transform
qft5.qasm          5-qubit quantum Fourier transform
grover3.qasm       Grover search, marks |101>
bv5.qasm           Bernstein-Vazirani, secret string 10101
teleport.qasm      Quantum teleportation protocol
random5.qasm       Random 5-qubit circuit with redundancy
phase_kick.qasm    Phase kickback with redundancy
adder3.qasm        3-qubit ripple carry adder
```

---

## Roadmap

- [x] Phase 0 - Workspace skeleton, CI, example circuits
- [x] Phase 1 - DAG-based circuit depth (critical path BFS)
- [x] Phase 2 - Gate cancellation, rotation merging, commutation analysis
- [x] Phase 3 - Hardware topology, SABRE routing, SWAP insertion
- [x] Phase 4 - IBM noise model, fidelity estimation, multi-strategy cost function
- [x] Phase 5 - Statevector simulator, Bell state verified
- [x] Phase 6 - Benchmark suite, HTML dashboard, 10 circuits
- [x] Phase 7 - IBM native gate decomposition (CX, RZ, SX, X)
- [x] Phase 8 - Gate fusion via ZYZ matrix decomposition
- [ ] Phase 9 - Python bindings via PyO3
- [ ] Phase 10 - IBM Quantum backend adapter (real hardware submission)
- [ ] Phase 11 - Autonomous compilation (circuit feature extraction + strategy classifier)

---

## License

MIT - use it, fork it, extend it.

---

## Author

**Mihran Ali** .

[mihranali.vercel.app](https://mihranali.vercel.app) · [github.com/MA-V4](https://github.com/MA-V4) · [mihran.ali.v4@gmail.com](mailto:mihran.ali.v4@gmail.com)