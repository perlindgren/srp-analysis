use indented::indented;
use serde::{Deserialize, Serialize};
use std::fmt;

// common data structures

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    pub id: String,
    pub start: u32,
    pub end: u32,
    pub inner: Vec<Trace>,
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} [{}...{}]", self.id, self.start, self.end)?;

        for i in &self.inner {
            write!(f, "{}", indented(i))?
        }
        Ok(())
    }
}

// Our task set
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Tasks(pub Vec<Task>);

impl fmt::Display for Tasks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in &self.0 {
            write!(f, "{}", t)?;
        }
        writeln!(f)
    }
}

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

impl Tasks {
    pub fn load(path: &PathBuf) -> std::io::Result<Tasks> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        let _size = file.read_to_string(&mut contents)?;
        // Convert the JSON string to Tasks.
        let deserialized: Tasks = serde_json::from_str(&contents).unwrap();
        Ok(deserialized)
    }

    pub fn store(&self, path: &PathBuf) -> std::io::Result<()> {
        // Convert Tasks to a JSON string.
        let serialized = serde_json::to_string(self).unwrap();
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn serde() {
        let tasks = crate::task_sets::task_set1();
        tasks.store(&PathBuf::from("task_sets/task_set1.json")).ok();
        let tasks_loaded = Tasks::load(&PathBuf::from("task_sets/task_set1.json")).unwrap();
        assert_eq!(tasks, tasks_loaded);
    }
}
