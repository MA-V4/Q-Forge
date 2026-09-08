// Cost function and multi-strategy compilation.
// Generates candidate circuits using different pass orderings,
// scores each against the noise model, returns the best.

use crate::estimator::{estimate_fidelity, FidelityEstimate};
use crate::model::NoiseModel;
use qforge_ir::Circuit;
use qforge_optimizer::{
    CommutationAnalysis, GateCancellation, IdentityElimination, PassManager, RotationMerging,
};

#[derive(Debug, Clone)]
pub struct CompilationStrategy {
    pub name: String,
    pub fidelity: FidelityEstimate,
    pub gates: usize,
    pub depth: usize,
    pub two_q: usize,
}

pub struct CostFunction {
    pub gate_weight: f64,
    pub depth_weight: f64,
    pub fidelity_weight: f64,
    pub two_q_weight: f64,
}

impl Default for CostFunction {
    fn default() -> Self {
        Self {
            gate_weight: 0.1,
            depth_weight: 0.1,
            fidelity_weight: 0.6,
            two_q_weight: 0.2,
        }
    }
}

impl CostFunction {
    pub fn score(&self, strategy: &CompilationStrategy, baseline_gates: usize) -> f64 {
        let gate_score = 1.0 - strategy.gates as f64 / baseline_gates.max(1) as f64;
        let fidelity_score = strategy.fidelity.fidelity;
        let two_q_penalty = strategy.two_q as f64 * 0.01;

        self.fidelity_weight * fidelity_score + self.gate_weight * gate_score - two_q_penalty
    }
}

/// Run multiple compilation strategies, score each, return ranked results.
pub fn evaluate_strategies(
    circuit: &Circuit,
    noise: &NoiseModel,
) -> Vec<(CompilationStrategy, f64)> {
    let baseline = circuit.gate_count();

    let strategies = build_strategies(circuit);
    let cost_fn = CostFunction::default();

    let mut scored: Vec<(CompilationStrategy, f64)> = strategies
        .into_iter()
        .map(|s| {
            let score = cost_fn.score(&s, baseline);
            (s, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

fn build_strategies(circuit: &Circuit) -> Vec<CompilationStrategy> {
    let configs: Vec<(&str, Vec<Box<dyn qforge_optimizer::Pass>>)> = vec![
        (
            "aggressive",
            vec![
                Box::new(IdentityElimination),
                Box::new(GateCancellation),
                Box::new(RotationMerging),
                Box::new(CommutationAnalysis),
                Box::new(GateCancellation),
                Box::new(RotationMerging),
                Box::new(GateCancellation),
            ],
        ),
        (
            "balanced",
            vec![
                Box::new(IdentityElimination),
                Box::new(GateCancellation),
                Box::new(RotationMerging),
                Box::new(GateCancellation),
            ],
        ),
        (
            "light",
            vec![Box::new(GateCancellation), Box::new(IdentityElimination)],
        ),
        (
            "commutation_first",
            vec![
                Box::new(CommutationAnalysis),
                Box::new(GateCancellation),
                Box::new(RotationMerging),
                Box::new(GateCancellation),
            ],
        ),
    ];

    configs
        .into_iter()
        .map(|(name, passes)| {
            let mut pm = PassManager::new();
            for pass in passes {
                pm.add_pass_boxed(pass);
            }
            let (compiled, _) = pm.run(circuit.clone());
            let noise_ibm = crate::NoiseModel::ibm_brisbane_like(compiled.qubit_count().max(5));
            let fidelity = estimate_fidelity(&compiled, &noise_ibm);

            CompilationStrategy {
                name: name.into(),
                gates: compiled.gate_count(),
                depth: compiled.depth(),
                two_q: compiled.two_qubit_gate_count(),
                fidelity,
            }
        })
        .collect()
}
