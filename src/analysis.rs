// SRP based analysis of task set

use crate::common::*;
use std::collections::{HashMap, HashSet};

// A map from Task/Resource identifiers to priority
pub type IdPrio = HashMap<String, u8>;

// A map from Task identifiers to a set of Resource identifiers
pub type TaskResources = HashMap<String, HashSet<String>>;

// helper functions
fn update_prio(prio: u8, trace: &Trace, hm: &mut IdPrio) {
    if let Some(old_prio) = hm.get(&trace.id) {
        if prio > *old_prio {
            hm.insert(trace.id.clone(), prio);
        }
    } else {
        hm.insert(trace.id.clone(), prio);
    }
    for cs in &trace.inner {
        update_prio(prio, cs, hm);
    }
}

fn update_tr(s: String, trace: &Trace, trmap: &mut TaskResources) {
    if let Some(seen) = trmap.get_mut(&s) {
        seen.insert(trace.id.clone());
    } else {
        let mut hs = HashSet::new();
        hs.insert(trace.id.clone());
        trmap.insert(s.clone(), hs);
    }
    for trace in &trace.inner {
        update_tr(s.clone(), trace, trmap);
    }
}

impl Tasks {
    // Derives the above maps from a set of tasks
    pub fn pre_analysis(&self) -> (IdPrio, TaskResources) {
        let mut ip = HashMap::new();
        let mut tr: TaskResources = HashMap::new();
        for t in &self.0 {
            update_prio(t.prio, &t.trace, &mut ip);
            for i in &t.trace.inner {
                update_tr(t.id.clone(), i, &mut tr);
            }
        }
        (ip, tr)
    }

    // total utilization
    pub fn total_utilization(&self) -> f32 {
        let mut tot_util = 0.0;

        for t in self.0.iter() {
            let wcet = t.trace.end - t.trace.start;
            let util = wcet as f32 / t.inter_arrival as f32;
            println!(
                "id {}, start {}, end {}, inter_arrival {}, wcet {}, ratio {}",
                t.id, t.trace.start, t.trace.end, t.inter_arrival, wcet, util
            );
            tot_util += util;
        }
        tot_util
    }

    // response time analysis
    pub fn response_time(&self) {
        let (ip, tr) = self.pre_analysis();
        println!("ip: {:?}", ip);
        println!("tr: {:?}", tr);
    }
}

// pub fn response_time

#[cfg(test)]
mod test {
    #[test]
    fn tot_util_task_set1() {
        let tasks = crate::task_sets::task_set1();
        let tot_util = tasks.total_utilization();
        println!("total utilization {}", tot_util);
    }

    #[test]
    fn response_time_set1() {
        let tasks = crate::task_sets::task_set1();
        let response_time = tasks.response_time();
    }
}
