use crate::event::{Accounting, Event, EventKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeState {
    pub discovered: bool,
    pub expanded: bool,
    pub frontier_entries: u32,
    pub g: Option<u64>,
    pub h: Option<u64>,
    pub f: Option<u64>,
    pub parent: Option<usize>,
    pub expanded_at: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayState {
    pub nodes: Vec<NodeState>,
    pub current: Option<usize>,
    pub path: Vec<usize>,
    pub completed: bool,
    pub found: bool,
    pub accounting: Accounting,
}

impl ReplayState {
    pub fn new(node_count: usize) -> Self {
        Self {
            nodes: vec![NodeState::default(); node_count],
            current: None,
            path: Vec::new(),
            completed: false,
            found: false,
            accounting: Accounting::default(),
        }
    }
    pub fn apply(&mut self, event: &Event) {
        let mut first_cost = false;
        let mut first_parent = false;
        match &event.kind {
            EventKind::NodeDiscovered { node } => self.nodes[*node].discovered = true,
            EventKind::NodeExpanded { node } => {
                self.nodes[*node].expanded = true;
                self.nodes[*node].expanded_at = Some(event.sequence);
                self.current = Some(*node);
            }
            EventKind::FrontierAdded { node } => self.nodes[*node].frontier_entries += 1,
            EventKind::FrontierRemoved { node } => self.nodes[*node].frontier_entries -= 1,
            EventKind::CostUpdated { node, g, h, f } => {
                let n = &mut self.nodes[*node];
                first_cost = n.g.is_none();
                n.g = Some(*g);
                n.h = Some(*h);
                n.f = Some(*f);
            }
            EventKind::ParentUpdated { node, parent } => {
                first_parent = self.nodes[*node].parent.is_none();
                self.nodes[*node].parent = Some(*parent);
            }
            EventKind::PathReconstructed { path, .. } => self.path = path.clone(),
            EventKind::RunCompleted { found } => {
                self.completed = true;
                self.found = *found;
            }
            _ => {}
        }
        self.accounting.apply(&event.kind, first_cost, first_parent);
    }
}

/// Cursor is an event count (0 means before the first event).
pub struct Replay<'a> {
    events: &'a [Event],
    snapshots: Vec<(usize, ReplayState)>,
    pub checkpoint_interval: usize,
    pub cursor: usize,
    pub state: ReplayState,
}
impl<'a> Replay<'a> {
    pub fn new(events: &'a [Event], node_count: usize) -> Self {
        let checkpoint_interval = 256.max(events.len().div_ceil(32));
        let mut state = ReplayState::new(node_count);
        let mut snapshots = vec![(0, state.clone())];
        for (i, event) in events.iter().enumerate() {
            state.apply(event);
            if (i + 1) % checkpoint_interval == 0 {
                snapshots.push((i + 1, state.clone()));
            }
        }
        Self {
            events,
            snapshots,
            checkpoint_interval,
            cursor: 0,
            state: ReplayState::new(node_count),
        }
    }
    pub fn seek(&mut self, target: usize) -> Result<(), String> {
        if target > self.events.len() {
            return Err("seek exceeds event count".into());
        }
        let (base, snapshot) = &self.snapshots[target / self.checkpoint_interval];
        self.state = snapshot.clone();
        for event in &self.events[*base..target] {
            self.state.apply(event);
        }
        self.cursor = target;
        Ok(())
    }
    pub fn step_forward(&mut self) -> bool {
        if let Some(event) = self.events.get(self.cursor) {
            self.state.apply(event);
            self.cursor += 1;
            true
        } else {
            false
        }
    }
    pub fn step_backward(&mut self) -> bool {
        if self.cursor == 0 {
            false
        } else {
            self.seek(self.cursor - 1).expect("valid cursor");
            true
        }
    }
    pub fn step_expansion(&mut self) -> bool {
        while self.cursor < self.events.len() {
            let expanded = matches!(
                self.events[self.cursor].kind,
                EventKind::NodeExpanded { .. }
            );
            self.step_forward();
            if expanded {
                return true;
            }
        }
        false
    }
}
