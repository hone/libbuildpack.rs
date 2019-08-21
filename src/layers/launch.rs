use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_derive::Serialize as DeriveSerialize;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(DeriveSerialize, Debug)]
pub struct Launch {
    processes: Processes,
}

impl Launch {
    pub fn new() -> Self {
        Launch {
            processes: Processes::new(),
        }
    }

    pub fn add_process<T: Into<String>, C: Into<String>>(&mut self, r#type: T, command: C) {
        let type_string = r#type.into();
        let command_string = command.into();

        self.processes.insert(
            type_string.clone(),
            Process {
                r#type: type_string,
                command: command_string,
            },
        );
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.processes
            .get(name)
            .map(|process| process.command.as_str())
    }

    pub fn processes_iter(&self) -> impl Iterator<Item = &Process> {
        self.processes.values()
    }
}

#[derive(Debug)]
struct Processes(HashMap<String, Process>);

impl Processes {
    pub fn new() -> Self {
        Processes(HashMap::new())
    }
}

impl Deref for Processes {
    type Target = HashMap<String, Process>;

    fn deref(&self) -> &HashMap<String, Process> {
        &self.0
    }
}

impl DerefMut for Processes {
    fn deref_mut(&mut self) -> &mut HashMap<String, Process> {
        &mut self.0
    }
}

impl Serialize for Processes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let processes = self;
        let mut seq = serializer.serialize_seq(Some(processes.len()))?;

        for (_, process) in processes.iter() {
            seq.serialize_element(&process)?;
        }
        seq.end()
    }
}

#[derive(DeriveSerialize, Debug)]
pub struct Process {
    r#type: String,
    command: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn it_serializes_to_toml() {
        let process = Process {
            r#type: "web".to_string(),
            command: "bin/rails".to_string(),
        };
        let mut processes = HashMap::new();
        processes.insert("web".to_string(), process);

        let launch = Launch {
            processes: Processes(processes),
        };

        let toml_string = toml::to_string(&launch);
        assert!(toml_string.is_ok());
        assert_eq!(
            r#"[[processes]]
type = "web"
command = "bin/rails"
"#,
            toml_string.unwrap()
        );
    }

    #[test]
    fn it_get_process_command() {
        let process = Process {
            r#type: "web".to_string(),
            command: "bin/rails".to_string(),
        };
        let mut processes = HashMap::new();
        processes.insert("web".to_string(), process);

        let launch = Launch {
            processes: Processes(processes),
        };

        assert_eq!(Some("bin/rails"), launch.get("web"));
    }

    #[test]
    fn it_get_returns_none() {
        let launch = Launch::new();

        assert_eq!(None, launch.get("foo"));
    }

    #[test]
    fn it_adds_process() {
        let mut launch = Launch::new();
        launch.add_process("web", "bin/rails");
        launch.add_process("worker", "bin/worker");

        assert_eq!(Some("bin/rails"), launch.get("web"));
        assert_eq!(Some("bin/worker"), launch.get("worker"));
    }

    #[test]
    fn it_adds_process_same_type() {
        let mut launch = Launch::new();
        launch.add_process("web", "bin/rails");
        launch.add_process("web", "bundle exec puma -p $PORT");

        assert_eq!(Some("bundle exec puma -p $PORT"), launch.get("web"));
    }

    #[test]
    fn it_can_iterate_through_processes() {
        let mut launch = Launch::new();
        launch.add_process("web", "bin/rails");

        for process in launch.processes_iter() {
            assert_eq!("web", process.r#type);
            assert_eq!("bin/rails", process.command);
        }
    }
}
