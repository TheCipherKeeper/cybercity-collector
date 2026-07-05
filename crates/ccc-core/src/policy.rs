use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Policy {
    #[serde(default)]
    pub host_permissions: Vec<HostPermission>,

    #[serde(default = "default_true")]
    pub allow_telemetry: bool,

    #[serde(default)]
    pub allowed_command_kinds: HashSet<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            host_permissions: vec![HostPermission::ReadFile {
                paths: vec!["/var/log".into()],
            }],
            allow_telemetry: true,
            allowed_command_kinds: ["status".into(), "read_file".into()].into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostPermission {
    ReadFile { paths: Vec<String> },
    ExecService { units: Vec<String> },
    WriteFile { paths: Vec<String> },
}

impl Policy {
    pub fn can_read_file(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();
        for perm in &self.host_permissions {
            if let HostPermission::ReadFile { paths } = perm {
                for allowed in paths {
                    if path_str.starts_with(allowed) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn can_exec_service(&self, unit: &str) -> bool {
        for perm in &self.host_permissions {
            if let HostPermission::ExecService { units } = perm {
                if units.iter().any(|u| u == unit) {
                    return true;
                }
            }
        }
        false
    }

    pub fn can_run_command(&self, kind: &str) -> bool {
        self.allowed_command_kinds.contains(kind)
    }
}

fn default_true() -> bool {
    true
}
