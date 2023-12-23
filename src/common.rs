use indented::indented;
use std::collections::{HashMap, HashSet};
use std::fmt;

// common data structures

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub prio: u8,
    pub deadline: u32,
    pub inter_arrival: u32,
    pub trace: Trace,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "id            {}", self.id)?;
        writeln!(f, "prio          {}", self.prio)?;
        writeln!(f, "deadline      {}", self.deadline)?;
        writeln!(f, "inter_arrival {}", self.inter_arrival)?;
        writeln!(f, "trace:\n{}", self.trace)
    }
}

//#[derive(Debug, Clone)]
#[derive(Debug)]
pub struct Trace {
    pub id: String,
    pub start: u32,
    pub end: u32,
    pub inner: Vec<Trace>,
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "id {} [{}...{}]", self.id, self.start, self.end)?;

        for i in &self.inner {
            write!(f, "{}", indented(i))?
        }
        Ok(())
    }
}

// useful types

// Our task set
#[derive(Debug)]
pub struct Tasks(pub Vec<Task>);

impl fmt::Display for Tasks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in &self.0 {
            write!(f, "{}", t)?;
        }
        writeln!(f)
    }
}

// A map from Task/Resource identifiers to priority
pub type IdPrio = HashMap<String, u8>;

// A map from Task identifiers to a set of Resource identifiers
pub type TaskResources = HashMap<String, HashSet<String>>;

// Derives the above maps from a set of tasks
pub fn pre_analysis(tasks: &Tasks) -> (IdPrio, TaskResources) {
    let mut ip = HashMap::new();
    let mut tr: TaskResources = HashMap::new();
    for t in &tasks.0 {
        update_prio(t.prio, &t.trace, &mut ip);
        for i in &t.trace.inner {
            update_tr(t.id.clone(), i, &mut tr);
        }
    }
    (ip, tr)
}

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
