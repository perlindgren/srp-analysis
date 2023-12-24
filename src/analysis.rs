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

impl Trace {
    pub fn wcet(&self) -> u32 {
        self.end - self.start
    }

    pub fn blocking(&self, t: &Task, ip: &IdPrio) -> u32 {
        if let Some(p) = ip.get(&self.id) {
            if *p > t.prio {
                return self.wcet();
            }
        }

        self.inner
            .iter()
            .fold(0, |blocking, trace| blocking.max(trace.blocking(t, ip)))
    }
}

impl Task {
    // The wcet of a task
    pub fn wcet(&self) -> u32 {
        self.trace.end - self.trace.start
    }

    // The blocking of a task
    pub fn blocking(&self, t: &Task, ip: &IdPrio) -> u32 {
        self.trace.blocking(t, ip)
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

    pub fn lower(&self, t: &Task) -> Tasks {
        Tasks(
            self.0
                .clone()
                .into_iter()
                .filter(|t1| t1.prio < t.prio)
                .collect(),
        )
    }

    pub fn higher(&self, t: &Task) -> Tasks {
        Tasks(
            self.0
                .clone()
                .into_iter()
                .filter(|t1| t1.prio > t.prio)
                .collect(),
        )
    }

    // total utilization
    pub fn total_utilization(&self) -> f32 {
        let mut tot_util = 0.0;

        for t in self.0.iter() {
            let wcet = t.wcet();
            let util = wcet as f32 / t.inter_arrival as f32;
            println!(
                "id {}, start {}, end {}, inter_arrival {}, wcet {}, ratio {}",
                t.id, t.trace.start, t.trace.end, t.inter_arrival, wcet, util
            );
            tot_util += util;
        }
        tot_util
    }

    // The blocking of a task
    pub fn blocking(&self, t: &Task, ip: &IdPrio) -> u32 {
        let lower = self.lower(t);
        println!("lower\n{}", lower);
        todo!();
    }

    // response time analysis
    pub fn response_time(&self) {
        let (ip, tr) = self.pre_analysis();
        println!("ip: {:?}", ip);
        println!("tr: {:?}", tr);
        // A map from Task identifiers to a set of Resource identifiers
        pub type TaskResources = HashMap<String, HashSet<String>>;
        for t in &self.0 {
            println!("task {}", t.id);
            let mut cs = 0;

            for t2 in &self.0 {
                if t2.prio < t.prio {
                    if let Some(resources) = tr.get(&t2.id) {
                        println!("blocked by {} : {:?}", t2.id, resources);
                        for r in resources {
                            if let Some(p) = ip.get(r) {
                                println!("r:id {} ceiling {}", r, p);
                            }
                        }
                    }
                }
            }

            let mut preemption = 0;
            let busy_period = t.deadline;
            for t2 in &self.0 {
                if t2.prio > t.prio {
                    let nr = 1 + busy_period / t2.inter_arrival;
                    let pre = nr * t2.wcet();
                    println!("preempted by {}, nr {}, time {}", t2.id, nr, pre);
                    preemption += pre
                }
            }
        }
    }
}

// pub fn response_time

#[cfg(test)]
mod test {
    use crate::common::*;
    use std::{collections::HashMap, path::PathBuf};

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

    #[test]
    fn response_time_set2() {
        let tasks = Tasks::load(&PathBuf::from("task_sets/task_set2.json")).unwrap();
        let response_time = tasks.response_time();
    }

    #[test]
    fn test_blocking() {
        let trace = Trace {
            id: "R1".to_string(),
            start: 10,
            end: 20,
            inner: vec![
                Trace {
                    id: "R2".to_string(),
                    start: 12,
                    end: 14,
                    inner: vec![],
                },
                Trace {
                    id: "R2".to_string(),
                    start: 14,
                    end: 18,
                    inner: vec![],
                },
            ],
        };

        let t0 = Task::default();
        let t1 = Task {
            prio: 1,
            ..Task::default()
        };

        let mut ip = HashMap::new();
        ip.insert("R1".to_string(), 1);
        ip.insert("R2".to_string(), 2);

        let cs = trace.blocking(&t0, &ip);
        println!("cs {:?}", cs);

        let cs = trace.blocking(&t1, &ip);
        println!("cs {:?}", cs);
    }
}
