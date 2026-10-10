//! Deterministic graph model, layout and shared traversal/priority search adapters.
use crate::{
    domain::{ArenaDomain, DomainLog, Recorder},
    grid::SeededRng,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap, VecDeque},
    time::Instant,
};
pub const ALGORITHMS: [&str; 6] = ["bfs", "dfs", "dijkstra", "astar", "prim", "kruskal"];
pub const FAMILIES: [&str; 8] = [
    "random",
    "sparse",
    "dense",
    "grid_like",
    "tree",
    "clustered",
    "bridge",
    "disconnected",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphConfig {
    pub size: usize,
    pub seed: u64,
    pub family: String,
    pub directed: bool,
    pub weighted: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub x: u64,
    pub y: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub directed: bool,
    pub weighted: bool,
    pub start: usize,
    pub goal: usize,
    pub config: GraphConfig,
}
impl Graph {
    pub fn validate(&self) -> Result<(), String> {
        let n = self.nodes.len();
        if n == 0
            || n > 256
            || self.edges.len() > n * (n - 1)
            || self.start >= n
            || self.goal >= n
            || self.nodes.iter().any(|p| p.x > 1000 || p.y > 1000)
        {
            return Err("invalid graph bounds".into());
        }
        let mut seen = BTreeSet::new();
        for e in &self.edges {
            let key = if self.directed {
                (e.from, e.to)
            } else {
                (e.from.min(e.to), e.from.max(e.to))
            };
            if e.from >= n
                || e.to >= n
                || e.from == e.to
                || e.weight == 0
                || e.weight > 1_000_000
                || (!self.weighted && e.weight != 1)
                || !seen.insert(key)
            {
                return Err("invalid or duplicate graph edge".into());
            }
        }
        Ok(())
    }
    pub fn adjacency(&self) -> Vec<Vec<(usize, usize, u64)>> {
        let mut a = vec![vec![]; self.nodes.len()];
        for (i, e) in self.edges.iter().enumerate() {
            a[e.from].push((e.to, i, e.weight));
            if !self.directed {
                a[e.to].push((e.from, i, e.weight));
            }
        }
        for row in &mut a {
            row.sort_by_key(|&(node, edge, _)| (node, edge));
        }
        a
    }
    pub fn connected(&self) -> bool {
        let a = self.adjacency();
        let mut seen = vec![false; a.len()];
        let mut q = vec![0];
        seen[0] = true;
        while let Some(u) = q.pop() {
            for &(v, _, _) in &a[u] {
                if !seen[v] {
                    seen[v] = true;
                    q.push(v);
                }
            }
        }
        seen.iter().all(|v| *v)
    }
}
pub fn generate(c: &GraphConfig) -> Result<Graph, String> {
    if !(1..=256).contains(&c.size) || !FAMILIES.contains(&c.family.as_str()) {
        return Err("invalid graph configuration".into());
    }
    let n = c.size;
    let mut rng = SeededRng::new(c.seed);
    let side = (n as f64).sqrt().ceil() as usize;
    let nodes = (0..n)
        .map(|i| {
            let (mut x, mut y) = ((i % side) * 1000 / side, (i / side) * 1000 / side);
            if c.family != "grid_like" {
                x = (x + 25 + rng.below((700 / side).max(1) as u64) as usize).min(1000);
                y = (y + 25 + rng.below((700 / side).max(1) as u64) as usize).min(1000);
            }
            Node {
                x: x as u64,
                y: y as u64,
            }
        })
        .collect();
    let mut keys = BTreeSet::new();
    let mut add = |a: usize, b: usize| {
        if a != b {
            keys.insert(if c.directed {
                (a, b)
            } else {
                (a.min(b), a.max(b))
            });
        }
    };
    for i in 1..n {
        if c.family == "disconnected" && i == n / 2 {
            continue;
        }
        let parent = if c.family == "tree" {
            rng.below(i as u64) as usize
        } else {
            i - 1
        };
        add(parent, i);
    }
    if c.family != "tree" {
        for i in 0..n {
            for j in i + 1..n {
                if c.family == "disconnected" && (i < n / 2) != (j < n / 2) {
                    continue;
                }
                if c.family == "bridge" && i / 16 != j / 16 {
                    continue;
                }
                let density = match c.family.as_str() {
                    "dense" => 50,
                    "clustered" => {
                        if i / 16 == j / 16 {
                            40
                        } else {
                            1
                        }
                    }
                    "sparse" | "bridge" => 3,
                    _ => 10,
                };
                if c.family == "grid_like" {
                    if j == i + side {
                        add(i, j)
                    }
                } else if rng.below(100) < density {
                    add(i, j);
                    if c.directed && rng.below(3) == 0 {
                        add(j, i);
                    }
                }
            }
        }
    }
    let edges = keys
        .into_iter()
        .map(|(from, to)| Edge {
            from,
            to,
            weight: if c.weighted { 1 + rng.below(20) } else { 1 },
        })
        .collect();
    let g = Graph {
        nodes,
        edges,
        directed: c.directed,
        weighted: c.weighted,
        start: 0,
        goal: n - 1,
        config: c.clone(),
    };
    g.validate()?;
    Ok(g)
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    visited: u64,
    edges_examined: u64,
    frontier_peak: usize,
    path_cost: Option<u64>,
    path_length: Option<usize>,
    logical_ops: u64,
    total_weight: u64,
    selected_edges: u64,
    rejected_edges: u64,
    dsu_ops: u64,
    priority_queue_ops: u64,
}
fn path(parent: &[Option<usize>], start: usize, goal: usize) -> Vec<usize> {
    let mut p = vec![goal];
    while *p.last().unwrap() != start {
        let Some(v) = parent[*p.last().unwrap()] else {
            return vec![];
        };
        p.push(v);
        if p.len() > parent.len() {
            return vec![];
        }
    }
    p.reverse();
    p
}
fn find(parent: &mut [usize], mut x: usize, ops: &mut u64) -> usize {
    while parent[x] != x {
        *ops += 1;
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    *ops += 1;
    x
}
pub fn execute(g: &Graph, algorithm: &str) -> Result<DomainLog, String> {
    g.validate()?;
    if !ALGORITHMS.contains(&algorithm) {
        return Err("unknown graph algorithm".into());
    }
    if matches!(algorithm, "prim" | "kruskal") && (g.directed || !g.weighted || !g.connected()) {
        return Err("MST requires a connected undirected weighted graph".into());
    }
    let began = Instant::now();
    let n = g.nodes.len();
    let a = g.adjacency();
    let mut r = Recorder::new(
        ArenaDomain::Graph,
        algorithm,
        serde_json::to_value(g).map_err(|e| e.to_string())?,
    );
    let mut m = Metrics::default();
    let mut visited = vec![false; n];
    let mut discovered = vec![false; n];
    let mut parent = vec![None; n];
    let mut dist = vec![u64::MAX; n];
    dist[g.start] = 0;
    if algorithm == "kruskal" {
        let mut edges = (0..g.edges.len()).collect::<Vec<_>>();
        edges.sort_by_key(|&i| (g.edges[i].weight, g.edges[i].from, g.edges[i].to, i));
        let mut dsu = (0..n).collect::<Vec<_>>();
        let mut rank = vec![0; n];
        r.emit("ComponentStarted", json!({"node":g.start}));
        for i in edges {
            let e = &g.edges[i];
            r.emit("EdgeEvaluated", json!({"edge":i,"from":e.from,"to":e.to}));
            m.edges_examined += 1;
            let x = find(&mut dsu, e.from, &mut m.dsu_ops);
            let y = find(&mut dsu, e.to, &mut m.dsu_ops);
            if x == y {
                m.rejected_edges += 1;
                r.emit("EdgeRejected", json!({"edge":i,"reason":"cycle"}));
            } else {
                if rank[x] < rank[y] {
                    dsu[x] = y
                } else {
                    dsu[y] = x;
                    if rank[x] == rank[y] {
                        rank[x] += 1
                    }
                }
                m.selected_edges += 1;
                m.total_weight += e.weight;
                r.emit("EdgeSelected", json!({"edge":i}));
                r.emit("MstEdgeAdded", json!({"edge":i,"weight":e.weight}));
            }
        }
    } else if algorithm == "prim" {
        let mut heap = BinaryHeap::new();
        visited[g.start] = true;
        m.visited += 1;
        r.emit("ComponentStarted", json!({"node":g.start}));
        r.emit("NodeDiscovered", json!({"node":g.start,"distance":0}));
        r.emit("NodeVisited", json!({"node":g.start}));
        for &(v, i, w) in &a[g.start] {
            heap.push(Reverse((w, i, g.start, v)));
            m.priority_queue_ops += 1;
        }
        m.frontier_peak = heap.len();
        while let Some(Reverse((w, i, u, v))) = heap.pop() {
            m.priority_queue_ops += 1;
            m.edges_examined += 1;
            r.emit("EdgeEvaluated", json!({"edge":i,"from":u,"to":v}));
            if visited[v] {
                m.rejected_edges += 1;
                r.emit("EdgeRejected", json!({"edge":i,"reason":"cycle"}));
                continue;
            }
            visited[v] = true;
            m.visited += 1;
            r.emit("NodeDiscovered", json!({"node":v}));
            r.emit("NodeVisited", json!({"node":v}));
            parent[v] = Some(u);
            r.emit(
                "EdgeSelected",
                json!({"edge":i,"from":u,"to":v,"parent":true}),
            );
            r.emit("MstEdgeAdded", json!({"edge":i,"weight":w}));
            m.selected_edges += 1;
            m.total_weight += w;
            for &(next, edge, cost) in &a[v] {
                if !visited[next] {
                    heap.push(Reverse((cost, edge, v, next)));
                    m.priority_queue_ops += 1;
                }
            }
            m.frontier_peak = m.frontier_peak.max(heap.len());
        }
    } else if matches!(algorithm, "bfs" | "dfs") {
        let roots = std::iter::once(g.start)
            .chain((0..n).filter(|&i| i != g.start))
            .collect::<Vec<_>>();
        for root in roots {
            if discovered[root] {
                continue;
            }
            r.emit("ComponentStarted", json!({"node":root}));
            let mut q = VecDeque::from([root]);
            m.frontier_peak = m.frontier_peak.max(q.len());
            discovered[root] = true;
            r.emit(
                "NodeDiscovered",
                json!({"node":root,"distance":if root==g.start{Some(0)}else{None}}),
            );
            while let Some(u) = if algorithm == "bfs" {
                q.pop_front()
            } else {
                q.pop_back()
            } {
                visited[u] = true;
                m.visited += 1;
                r.emit("NodeVisited", json!({"node":u}));
                let neighbors = if algorithm == "dfs" {
                    a[u].iter().rev().copied().collect::<Vec<_>>()
                } else {
                    a[u].clone()
                };
                for (v, i, w) in neighbors {
                    m.edges_examined += 1;
                    r.emit("EdgeEvaluated", json!({"edge":i,"from":u,"to":v}));
                    if !discovered[v] {
                        discovered[v] = true;
                        parent[v] = Some(u);
                        if dist[u] != u64::MAX {
                            dist[v] = dist[u] + w;
                        }
                        r.emit("NodeDiscovered", json!({"node":v}));
                        r.emit("EdgeRelaxed",json!({"edge":i,"from":u,"to":v,"distance":if dist[v]==u64::MAX{None}else{Some(dist[v])}}));
                        r.emit("EdgeSelected", json!({"edge":i}));
                        q.push_back(v);
                    } else {
                        r.emit(
                            "EdgeRejected",
                            json!({"edge":i,"reason":"already discovered"}),
                        );
                        m.rejected_edges += 1;
                    }
                    m.frontier_peak = m.frontier_peak.max(q.len());
                }
            }
        }
    } else {
        let scale = g
            .edges
            .iter()
            .map(|e| {
                let x = &g.nodes[e.from];
                let y = &g.nodes[e.to];
                e.weight * 1000 / (x.x.abs_diff(y.x) + x.y.abs_diff(y.y)).max(1)
            })
            .min()
            .unwrap_or(0);
        let h = |u: usize| {
            if algorithm == "astar" {
                let p = &g.nodes[u];
                let q = &g.nodes[g.goal];
                (p.x.abs_diff(q.x) + p.y.abs_diff(q.y)) * scale / 1000
            } else {
                0
            }
        };
        let mut heap = BinaryHeap::from([Reverse((h(g.start), 0, g.start))]);
        m.frontier_peak = 1;
        m.priority_queue_ops = 1;
        r.emit("ComponentStarted", json!({"node":g.start}));
        r.emit("NodeDiscovered", json!({"node":g.start,"distance":0}));
        discovered[g.start] = true;
        while let Some(Reverse((_, cost, u))) = heap.pop() {
            m.priority_queue_ops += 1;
            if cost != dist[u] {
                continue;
            }
            visited[u] = true;
            m.visited += 1;
            r.emit("NodeVisited", json!({"node":u}));
            if u == g.goal {
                break;
            }
            for &(v, i, w) in &a[u] {
                m.edges_examined += 1;
                r.emit("EdgeEvaluated", json!({"edge":i,"from":u,"to":v}));
                let candidate = cost + w;
                if candidate < dist[v] {
                    dist[v] = candidate;
                    parent[v] = Some(u);
                    if !discovered[v] {
                        discovered[v] = true;
                        r.emit("NodeDiscovered", json!({"node":v}));
                    }
                    r.emit(
                        "EdgeRelaxed",
                        json!({"edge":i,"from":u,"to":v,"distance":candidate}),
                    );
                    heap.push(Reverse((candidate + h(v), candidate, v)));
                    m.priority_queue_ops += 1;
                } else {
                    m.rejected_edges += 1;
                    r.emit("EdgeRejected", json!({"edge":i,"reason":"no improvement"}));
                }
            }
            m.frontier_peak = m.frontier_peak.max(heap.len());
        }
    }
    if !matches!(algorithm, "prim" | "kruskal") {
        let p = path(&parent, g.start, g.goal);
        if !p.is_empty() {
            for pair in p.windows(2) {
                if let Some(edge) = g.edges.iter().position(|e| {
                    (e.from == pair[0] && e.to == pair[1])
                        || (!g.directed && e.from == pair[1] && e.to == pair[0])
                }) {
                    r.emit("EdgeSelected", json!({"edge":edge,"final":true}));
                }
            }
            m.path_cost = Some(dist[g.goal]);
            m.path_length = Some(p.len() - 1);
            r.emit(
                "GoalReached",
                json!({"node":g.goal,"path":p,"cost":dist[g.goal]}),
            );
        }
    }
    m.logical_ops = m.visited + m.edges_examined + m.dsu_ops + m.priority_queue_ops;
    r.emit(
        "RunCompleted",
        json!({"found":m.path_cost.is_some(),"mst":m.selected_edges==n.saturating_sub(1) as u64}),
    );
    r.log.metrics = serde_json::to_value(m).map_err(|e| e.to_string())?;
    r.log.compute_ns = began.elapsed().as_nanos() as u64;
    r.log.seal();
    Ok(r.log)
}
