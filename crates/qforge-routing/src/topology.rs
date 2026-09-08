use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingEdge {
    pub source: usize,
    pub target: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTopology {
    pub name: String,
    pub qubit_count: usize,
    pub coupling_map: Vec<CouplingEdge>,
    pub native_gates: Vec<String>,
    pub gate_duration: HashMap<String, f64>,
}

impl HardwareTopology {
    pub fn are_adjacent(&self, q0: usize, q1: usize) -> bool {
        self.coupling_map
            .iter()
            .any(|e| (e.source == q0 && e.target == q1) || (e.source == q1 && e.target == q0))
    }

    pub fn neighbours(&self, qubit: usize) -> Vec<usize> {
        self.coupling_map
            .iter()
            .flat_map(|e| {
                if e.source == qubit {
                    Some(e.target)
                } else if e.target == qubit {
                    Some(e.source)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Shortest path length between two physical qubits (BFS).
    pub fn distance(&self, src: usize, dst: usize) -> usize {
        if src == dst {
            return 0;
        }
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((src, 0usize));
        visited.insert(src);
        while let Some((node, dist)) = queue.pop_front() {
            for nb in self.neighbours(node) {
                if nb == dst {
                    return dist + 1;
                }
                if visited.insert(nb) {
                    queue.push_back((nb, dist + 1));
                }
            }
        }
        usize::MAX // disconnected
    }

    /// Pre-compute full distance matrix for faster routing.
    pub fn distance_matrix(&self) -> Vec<Vec<usize>> {
        let n = self.qubit_count;
        (0..n)
            .map(|i| (0..n).map(|j| self.distance(i, j)).collect())
            .collect()
    }

    // Built-in topologies

    pub fn linear(n: usize) -> Self {
        Self {
            name: format!("linear-{}", n),
            qubit_count: n,
            coupling_map: (0..n - 1)
                .map(|i| CouplingEdge {
                    source: i,
                    target: i + 1,
                })
                .collect(),
            native_gates: vec!["cx".into(), "rz".into(), "sx".into(), "x".into()],
            gate_duration: HashMap::new(),
        }
    }

    pub fn grid(rows: usize, cols: usize) -> Self {
        let n = rows * cols;
        let mut edges = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let idx = r * cols + c;
                if c + 1 < cols {
                    edges.push(CouplingEdge {
                        source: idx,
                        target: idx + 1,
                    });
                }
                if r + 1 < rows {
                    edges.push(CouplingEdge {
                        source: idx,
                        target: idx + cols,
                    });
                }
            }
        }
        Self {
            name: format!("grid-{}x{}", rows, cols),
            qubit_count: n,
            coupling_map: edges,
            native_gates: vec!["cx".into(), "rz".into(), "sx".into(), "x".into()],
            gate_duration: HashMap::new(),
        }
    }

    pub fn heavy_hex(n: usize) -> Self {
        // Simplified heavy-hex: IBM-style topology with degree-2 and degree-3 nodes.
        // Using a 7-qubit heavy-hex as the base unit, scaling linearly.
        // For a real implementation this would use IBM's exact coupling maps.
        let mut edges = Vec::new();
        // Heavy-hex pattern: chains connected by bridges
        for i in (0..n.saturating_sub(1)).step_by(2) {
            edges.push(CouplingEdge {
                source: i,
                target: i + 1,
            });
            if i + 2 < n {
                edges.push(CouplingEdge {
                    source: i + 1,
                    target: i + 2,
                });
            }
            if i + 3 < n {
                edges.push(CouplingEdge {
                    source: i + 2,
                    target: i + 3,
                });
            }
        }
        Self {
            name: format!("heavy-hex-{}", n),
            qubit_count: n,
            coupling_map: edges,
            native_gates: vec!["cx".into(), "rz".into(), "sx".into(), "x".into()],
            gate_duration: HashMap::new(),
        }
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
