// SRP based analysis of task set

use crate::common::*;
use std::collections::HashMap;

// A map from Task/Resource identifiers to priority
pub type IdPrio = HashMap<String, u8>;

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

impl Trace {
    pub fn wcet(&self) -> u32 {
        self.end - self.start
    }

    pub fn blocking(&self, t: &Task, ip: &IdPrio) -> u32 {
        println!("blocking trace\n{}", self);
        if let Some(p) = ip.get(&self.id) {
            println!("block on {} at prio {}", self.id, p);
            if *p >= t.prio {
                println!("-- blocking -- {}", self.wcet());
                return self.wcet();
            }
        }

        self.inner.iter().fold(0, |blocking, trace| {
            // println!("trace {}", trace);
            blocking.max(trace.blocking(t, ip))
        })
    }
}

impl Task {
    // The wcet of self
    pub fn wcet(&self) -> u32 {
        self.trace.end - self.trace.start
    }

    // The blocking of self to a task t
    pub fn blocking(&self, t: &Task, ip: &IdPrio) -> u32 {
        println!("check blocking of {} by {}", t.id, self.id);
        self.trace.blocking(t, ip)
    }
}

impl Tasks {
    // Derives the above maps from a set of tasks
    // pub fn pre_analysis(&self) -> (IdPrio, TaskResources) {

    pub fn pre_analysis(&self) -> IdPrio {
        let mut ip = HashMap::new();

        // let mut tr: TaskResources = HashMap::new();

        for t in &self.0 {
            update_prio(t.prio, &t.trace, &mut ip);
            // for i in &t.trace.inner {
            //     update_tr(t.id.clone(), i, &mut tr);
            // }
        }
        // (ip, tr)
        ip
    }

    // The set of tasks with lower priority than t
    pub fn lower(&self, t: &Task) -> Tasks {
        Tasks(
            self.0
                .clone()
                .into_iter()
                .filter(|t1| t1.prio < t.prio)
                .collect(),
        )
    }

    // The set of tasks with higher priority than t
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

    // The blocking of lower priority tasks to task t
    pub fn blocking(&self, t: &Task, ip: &IdPrio) -> u32 {
        let lower = self.lower(t);

        let blocking = lower
            .0
            .iter()
            .fold(0, |blocking, t1| blocking.max(t1.blocking(t, ip)));
        println!("max blocking {}\n----\n", blocking);
        blocking
    }

    // The interference of higher priority tasks to task t
    pub fn interference(&self, t: &Task) -> u32 {
        let higher = self.higher(t);
        let busy_period = t.deadline;
        println!(
            "interference to task {} during busy period {}",
            t.id, busy_period
        );
        let interference = higher.0.iter().fold(0, |interference, t1| {
            if t1.prio > t.prio {
                let nr = 1 + busy_period / t1.inter_arrival;
                let pre = nr * t1.wcet();
                println!(
                    "interference by {},  {} = {} (times) * {} (wcet), inter_arrival {}",
                    t1.id,
                    pre,
                    nr,
                    t1.wcet(),
                    t1.inter_arrival
                );
                interference + pre
            } else {
                interference
            }
        });

        println!("total interference {}\n----\n", interference);
        interference
    }

    // response time analysis
    pub fn response_time(&self) {
        let ip = self.pre_analysis();
        println!("ip: {:?}", ip);

        for t in &self.0 {
            println!("analyzing task {}", t.id);
            let blocking = self.blocking(t, &ip);
            let interference = self.interference(t);

            println!("task {}", t.id);
            println!("response time {}", blocking + interference + t.wcet());
            println!("wcet          {}", t.wcet());
            println!("blocking      {}", blocking);
            println!("interference  {}", interference);
            println!("----------------\n");
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
        let tasks = Tasks::load(&PathBuf::from("task_sets/task_set1.json")).unwrap();
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
