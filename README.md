# QForge

**An open-source quantum circuit compiler and optimizer.**

QForge takes OpenQASM 2.0 circuits, optimizes them through a configurable pass pipeline, maps them to real hardware topologies, and benchmarks the results against Qiskit and Cirq.

## Quick start

```bash
cargo build

# Compile and optimize a circuit
cargo run --bin qforge -- compile circuits/ghz.qasm

# Dump the IR as JSON
cargo run --bin qforge -- ir circuits/bell.qasm
```

## Architecture

```
crates/
  qforge-ir/         Gate types, circuit representation, qubit references
  qforge-parser/     OpenQASM 2.0 lexer and recursive descent parser
  qforge-optimizer/  Pass manager: gate cancellation, rotation merging, identity elimination
  qforge-routing/    Hardware topology, qubit allocation, SWAP insertion   [Phase 3]
  qforge-noise/      Noise models, fidelity estimation                     [Phase 4]
  qforge-simulator/  Statevector simulator                                 [Phase 5]
  qforge-bench/      MQTBench integration, metrics, comparison             [Phase 6]
  qforge-server/     HTTP API for dashboard                                [Phase 5]
  qforge-cli/        Command-line interface
```

## License

MIT
