use anyhow::{anyhow, Result};
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("compile") => cmd_compile(&args[2..]),
        Some("ir") => cmd_ir(&args[2..]),
        Some("simulate") => cmd_simulate(&args[2..]),
        Some("bench") => cmd_bench(&args[2..]),
        _ => {
            println!("QForge: an open-source quantum circuit compiler.\n");
            println!("  compile  <circuit.qasm> [--target linear|grid|heavy-hex] [--qubits N] [--native]");
            println!("  ir       <circuit.qasm>                  Dump IR as JSON");
            println!("  simulate <circuit.qasm> [--shots 1024]   Simulate statevector");
            println!("  bench    [--circuits circuits/] [--out results.json] [--html report.html]");
            Ok(())
        }
    }
}

fn cmd_compile(args: &[String]) -> Result<()> {
    let path = args.first().ok_or_else(|| {
        anyhow!(
            "usage: qforge compile <circuit.qasm> [--target linear|grid|heavy-hex] [--qubits N] [--native]"
        )
    })?;

    let source =
        std::fs::read_to_string(path).map_err(|e| anyhow!("cannot read {}: {}", path, e))?;

    let t0 = Instant::now();
    let circuit = qforge_parser::parse(&source).map_err(|e| anyhow!("parse error: {}", e))?;
    let parse_ms = t0.elapsed().as_millis();

    let input_gates = circuit.gate_count();
    let input_depth = circuit.depth();
    let input_2q = circuit.two_qubit_gate_count();
    let input_q = circuit.qubit_count();

    let t1 = Instant::now();
    let mut pm = qforge_optimizer::PassManager::new();
    pm.add_pass(qforge_optimizer::IdentityElimination)
        .add_pass(qforge_optimizer::GateCancellation)
        .add_pass(qforge_optimizer::RotationMerging)
        .add_pass(qforge_optimizer::CommutationAnalysis)
        .add_pass(qforge_optimizer::GateCancellation)
        .add_pass(qforge_optimizer::RotationMerging)
        .add_pass(qforge_optimizer::GateCancellation);
    let (optimized, report) = pm.run(circuit);
    let opt_ms = t1.elapsed().as_millis();

    // Native gate decomposition (optional — must run before routing)
    let native = args.iter().any(|a| a == "--native");
    let optimized = if native {
        let mut pm2 = qforge_optimizer::PassManager::new();
        pm2.add_pass(qforge_optimizer::NativeGateDecomposition)
            .add_pass(qforge_optimizer::GateFusion)
            .add_pass(qforge_optimizer::RotationMerging)
            .add_pass(qforge_optimizer::GateCancellation);
        let (decomposed, _) = pm2.run(optimized);
        decomposed
    } else {
        optimized
    };

    // Routing (optional)
    let target_name = flag(args, "--target");
    let qubit_count: usize = flag(args, "--qubits")
        .and_then(|s| s.parse().ok())
        .unwrap_or(optimized.qubit_count().max(5));

    let routing_result = target_name.map(|t| {
        let topology = match t {
            "grid" => qforge_routing::HardwareTopology::grid(3, 3),
            "heavy-hex" => qforge_routing::HardwareTopology::heavy_hex(qubit_count),
            _ => qforge_routing::HardwareTopology::linear(qubit_count),
        };
        let mapping = qforge_routing::allocate_qubits(&optimized, &topology);
        let (routed, r_report) = qforge_routing::route(&optimized, &topology, &mapping);
        (topology, mapping, routed, r_report)
    });

    println!("\nQForge v{}\n", env!("CARGO_PKG_VERSION"));
    println!("  Parsing...        done  ({}ms)", parse_ms);
    println!("  Optimizing...     done  ({}ms)", opt_ms);
    if native {
        println!("  Decomposing...    done  (IBM native: CX, RZ, SX, X)");
    }
    if routing_result.is_some() {
        println!("  Routing...        done");
    }

    let w = 45;
    println!("\n  {}", "=".repeat(w));
    println!("  COMPILATION REPORT");
    println!("  {}", "=".repeat(w));
    println!();
    println!("  Input");
    println!("    Qubits          {}", input_q);
    println!("    Gates           {}", input_gates);
    println!("    2Q gates        {}", input_2q);
    println!("    Depth           {}", input_depth);
    println!();
    println!("  Output (optimized)");
    println!(
        "    Gates           {}      ({:+.1}%)",
        report.output_gates,
        -report.gate_reduction_pct()
    );
    println!(
        "    2Q gates        {}      ({})",
        report.output_2q,
        if report.input_2q > 0 {
            format!(
                "{:+.1}%",
                -(report.input_2q as f64 - report.output_2q as f64) / report.input_2q as f64
                    * 100.0
            )
        } else {
            "n/a".into()
        }
    );
    println!(
        "    Depth           {}      ({:+.1}%)",
        report.output_depth,
        -report.depth_reduction_pct()
    );
    println!();

    if native {
        let native_gates = optimized.gate_count();
        let native_2q = optimized.two_qubit_gate_count();
        let native_depth = optimized.depth();
        println!("  Output (native IBM gates)");
        println!("    Gates           {}", native_gates);
        println!("    2Q gates        {}", native_2q);
        println!("    Depth           {}", native_depth);
        println!("    Gate set        CX, RZ, SX, X");
        println!();
    }

    if let Some((topology, mapping, _routed, r_report)) = &routing_result {
        println!("  Hardware ({}):", topology.name);
        println!("    Qubits          {}", topology.qubit_count);
        println!("    Mapping         {:?}", mapping);
        println!("    SWAPs inserted  {}", r_report.swaps_inserted);
        println!("    SWAP cost (CX)  {}", r_report.swap_cost_in_cx);
        println!("    Routed gates    {}", r_report.final_gate_count);
        println!();
    }

    // Noise-aware fidelity estimate
    let noise = qforge_noise::NoiseModel::ibm_brisbane_like(optimized.qubit_count().max(5));
    let strategies = qforge_noise::cost::evaluate_strategies(&optimized, &noise);

    println!("  Noise-aware candidates  ({}):", noise.name);
    println!(
        "    {:<22} {:>8}  {:>8}  {:>10}",
        "Strategy", "Gates", "Depth", "Fidelity"
    );
    println!("    {}", "-".repeat(54));
    for (i, (s, _score)) in strategies.iter().enumerate() {
        println!(
            "    {:<22} {:>8}  {:>8}  {:>9.2}%  {}",
            s.name,
            s.gates,
            s.depth,
            s.fidelity.fidelity * 100.0,
            if i == 0 { "<-- selected" } else { "" },
        );
    }
    if let Some((best, _)) = strategies.first() {
        println!();
        println!("  Selected: {}", best.name);
        println!(
            "    Est. fidelity:   {:.2}%",
            best.fidelity.fidelity * 100.0
        );
        println!(
            "    Circuit time:    {:.0}ns",
            best.fidelity.total_duration_ns
        );
        println!(
            "    Readout error:   {:.2}%",
            best.fidelity.readout_error * 100.0
        );
        println!(
            "    Decoherence:     {:.2}%",
            best.fidelity.decoherence_error * 100.0
        );
    }
    println!();

    println!("  Optimization breakdown");
    let mut total: i64 = 0;
    for pass in &report.passes {
        if pass.gates_removed != 0 {
            println!("    {:<30} -{}", pass.name, pass.gates_removed);
            total += pass.gates_removed;
        }
    }
    println!("    {}", "-".repeat(38));
    println!("    {:<30} -{}", "Total", total);
    println!();
    println!("  Time               {}ms", report.time_ms);
    println!("  {}\n", "=".repeat(w));

    Ok(())
}

fn cmd_simulate(args: &[String]) -> Result<()> {
    let path = args
        .first()
        .ok_or_else(|| anyhow!("usage: qforge simulate <circuit.qasm> [--shots 1024]"))?;

    let shots: usize = flag(args, "--shots")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024);

    let source =
        std::fs::read_to_string(path).map_err(|e| anyhow!("cannot read {}: {}", path, e))?;
    let circuit = qforge_parser::parse(&source).map_err(|e| anyhow!("parse error: {}", e))?;

    println!("\nQForge v{}\n", env!("CARGO_PKG_VERSION"));
    println!("  Circuit:  {}", path);
    println!("  Qubits:   {}", circuit.qubit_count());
    println!("  Gates:    {}", circuit.gate_count());
    println!("  Shots:    {}\n", shots);

    let result = qforge_simulator::simulate(&circuit, shots);

    println!("  Results");
    println!("  {}", "=".repeat(40));

    let mut outcomes: Vec<(&String, &usize)> = result.counts.iter().collect();
    outcomes.sort_by(|a, b| b.1.cmp(a.1));

    let _bar_max = outcomes.first().map(|(_, c)| **c).unwrap_or(1);

    for (bitstring, count) in &outcomes {
        let prob = **count as f64 / shots as f64;
        let bars = (prob * 20.0).round() as usize;
        let bar = "█".repeat(bars);
        println!(
            "  |{}>  {:>6}  {:>6.2}%  {}",
            bitstring,
            count,
            prob * 100.0,
            bar
        );
    }

    println!("  {}", "=".repeat(40));

    if let Some((state, prob)) = result.most_probable() {
        println!("\n  Most probable: |{}>  ({:.2}%)\n", state, prob * 100.0);
    }

    Ok(())
}

fn cmd_ir(args: &[String]) -> Result<()> {
    let path = args
        .first()
        .ok_or_else(|| anyhow!("usage: qforge ir <circuit.qasm>"))?;
    let source = std::fs::read_to_string(path)?;
    let circuit = qforge_parser::parse(&source).map_err(|e| anyhow!("{}", e))?;
    println!("{}", serde_json::to_string_pretty(&circuit)?);
    Ok(())
}

fn cmd_bench(args: &[String]) -> Result<()> {
    let circuits_dir = flag(args, "--circuits").unwrap_or("circuits");
    let out_json = flag(args, "--out").unwrap_or("bench_results.json");
    let out_html = flag(args, "--html").unwrap_or("bench_report.html");

    println!(
        "\nQForge v{} — Benchmark Suite\n",
        env!("CARGO_PKG_VERSION")
    );
    println!("  Circuits: {}", circuits_dir);
    println!("  Running...\n");

    let suite = qforge_bench::run_suite(circuits_dir)?;

    println!();
    println!("  =============================================");
    println!("  SUMMARY");
    println!("  =============================================");
    println!("  Circuits compiled:    {}", suite.total_circuits);
    println!("  Avg gate reduction:   {:.1}%", suite.avg_gate_reduction);
    println!("  Avg fidelity:         {:.1}%", suite.avg_fidelity);
    println!("  Total compile time:   {}ms", suite.total_compile_ms);
    println!("  =============================================\n");

    let json = serde_json::to_string_pretty(&suite)?;
    std::fs::write(out_json, &json)?;
    println!("  Results saved to: {}", out_json);

    let html = qforge_bench::generate_report(&suite);
    std::fs::write(out_html, &html)?;
    println!("  HTML report:      {}", out_html);
    println!(
        "  Open {} in your browser to see the dashboard.\n",
        out_html
    );

    Ok(())
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].as_str())
}
