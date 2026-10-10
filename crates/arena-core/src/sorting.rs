//! Instrumented algorithms: every array read/write and comparison is recorded.
use crate::{
    domain::{ArenaDomain, DomainLog, Recorder},
    grid::SeededRng,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

pub const ALGORITHMS: [&str; 6] = ["bubble", "insertion", "selection", "merge", "quick", "heap"];
pub const FAMILIES: [&str; 7] = [
    "random",
    "nearly_sorted",
    "reversed",
    "few_unique",
    "duplicates",
    "sawtooth",
    "shuffled_blocks",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SortConfig {
    pub length: usize,
    pub min: i64,
    pub max: i64,
    pub distribution: String,
    pub seed: u64,
}
impl Default for SortConfig {
    fn default() -> Self {
        Self {
            length: 64,
            min: 1,
            max: 100,
            distribution: "random".into(),
            seed: 123,
        }
    }
}
pub fn generate(c: &SortConfig) -> Result<Vec<i64>, String> {
    if c.length > 256
        || c.min > c.max
        || c.min < -1_000_000
        || c.max > 1_000_000
        || !FAMILIES.contains(&c.distribution.as_str())
    {
        return Err("invalid sorting configuration".into());
    }
    let mut rng = SeededRng::new(c.seed);
    let mut a = (0..c.length)
        .map(|_| c.min + rng.below((c.max - c.min + 1) as u64) as i64)
        .collect::<Vec<_>>();
    match c.distribution.as_str() {
        "nearly_sorted" => {
            a.sort();
            for _ in 0..c.length / 20 {
                let x = rng.below(c.length as u64) as usize;
                let y = rng.below(c.length as u64) as usize;
                a.swap(x, y);
            }
        }
        "reversed" => {
            if (c.max - c.min + 1) as usize >= c.length {
                for (i, v) in a.iter_mut().enumerate() {
                    *v = c.max - i as i64;
                }
            } else {
                a.sort();
                a.reverse();
            }
        }
        "few_unique" => {
            for v in &mut a {
                *v = c.min + ((*v - c.min) % 4).min(c.max - c.min);
            }
        }
        "duplicates" => {
            for v in &mut a {
                if rng.below(3) != 0 {
                    *v = c.min;
                }
            }
        }
        "sawtooth" => {
            for (i, v) in a.iter_mut().enumerate() {
                *v = c.min + (i as i64 % 8).min(c.max - c.min)
            }
        }
        "shuffled_blocks" => {
            a.sort();
            let blocks = c.length.div_ceil(8);
            for i in (1..blocks).rev() {
                let j = rng.below((i + 1) as u64) as usize;
                for k in 0..8 {
                    if i * 8 + k < c.length && j * 8 + k < c.length {
                        a.swap(i * 8 + k, j * 8 + k)
                    }
                }
            }
        }
        _ => {}
    }
    Ok(a)
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SortMetrics {
    pub comparisons: u64,
    pub swaps: u64,
    pub writes: u64,
    pub reads: u64,
    pub recursive_calls: u64,
    pub max_depth: usize,
    pub auxiliary_memory: usize,
    pub logical_ops: u64,
    pub sorted: bool,
    pub stable: bool,
    pub in_place: bool,
}
struct Sorter {
    a: Vec<(i64, usize)>,
    r: Recorder,
    m: SortMetrics,
}
impl Sorter {
    fn read(&mut self, i: usize) -> (i64, usize) {
        self.m.reads += 1;
        self.r.emit(
            "Read",
            json!({"index":i,"value":self.a[i].0,"identity":self.a[i].1}),
        );
        self.a[i]
    }
    fn compare(&mut self, i: usize, j: usize) -> bool {
        let x = self.read(i);
        let y = self.read(j);
        self.cmp(i, j, x.0, y.0)
    }
    fn auxiliary_read(&mut self, i: usize, v: (i64, usize)) -> (i64, usize) {
        self.m.reads += 1;
        self.r.emit(
            "Read",
            json!({"index":i,"value":v.0,"identity":v.1,"auxiliary":true}),
        );
        v
    }
    fn cmp(&mut self, i: usize, j: usize, x: i64, y: i64) -> bool {
        self.m.comparisons += 1;
        self.r.emit(
            "Compare",
            json!({"left":i,"right":j,"leftValue":x,"rightValue":y}),
        );
        x > y
    }
    fn write(&mut self, i: usize, v: (i64, usize), source: usize, aux: bool) {
        self.a[i] = v;
        self.m.writes += 1;
        self.r.emit(
            "Write",
            json!({"index":i,"value":v.0,"identity":v.1,"source":source,"auxiliary":aux}),
        );
    }
    fn swap(&mut self, i: usize, j: usize) {
        if i == j {
            return;
        }
        self.read(i);
        self.read(j);
        self.a.swap(i, j);
        self.m.swaps += 1;
        self.m.writes += 2;
        self.r.emit("Swap", json!({"left":i,"right":j}));
    }
    fn call(&mut self, lo: usize, hi: usize, depth: usize) {
        self.m.recursive_calls += 1;
        self.m.max_depth = self.m.max_depth.max(depth);
        self.r
            .emit("RangeSelected", json!({"start":lo,"end":hi,"depth":depth}));
    }
    fn merge(&mut self, lo: usize, hi: usize, depth: usize) {
        self.call(lo, hi, depth);
        if hi - lo < 2 {
            return;
        }
        let mid = (lo + hi) / 2;
        self.merge(lo, mid, depth + 1);
        self.merge(mid, hi, depth + 1);
        let mut buf = Vec::with_capacity(hi - lo);
        for i in lo..hi {
            buf.push(self.read(i));
        }
        self.m.auxiliary_memory = self.m.auxiliary_memory.max(buf.len());
        self.r.emit("MergeStarted",json!({"start":lo,"middle":mid,"end":hi,"buffer":buf.iter().map(|x|x.0).collect::<Vec<_>>()}));
        let (mut x, mut y) = (0, mid - lo);
        for k in lo..hi {
            let source = if x >= mid - lo {
                let s = y;
                y += 1;
                s
            } else if y >= hi - lo || {
                let left = self.auxiliary_read(lo + x, buf[x]);
                let right = self.auxiliary_read(lo + y, buf[y]);
                !self.cmp(lo + x, lo + y, left.0, right.0)
            } {
                let s = x;
                x += 1;
                s
            } else {
                let s = y;
                y += 1;
                s
            };
            let value = self.auxiliary_read(lo + source, buf[source]);
            self.write(k, value, lo + source, true);
        }
        self.r.emit("MergeCompleted", json!({"start":lo,"end":hi}));
    }
    fn quick(&mut self, lo: usize, hi: usize, depth: usize) {
        self.call(lo, hi, depth);
        if hi - lo < 2 {
            return;
        }
        let p = hi - 1;
        let pivot = self.read(p);
        self.r.emit(
            "PivotSelected",
            json!({"index":p,"value":pivot.0,"policy":"last-element Lomuto"}),
        );
        let mut j = lo;
        for i in lo..p {
            let value = self.read(i);
            if !self.cmp(i, p, value.0, pivot.0) {
                self.swap(i, j);
                j += 1
            }
        }
        self.swap(j, p);
        self.r
            .emit("PartitionCompleted", json!({"start":lo,"end":hi,"pivot":j}));
        self.quick(lo, j, depth + 1);
        self.quick(j + 1, hi, depth + 1);
    }
    fn heapify(&mut self, n: usize, root: usize) {
        self.r.emit("Heapify", json!({"root":root,"size":n}));
        let mut i = root;
        loop {
            let l = i * 2 + 1;
            if l >= n {
                break;
            }
            let mut largest = l;
            if l + 1 < n && self.compare(l + 1, l) {
                largest = l + 1
            }
            if !self.compare(largest, i) {
                break;
            }
            self.swap(i, largest);
            i = largest;
        }
    }
}
pub fn execute(input: &[i64], algorithm: &str, config: &SortConfig) -> Result<DomainLog, String> {
    if input.len() > 256
        || input.iter().any(|v| v.unsigned_abs() > 1_000_000)
        || !ALGORITHMS.contains(&algorithm)
    {
        return Err("invalid sorting problem or algorithm".into());
    }
    let began = Instant::now();
    let n = input.len();
    let mut s = Sorter {
        a: input.iter().enumerate().map(|(i, &v)| (v, i)).collect(),
        r: Recorder::new(
            ArenaDomain::Sorting,
            algorithm,
            json!({"values":input,"config":config}),
        ),
        m: SortMetrics {
            stable: matches!(algorithm, "bubble" | "insertion" | "merge"),
            in_place: algorithm != "merge",
            ..Default::default()
        },
    };
    match algorithm {
        "bubble" => {
            for end in (1..n).rev() {
                s.r.emit("RangeSelected", json!({"start":0,"end":end+1}));
                let mut changed = false;
                for i in 0..end {
                    if s.compare(i, i + 1) {
                        s.swap(i, i + 1);
                        changed = true
                    }
                }
                if !changed {
                    break;
                }
            }
        }
        "insertion" => {
            for i in 1..n {
                let value = s.read(i);
                let mut j = i;
                s.r.emit("RangeSelected", json!({"start":0,"end":i+1}));
                while j > 0 {
                    let previous = s.read(j - 1);
                    if !s.cmp(j - 1, i, previous.0, value.0) {
                        break;
                    }
                    s.write(j, previous, j - 1, false);
                    j -= 1;
                }
                s.write(j, value, i, false);
            }
        }
        "selection" => {
            for i in 0..n {
                let mut min = i;
                for j in i + 1..n {
                    if s.compare(min, j) {
                        min = j
                    }
                }
                s.swap(i, min)
            }
        }
        "merge" => s.merge(0, n, 1),
        "quick" => s.quick(0, n, 1),
        "heap" => {
            for i in (0..n / 2).rev() {
                s.heapify(n, i)
            }
            for end in (1..n).rev() {
                s.swap(0, end);
                s.heapify(end, 0)
            }
        }
        _ => unreachable!(),
    }
    s.m.sorted = s.a.windows(2).all(|w| w[0].0 <= w[1].0);
    s.m.logical_ops = s.m.reads + s.m.writes + s.m.comparisons;
    s.r.emit("RunCompleted",json!({"sorted":s.m.sorted,"values":s.a.iter().map(|v|v.0).collect::<Vec<_>>(),"identities":s.a.iter().map(|v|v.1).collect::<Vec<_>>()}));
    s.r.log.metrics = serde_json::to_value(s.m).map_err(|e| e.to_string())?;
    s.r.log.compute_ns = began.elapsed().as_nanos() as u64;
    s.r.log.seal();
    Ok(s.r.log)
}
pub fn replay(log: &DomainLog, cursor: usize) -> Result<Vec<i64>, String> {
    let mut a: Vec<i64> =
        serde_json::from_value(log.problem["values"].clone()).map_err(|e| e.to_string())?;
    if cursor > log.events.len() {
        return Err("cursor out of range".into());
    }
    for e in &log.events[..cursor] {
        match e.event_type.as_str() {
            "Swap" => a.swap(
                e.payload["left"].as_u64().ok_or("left")? as usize,
                e.payload["right"].as_u64().ok_or("right")? as usize,
            ),
            "Write" => {
                a[e.payload["index"].as_u64().ok_or("index")? as usize] =
                    e.payload["value"].as_i64().ok_or("value")?
            }
            _ => {}
        }
    }
    Ok(a)
}
pub fn features(a: &[i64]) -> serde_json::Value {
    let n = a.len();
    let mut inversions = 0;
    for i in 0..n {
        for j in i + 1..n {
            inversions += usize::from(a[i] > a[j]);
        }
    }
    let unique = a.iter().collect::<std::collections::BTreeSet<_>>().len();
    let runs = if n == 0 {
        0
    } else {
        1 + a.windows(2).filter(|w| w[0] > w[1]).count()
    };
    json!({"inversionCount":inversions,"duplicateRatio":if n==0{0.0}else{1.0-unique as f64/n as f64},"uniqueRatio":if n==0{0.0}else{unique as f64/n as f64},"monotonicRuns":runs,"disorderScore":if n<2{0.0}else{inversions as f64/(n*(n-1)/2) as f64},"disorderFormula":"strict inversions / (n*(n-1)/2)"})
}
