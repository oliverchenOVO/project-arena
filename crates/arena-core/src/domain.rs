//! Versioned multi-domain envelope. Legacy pathfinding bytes are never rewritten.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArenaDomain {
    Pathfinding,
    Sorting,
    Graph,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainEvent {
    pub schema_version: u32,
    pub domain: ArenaDomain,
    pub run_id: String,
    pub algorithm_id: String,
    pub tick: usize,
    pub sequence: usize,
    pub event_type: String,
    pub payload: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DomainLog {
    pub schema_version: u32,
    pub domain: ArenaDomain,
    pub algorithm_id: String,
    pub algorithm_version: String,
    pub problem: Value,
    pub events: Vec<DomainEvent>,
    pub metrics: Value,
    pub canonical_hash: String,
    pub compute_ns: u64,
}
impl DomainLog {
    pub fn hash(&self) -> String {
        let data = json!({"schemaVersion":2,"domain":self.domain,"algorithmId":self.algorithm_id,
            "algorithmVersion":self.algorithm_version,"problem":self.problem,
            "events":self.events,"metrics":self.metrics});
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&data).expect("serializable log"))
        )
    }
    pub fn seal(&mut self) {
        self.canonical_hash = self.hash();
    }
    pub fn validate_envelope(&self) -> Result<(), String> {
        if self.schema_version != 2
            || self.algorithm_version != "1"
            || self.canonical_hash != self.hash()
        {
            return Err("invalid v2 version or canonical hash".into());
        }
        for (i, e) in self.events.iter().enumerate() {
            if e.schema_version != 2
                || e.sequence != i
                || e.tick != i
                || e.domain != self.domain
                || e.algorithm_id != self.algorithm_id
            {
                return Err("invalid event envelope".into());
            }
        }
        Ok(())
    }
}

pub struct Recorder {
    pub log: DomainLog,
    run_id: String,
}
impl Recorder {
    pub fn new(domain: ArenaDomain, algorithm: &str, problem: Value) -> Self {
        let run_id = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(domain, algorithm, &problem)).expect("problem serialization")
            )
        );
        Self {
            run_id,
            log: DomainLog {
                schema_version: 2,
                domain,
                algorithm_id: algorithm.into(),
                algorithm_version: "1".into(),
                problem,
                events: vec![],
                metrics: json!({}),
                canonical_hash: String::new(),
                compute_ns: 0,
            },
        }
    }
    pub fn emit(&mut self, kind: &str, payload: Value) {
        let sequence = self.log.events.len();
        self.log.events.push(DomainEvent {
            schema_version: 2,
            domain: self.log.domain,
            run_id: self.run_id.clone(),
            algorithm_id: self.log.algorithm_id.clone(),
            tick: sequence,
            sequence,
            event_type: kind.into(),
            payload,
        });
    }
}

/// A v2 wrapper around untouched legacy logs; hash belongs to the inner log.
pub fn wrap_legacy(logs: &[crate::RunLog]) -> Value {
    json!({"format":"arena-run","version":2,"domain":"pathfinding", "problem":{"grid":logs.first().map(|l| &l.grid),"map_config":logs.first().map(|l| &l.map_config)},"algorithms":logs.iter().map(|l| &l.config).collect::<Vec<_>>(),"logs":logs})
}
/// Adapt old events without changing their serialized bytes or their canonical hash.
pub fn legacy_event_envelopes(log: &crate::RunLog) -> Vec<DomainEvent> {
    log.events
        .iter()
        .map(|e| {
            let mut payload = serde_json::to_value(&e.kind).expect("event serialization");
            let event_type = payload
                .as_object_mut()
                .expect("tagged event")
                .remove("type")
                .expect("event type")
                .as_str()
                .expect("event string")
                .to_string();
            DomainEvent {
                schema_version: 2,
                domain: ArenaDomain::Pathfinding,
                run_id: log.run_id.clone(),
                algorithm_id: serde_json::to_value(log.config.algorithm)
                    .expect("algorithm")
                    .as_str()
                    .expect("algorithm name")
                    .into(),
                tick: e.tick as usize,
                sequence: e.sequence,
                event_type,
                payload,
            }
        })
        .collect()
}
