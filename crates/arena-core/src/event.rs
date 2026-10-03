use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    RunStarted,
    NodeDiscovered { node: usize },
    NodeExpanded { node: usize },
    FrontierAdded { node: usize },
    FrontierRemoved { node: usize },
    EdgeEvaluated { from: usize, to: usize, cost: u64 },
    RelaxationCompared { node: usize, candidate: u64 },
    CostUpdated { node: usize, g: u64, h: u64, f: u64 },
    ParentUpdated { node: usize, parent: usize },
    GoalReached { node: usize },
    PathReconstructed { path: Vec<usize>, cost: u64 },
    RunCompleted { found: bool },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub sequence: usize,
    pub tick: u64,
    #[serde(flatten)]
    pub kind: EventKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metrics {
    pub expanded: u64,
    pub generated: u64,
    pub edges_evaluated: u64,
    pub frontier_pushes: u64,
    pub frontier_pops: u64,
    pub cost_updates: u64,
    pub relaxation_comparisons: u64,
    pub peak_frontier: u64,
    pub peak_discovered: u64,
    pub peak_state_entries: u64,
    pub path_length: Option<u64>,
    pub path_cost: Option<u64>,
    pub logical_ops: u64,
}

/// The single accounting policy shared by execution and replay.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accounting {
    pub metrics: Metrics,
    pub frontier: u64,
    pub finite_cost_entries: u64,
    pub parent_entries: u64,
}

impl Accounting {
    pub fn apply(&mut self, kind: &EventKind, first_cost: bool, first_parent: bool) {
        let m = &mut self.metrics;
        match kind {
            EventKind::NodeDiscovered { .. } => {
                m.generated += 1;
                m.peak_discovered = m.generated;
            }
            EventKind::NodeExpanded { .. } => {
                m.expanded += 1;
                m.logical_ops += 1;
            }
            EventKind::FrontierAdded { .. } => {
                self.frontier += 1;
                m.frontier_pushes += 1;
                m.logical_ops += 1;
                m.peak_frontier = m.peak_frontier.max(self.frontier);
            }
            EventKind::FrontierRemoved { .. } => {
                self.frontier -= 1;
                m.frontier_pops += 1;
                m.logical_ops += 1;
            }
            EventKind::EdgeEvaluated { .. } => {
                m.edges_evaluated += 1;
                m.logical_ops += 1;
            }
            EventKind::RelaxationCompared { .. } => {
                m.relaxation_comparisons += 1;
                m.logical_ops += 1;
            }
            EventKind::CostUpdated { .. } => {
                m.cost_updates += 1;
                if first_cost {
                    self.finite_cost_entries += 1;
                }
            }
            EventKind::ParentUpdated { .. } => {
                if first_parent {
                    self.parent_entries += 1;
                }
            }
            EventKind::PathReconstructed { path, cost } => {
                m.path_length = Some(path.len().saturating_sub(1) as u64);
                m.path_cost = Some(*cost);
            }
            _ => {}
        }
        m.peak_state_entries = m
            .peak_state_entries
            .max(m.generated + self.finite_cost_entries + self.parent_entries + self.frontier);
    }
}
