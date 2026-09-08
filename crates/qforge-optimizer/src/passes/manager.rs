use crate::pass::{Pass, PassReport};
use crate::report::{OptimizationReport, PassSummary};
use qforge_ir::Circuit;
use std::time::Instant;

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
    max_iterations: usize,
}

impl PassManager {
    pub fn new() -> Self {
        Self { passes: Vec::new(), max_iterations: 10 }
    }

    pub fn add_pass<P: Pass + 'static>(&mut self, pass: P) -> &mut Self {
        self.passes.push(Box::new(pass));
        self
    }

    pub fn add_pass_boxed(&mut self, pass: Box<dyn Pass>) -> &mut Self {
    self.passes.push(pass);
    self
    }
    
    pub fn run(&self, circuit: Circuit) -> (Circuit, OptimizationReport) {
        let start = Instant::now();
        let input_gates = circuit.gate_count();
        let input_depth = circuit.depth();
        let input_2q    = circuit.two_qubit_gate_count();

        let mut current = circuit;
        let mut all_reports: Vec<PassReport> = Vec::new();

        for _ in 0..self.max_iterations {
            let before = current.gate_count();
            for pass in &self.passes {
                let (next, report) = pass.run(current);
                current = next;
                all_reports.push(report);
            }
            if current.gate_count() == before { break; } // fixed point
        }

        let report = OptimizationReport {
            input_gates,
            output_gates: current.gate_count(),
            input_depth,
            output_depth: current.depth(),
            input_2q,
            output_2q:    current.two_qubit_gate_count(),
            passes: all_reports.into_iter().map(|r| PassSummary {
                name:          r.pass_name,
                gates_removed: r.gates_removed,
                reason:        r.reason,
            }).collect(),
            time_ms: start.elapsed().as_millis(),
        };

        (current, report)
    }
}

impl Default for PassManager {
    fn default() -> Self { Self::new() }
}
