use crate::metrics::{BenchmarkResult, BenchmarkSuite};
use anyhow::Result;
use std::time::Instant;

pub fn run_suite(circuits_dir: &str) -> Result<BenchmarkSuite> {
    let dir = std::path::Path::new(circuits_dir);
    if !dir.exists() {
        anyhow::bail!("circuits directory not found: {}", circuits_dir);
    }

    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "qasm").unwrap_or(false))
        .collect();

    entries.sort();

    let mut results = Vec::new();

    for path in &entries {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let circuit_file = path.to_string_lossy().to_string();

        match compile_circuit(path) {
            Ok(result) => {
                println!("  {:.<35} {:>6} gates -> {:>6} gates  ({:+.1}%)  {:.1}% fidelity",
                    name,
                    result.input_gates,
                    result.output_gates,
                    -result.gate_reduction_pct,
                    result.fidelity_pct,
                );
                results.push(result);
            }
            Err(e) => {
                println!("  {:.<35} FAILED: {}", name, e);
                results.push(BenchmarkResult {
                    name,
                    circuit_file,
                    input_qubits:        0,
                    input_gates:         0,
                    input_depth:         0,
                    input_2q:            0,
                    output_gates:        0,
                    output_depth:        0,
                    output_2q:           0,
                    gate_reduction_pct:  0.0,
                    depth_reduction_pct: 0.0,
                    fidelity_pct:        0.0,
                    compile_ms:          0,
                    status:              format!("error: {}", e),
                });
            }
        }
    }

    Ok(BenchmarkSuite::new(results))
}

fn compile_circuit(path: &std::path::Path) -> anyhow::Result<BenchmarkResult> {
    let name         = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
    let circuit_file = path.to_string_lossy().to_string();
    let source       = std::fs::read_to_string(path)?;

    let t0      = Instant::now();
    let circuit = qforge_parser::parse(&source)
        .map_err(|e| anyhow::anyhow!("parse error: {}", e))?;

    let input_qubits = circuit.qubit_count();
    let input_gates  = circuit.gate_count();
    let input_depth  = circuit.depth();
    let input_2q     = circuit.two_qubit_gate_count();

    let mut pm = qforge_optimizer::PassManager::new();
    pm.add_pass(qforge_optimizer::IdentityElimination)
      .add_pass(qforge_optimizer::GateCancellation)
      .add_pass(qforge_optimizer::RotationMerging)
      .add_pass(qforge_optimizer::CommutationAnalysis)
      .add_pass(qforge_optimizer::GateCancellation)
      .add_pass(qforge_optimizer::RotationMerging)
      .add_pass(qforge_optimizer::GateCancellation);

    let (optimized, report) = pm.run(circuit);
    let compile_ms = t0.elapsed().as_millis();

    let noise    = qforge_noise::NoiseModel::ibm_brisbane_like(optimized.qubit_count().max(5));
    let fidelity = qforge_noise::estimator::estimate_fidelity(&optimized, &noise);

    Ok(BenchmarkResult {
        name,
        circuit_file,
        input_qubits,
        input_gates,
        input_depth,
        input_2q,
        output_gates:        report.output_gates,
        output_depth:        report.output_depth,
        output_2q:           report.output_2q,
        gate_reduction_pct:  report.gate_reduction_pct(),
        depth_reduction_pct: report.depth_reduction_pct(),
        fidelity_pct:        fidelity.fidelity * 100.0,
        compile_ms,
        status:              "ok".into(),
    })
}