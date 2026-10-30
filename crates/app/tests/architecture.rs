//! The dependency rule, checked against the real dependency graph:
//! the domain stays free of IO, the application knows no adapter, and
//! adapters never depend on each other.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::process::Command;

use serde_json::Value;

/// Crates that bring IO, an async runtime, HTTP or XML into a crate.
const IO_CRATES: &[&str] = &[
    "tokio",
    "reqwest",
    "hyper",
    "axum",
    "quick-xml",
    "mio",
    "rust-embed",
];

struct Graph {
    names: HashMap<String, String>,
    edges: HashMap<String, Vec<String>>,
}

impl Graph {
    fn load() -> Graph {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let output = Command::new(cargo)
            .args(["metadata", "--format-version", "1"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("cargo metadata runs");
        assert!(output.status.success(), "cargo metadata failed");
        let metadata: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");

        let names = metadata["packages"]
            .as_array()
            .expect("packages")
            .iter()
            .map(|p| {
                (
                    p["id"].as_str().unwrap().to_owned(),
                    p["name"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        // Only normal (non-dev, non-build) dependencies count for the rule.
        let edges = metadata["resolve"]["nodes"]
            .as_array()
            .expect("resolve nodes")
            .iter()
            .map(|node| {
                let deps = node["deps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|dep| {
                        dep["dep_kinds"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|k| k["kind"].is_null())
                    })
                    .map(|dep| dep["pkg"].as_str().unwrap().to_owned())
                    .collect();
                (node["id"].as_str().unwrap().to_owned(), deps)
            })
            .collect();
        Graph { names, edges }
    }

    fn id(&self, name: &str) -> String {
        let Some((id, _)) = self.names.iter().find(|(_, n)| n.as_str() == name) else {
            panic!("crate {name} not in the workspace")
        };
        id.clone()
    }

    /// Names of every crate reachable from `name` through normal dependencies.
    fn reachable(&self, name: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([self.id(name)]);
        while let Some(id) = queue.pop_front() {
            for dep in self.edges.get(&id).into_iter().flatten() {
                if seen.insert(self.names[dep].clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }
        seen
    }
}

fn forbidden(reachable: &BTreeSet<String>, forbidden: &[&str]) -> Vec<String> {
    reachable
        .iter()
        .filter(|name| forbidden.contains(&name.as_str()))
        .cloned()
        .collect()
}

#[test]
fn domain_has_no_io_dependencies() {
    let graph = Graph::load();
    let found = forbidden(&graph.reachable("jukelight-domain"), IO_CRATES);
    assert!(
        found.is_empty(),
        "jukelight-domain reaches IO crates: {found:?}"
    );
}

#[test]
fn application_knows_no_adapter_and_no_io_client() {
    let graph = Graph::load();
    let reachable = graph.reachable("jukelight-application");
    let adapters: Vec<_> = reachable
        .iter()
        .filter(|n| n.starts_with("jukelight-adapter"))
        .collect();
    assert!(
        adapters.is_empty(),
        "jukelight-application depends on adapters: {adapters:?}"
    );
    let found = forbidden(
        &reachable,
        &["reqwest", "hyper", "axum", "quick-xml", "rust-embed"],
    );
    assert!(
        found.is_empty(),
        "jukelight-application reaches IO clients: {found:?}"
    );
}

#[test]
fn adapters_do_not_depend_on_each_other() {
    let graph = Graph::load();
    for adapter in [
        "jukelight-adapter-bluos",
        "jukelight-adapter-fake",
        "jukelight-adapter-artwork",
        "jukelight-adapter-http",
        "jukelight-adapter-music-assistant",
    ] {
        let others: Vec<_> = graph
            .reachable(adapter)
            .into_iter()
            .filter(|n| n.starts_with("jukelight-adapter") && n != adapter)
            .collect();
        assert!(
            others.is_empty(),
            "{adapter} depends on other adapters: {others:?}"
        );
    }
}
