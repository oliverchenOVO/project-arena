//! Domain event reducer with a bounded checkpoint budget; contains no algorithms.
use crate::domain::{ArenaDomain, DomainEvent, DomainLog};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeState {
    pub discovered: bool,
    pub visited: bool,
    pub parent: Option<usize>,
    pub distance: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeState {
    pub evaluated: u64,
    pub selected: bool,
    pub rejected: bool,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub values: Vec<i64>,
    pub nodes: Vec<NodeState>,
    pub edges: Vec<EdgeState>,
    pub comparisons: u64,
    pub reads: u64,
    pub writes: u64,
    pub swaps: u64,
    pub visited: u64,
    pub edges_examined: u64,
    pub total_weight: u64,
    pub selected_edges: u64,
    pub completed: bool,
    pub path: Vec<usize>,
}
impl State {
    pub fn new(log: &DomainLog) -> Self {
        let values =
            serde_json::from_value::<Vec<i64>>(log.problem["values"].clone()).unwrap_or_default();
        let n = log.problem["nodes"]
            .as_array()
            .map_or(values.len(), Vec::len);
        let edges = log.problem["edges"].as_array().map_or(0, Vec::len);
        Self {
            values,
            nodes: vec![NodeState::default(); n],
            edges: vec![EdgeState::default(); edges],
            comparisons: 0,
            reads: 0,
            writes: 0,
            swaps: 0,
            visited: 0,
            edges_examined: 0,
            total_weight: 0,
            selected_edges: 0,
            completed: false,
            path: vec![],
        }
    }
    pub fn apply(&mut self, e: &DomainEvent) {
        let p = &e.payload;
        let index = |k: &str| p[k].as_u64().expect("validated integer") as usize;
        match e.event_type.as_str() {
            "Read" => self.reads += 1,
            "Compare" => self.comparisons += 1,
            "Write" => {
                self.values[index("index")] = p["value"].as_i64().expect("validated value");
                self.writes += 1
            }
            "Swap" => {
                self.values.swap(index("left"), index("right"));
                self.swaps += 1;
                self.writes += 2
            }
            "NodeDiscovered" => {
                let n = &mut self.nodes[index("node")];
                n.discovered = true;
                if p.get("distance").is_some() {
                    n.distance = p["distance"].as_u64();
                }
            }
            "NodeVisited" => {
                self.nodes[index("node")].visited = true;
                self.visited += 1
            }
            "EdgeEvaluated" => {
                self.edges[index("edge")].evaluated += 1;
                self.edges_examined += 1
            }
            "EdgeRelaxed" => {
                let n = &mut self.nodes[index("to")];
                n.parent = Some(index("from"));
                n.distance = p["distance"].as_u64();
            }
            "EdgeSelected" => {
                self.edges[index("edge")].selected = true;
                if p["parent"] == true {
                    self.nodes[index("to")].parent = Some(index("from"));
                }
            }
            "EdgeRejected" => {
                let e = &mut self.edges[index("edge")];
                e.rejected = true;
                e.reason = p["reason"].as_str().unwrap_or_default().into();
            }
            "MstEdgeAdded" => {
                self.edges[index("edge")].selected = true;
                self.total_weight += p["weight"].as_u64().expect("weight");
                self.selected_edges += 1
            }
            "GoalReached" => {
                self.path = serde_json::from_value(p["path"].clone()).unwrap_or_default()
            }
            "RunCompleted" => self.completed = true,
            _ => {}
        }
    }
    pub fn check_metrics(&self, log: &DomainLog) -> Result<(), String> {
        let pairs = match log.domain {
            ArenaDomain::Sorting => vec![
                ("comparisons", self.comparisons),
                ("reads", self.reads),
                ("writes", self.writes),
                ("swaps", self.swaps),
            ],
            ArenaDomain::Graph => vec![
                ("visited", self.visited),
                ("edgesExamined", self.edges_examined),
                ("totalWeight", self.total_weight),
                ("selectedEdges", self.selected_edges),
            ],
            _ => vec![],
        };
        for (k, v) in pairs {
            if log.metrics[k] != v {
                return Err(format!("replay accounting mismatch {k}"));
            }
        }
        Ok(())
    }
}
pub struct DomainReplay<'a> {
    log: &'a DomainLog,
    pub state: State,
    pub cursor: usize,
    pub interval: usize,
    pub checkpoints: Vec<State>,
}
impl<'a> DomainReplay<'a> {
    pub fn new(log: &'a DomainLog) -> Self {
        let interval = 256.max(log.events.len().div_ceil(32));
        let mut state = State::new(log);
        let mut checkpoints = vec![state.clone()];
        for (i, e) in log.events.iter().enumerate() {
            state.apply(e);
            if (i + 1) % interval == 0 {
                checkpoints.push(state.clone());
            }
        }
        Self {
            log,
            state: State::new(log),
            cursor: 0,
            interval,
            checkpoints,
        }
    }
    pub fn seek(&mut self, target: usize) -> Result<(), String> {
        if target > self.log.events.len() {
            return Err("seek exceeds event count".into());
        }
        let base = target / self.interval;
        self.state = self.checkpoints[base].clone();
        self.cursor = base * self.interval;
        for e in &self.log.events[self.cursor..target] {
            self.state.apply(e);
        }
        self.cursor = target;
        Ok(())
    }
}
