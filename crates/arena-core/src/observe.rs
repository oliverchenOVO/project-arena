//! One entry point, domain-specific deterministic arithmetic, no provider dependency.
use crate::{
    domain::{ArenaDomain, DomainLog},
    event::EventKind,
    Algorithm, RunLog,
};
use serde_json::{json, Value};
pub enum RunSet<'a> {
    Pathfinding(&'a [RunLog]),
    Domain(&'a [DomainLog]),
}
pub fn observe(runs: RunSet<'_>) -> Value {
    match runs {
        RunSet::Pathfinding(logs) => {
            let baseline = logs
                .iter()
                .find(|l| l.config.algorithm == Algorithm::Dijkstra);
            let measured=logs.iter().map(|l|{let forward=l.events.iter().filter(|e|matches!(&e.kind,EventKind::DirectionExpanded{side,..}if side=="forward")).count();let backward=l.events.iter().filter(|e|matches!(&e.kind,EventKind::DirectionExpanded{side,..}if side=="backward")).count();json!({"algorithm":l.config.algorithm,"metrics":l.metrics,"forwardExpanded":forward,"backwardExpanded":backward,"meetingEvent":l.events.iter().find(|e|matches!(e.kind,EventKind::MeetingPoint{..})).map(|e|e.sequence),"weight":l.config.weight_percent.map(|w|w as f64/100.0)})}).collect::<Vec<_>>();
            let derived=logs.iter().map(|l|json!({"algorithm":l.config.algorithm,"expansionReduction":baseline.and_then(|b|crate::observation::ratio(l.metrics.expanded,b.metrics.expanded)).map(|v|1.0-v),"costIncrease":baseline.and_then(|b|l.metrics.path_cost.zip(b.metrics.path_cost)).and_then(|(a,b)|crate::observation::ratio(a,b)).map(|v|v-1.0)})).collect::<Vec<_>>();
            json!({"version":"observation/2","domain":"pathfinding","measured":measured,"derived":derived,"interpretation":[],"formulas":{"expansionReduction":"1 - expanded / baseline expanded","costIncrease":"cost / baseline cost - 1"}})
        }
        RunSet::Domain(logs) => {
            let Some(first) = logs.first() else {
                return json!({});
            };
            let measured = logs
                .iter()
                .map(|l| json!({"algorithm":l.algorithm_id,"metrics":l.metrics}))
                .collect::<Vec<_>>();
            let features = if first.domain == ArenaDomain::Sorting {
                crate::sorting::features(
                    &serde_json::from_value::<Vec<i64>>(first.problem["values"].clone())
                        .unwrap_or_default(),
                )
            } else {
                let n = first.problem["nodes"].as_array().map_or(0, Vec::len);
                let e = first.problem["edges"].as_array().map_or(0, Vec::len);
                json!({"nodeCount":n,"edgeCount":e,"density":if n<2{0.0}else{e as f64/(n*(n-1)) as f64*if first.problem["directed"]==true{1.0}else{2.0}},"densityFormula":"edges / possible non-self edges","layout":"seeded integer grid jitter / v1"})
            };
            let mut interpretation = vec![];
            for l in logs {
                let key = if first.domain == ArenaDomain::Sorting {
                    "comparisons"
                } else {
                    "edgesExamined"
                };
                if let Some(count) = l.metrics[key].as_u64() {
                    interpretation.push(json!({"ruleVersion":"observation/2","messageKey":"Recorded {algorithm}: {count} {operation}.","values":{"algorithm":l.algorithm_id,"count":count,"operation":key}}));
                }
            }
            for pair in logs.windows(2) {
                let a = &pair[0];
                let b = &pair[1];
                if first.domain == ArenaDomain::Sorting {
                    interpretation.push(json!({"ruleVersion":"observation/2","messageKey":"{a} / {b}: writes {aw} / {bw}; auxiliary elements {am} / {bm}; maximum recursion depth {ad} / {bd}. These are separate tradeoffs.","values":{"a":a.algorithm_id,"b":b.algorithm_id,"aw":a.metrics["writes"],"bw":b.metrics["writes"],"am":a.metrics["auxiliaryMemory"],"bm":b.metrics["auxiliaryMemory"],"ad":a.metrics["maxDepth"],"bd":b.metrics["maxDepth"]}}));
                } else if ["prim", "kruskal"].contains(&a.algorithm_id.as_str())
                    && ["prim", "kruskal"].contains(&b.algorithm_id.as_str())
                {
                    interpretation.push(json!({"ruleVersion":"observation/2","messageKey":"{a} / {b}: forest weight {aw} / {bw}; examined edges {ae} / {be}. Equal weight does not require identical selected edges.","values":{"a":a.algorithm_id,"b":b.algorithm_id,"aw":a.metrics["totalWeight"],"bw":b.metrics["totalWeight"],"ae":a.metrics["edgesExamined"],"be":b.metrics["edgesExamined"]}}));
                }
            }
            json!({"version":"observation/2","domain":first.domain,"measured":measured,"derived":features,"interpretation":interpretation,"formulas":{"disorderScore":"strict inversions / (n*(n-1)/2)","memoryProxy":"maximum live auxiliary array elements; recursion tracked separately; not RAM bytes"}})
        }
    }
}
