// Plugin dependency resolver
//
// Resolves plugin dependencies, detects cycles, and produces topological load order.

use crate::error::{CliError, Result};
use crate::plugin::manifest::PluginManifest;
use std::collections::{HashMap, HashSet, VecDeque};

/// Plugin dependency resolver
/// 插件依赖解析器
pub struct DependencyResolver;

impl DependencyResolver {
    /// Resolve dependencies and return topologically sorted load order.
    /// Dependencies are loaded first.
    /// 解析依赖并返回拓扑排序后的加载顺序（依赖先加载）
    pub fn resolve(plugins: &HashMap<String, PluginManifest>) -> Result<Vec<String>> {
        // Check for cycles first
        if let Some(cycle) = Self::detect_cycle(plugins) {
            return Err(CliError::Plugin(format!(
                "Circular dependency detected / 检测到循环依赖: {}",
                cycle.join(" -> ")
            )));
        }

        // Kahn's algorithm for topological sort
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize all nodes
        for name in plugins.keys() {
            in_degree.entry(name.as_str()).or_insert(0);
            adj.entry(name.as_str()).or_default();
        }

        // Build adjacency list and in-degree counts
        for (name, manifest) in plugins {
            for dep_name in manifest.dependencies.keys() {
                if plugins.contains_key(dep_name) {
                    adj.entry(dep_name.as_str()).or_default().push(name.as_str());
                    *in_degree.entry(name.as_str()).or_insert(0) += 1;
                }
            }
        }

        // Start with nodes that have no dependencies
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        // Sort queue for deterministic output
        let mut sorted_queue: Vec<&str> = queue.drain(..).collect();
        sorted_queue.sort();
        queue.extend(sorted_queue);

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.to_string());
            if let Some(neighbors) = adj.get(node) {
                let mut next_nodes = Vec::new();
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_nodes.push(neighbor);
                        }
                    }
                }
                // Sort for deterministic output
                next_nodes.sort();
                queue.extend(next_nodes);
            }
        }

        Ok(result)
    }

    /// Detect circular dependencies using DFS.
    /// Returns Some(cycle_path) if a cycle is found, None otherwise.
    /// 检测循环依赖
    pub fn detect_cycle(plugins: &HashMap<String, PluginManifest>) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        let mut names: Vec<&String> = plugins.keys().collect();
        names.sort(); // deterministic order

        for name in names {
            if !visited.contains(name.as_str()) {
                if let Some(cycle) =
                    Self::dfs_cycle(name, plugins, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn dfs_cycle<'a>(
        node: &'a str,
        plugins: &'a HashMap<String, PluginManifest>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
        path: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(manifest) = plugins.get(node) {
            let mut dep_names: Vec<&String> = manifest.dependencies.keys().collect();
            dep_names.sort();

            for dep_name in dep_names {
                if !plugins.contains_key(dep_name.as_str()) {
                    continue; // skip uninstalled deps
                }
                if !visited.contains(dep_name.as_str()) {
                    if let Some(cycle) =
                        Self::dfs_cycle(dep_name, plugins, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep_name.as_str()) {
                    // Found a cycle - extract the cycle path
                    let cycle_start = path.iter().position(|&n| n == dep_name.as_str()).unwrap();
                    let mut cycle: Vec<String> =
                        path[cycle_start..].iter().map(|s| s.to_string()).collect();
                    cycle.push(dep_name.to_string()); // close the cycle
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }

    /// Validate that all dependencies of a plugin are installed and version-compatible.
    /// 验证插件的所有依赖已安装且版本兼容
    pub fn validate_dependencies(
        plugin: &PluginManifest,
        installed: &HashMap<String, PluginManifest>,
    ) -> Result<()> {
        for (dep_name, version_req) in &plugin.dependencies {
            match installed.get(dep_name) {
                None => {
                    return Err(CliError::Plugin(format!(
                        "Plugin '{}' requires dependency '{}' ({}), which is not installed / \
                         插件 '{}' 依赖 '{}' ({})，但该依赖未安装",
                        plugin.name, dep_name, version_req, plugin.name, dep_name, version_req
                    )));
                }
                Some(dep_manifest) => {
                    // Parse version requirement and check compatibility
                    if let Ok(req) = semver::VersionReq::parse(version_req) {
                        if let Ok(ver) = semver::Version::parse(&dep_manifest.version) {
                            if !req.matches(&ver) {
                                return Err(CliError::Plugin(format!(
                                    "Plugin '{}' requires '{}' {}, but installed version is {} / \
                                     插件 '{}' 要求 '{}' {}，但已安装版本为 {}",
                                    plugin.name,
                                    dep_name,
                                    version_req,
                                    dep_manifest.version,
                                    plugin.name,
                                    dep_name,
                                    version_req,
                                    dep_manifest.version
                                )));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::manifest::{PluginCapabilities, PluginPermissions};

    fn make_manifest(name: &str, deps: Vec<(&str, &str)>) -> PluginManifest {
        PluginManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: "test".to_string(),
            min_tool_version: "0.4.0".to_string(),
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: None,
            capabilities: PluginCapabilities::default(),
            dependencies: deps
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            config: HashMap::new(),
            permissions: PluginPermissions::default(),
            priority: 100,
        }
    }

    #[test]
    fn test_no_dependencies() {
        let mut plugins = HashMap::new();
        plugins.insert("a".to_string(), make_manifest("a", vec![]));
        plugins.insert("b".to_string(), make_manifest("b", vec![]));
        let order = DependencyResolver::resolve(&plugins).unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_simple_chain() {
        let mut plugins = HashMap::new();
        plugins.insert("a".to_string(), make_manifest("a", vec![]));
        plugins.insert("b".to_string(), make_manifest("b", vec![("a", ">=0.1.0")]));
        plugins.insert("c".to_string(), make_manifest("c", vec![("b", ">=0.1.0")]));
        let order = DependencyResolver::resolve(&plugins).unwrap();
        let pos_a = order.iter().position(|n| n == "a").unwrap();
        let pos_b = order.iter().position(|n| n == "b").unwrap();
        let pos_c = order.iter().position(|n| n == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_diamond_dependency() {
        let mut plugins = HashMap::new();
        plugins.insert("a".to_string(), make_manifest("a", vec![]));
        plugins.insert("b".to_string(), make_manifest("b", vec![("a", ">=0.1.0")]));
        plugins.insert("c".to_string(), make_manifest("c", vec![("a", ">=0.1.0")]));
        plugins.insert(
            "d".to_string(),
            make_manifest("d", vec![("b", ">=0.1.0"), ("c", ">=0.1.0")]),
        );
        let order = DependencyResolver::resolve(&plugins).unwrap();
        let pos_a = order.iter().position(|n| n == "a").unwrap();
        let pos_b = order.iter().position(|n| n == "b").unwrap();
        let pos_c = order.iter().position(|n| n == "c").unwrap();
        let pos_d = order.iter().position(|n| n == "d").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_direct_cycle() {
        let mut plugins = HashMap::new();
        plugins.insert("a".to_string(), make_manifest("a", vec![("b", ">=0.1.0")]));
        plugins.insert("b".to_string(), make_manifest("b", vec![("a", ">=0.1.0")]));
        assert!(DependencyResolver::detect_cycle(&plugins).is_some());
        assert!(DependencyResolver::resolve(&plugins).is_err());
    }

    #[test]
    fn test_indirect_cycle() {
        let mut plugins = HashMap::new();
        plugins.insert("a".to_string(), make_manifest("a", vec![("b", ">=0.1.0")]));
        plugins.insert("b".to_string(), make_manifest("b", vec![("c", ">=0.1.0")]));
        plugins.insert("c".to_string(), make_manifest("c", vec![("a", ">=0.1.0")]));
        assert!(DependencyResolver::detect_cycle(&plugins).is_some());
    }

    #[test]
    fn test_no_cycle() {
        let mut plugins = HashMap::new();
        plugins.insert("a".to_string(), make_manifest("a", vec![]));
        plugins.insert("b".to_string(), make_manifest("b", vec![("a", ">=0.1.0")]));
        assert!(DependencyResolver::detect_cycle(&plugins).is_none());
    }

    #[test]
    fn test_validate_missing_dependency() {
        let plugin = make_manifest("a", vec![("missing", ">=0.1.0")]);
        let installed = HashMap::new();
        assert!(DependencyResolver::validate_dependencies(&plugin, &installed).is_err());
    }

    #[test]
    fn test_validate_version_mismatch() {
        let plugin = make_manifest("a", vec![("b", ">=1.0.0")]);
        let mut installed = HashMap::new();
        installed.insert("b".to_string(), make_manifest("b", vec![]));
        // b is version 0.1.0, but a requires >=1.0.0
        assert!(DependencyResolver::validate_dependencies(&plugin, &installed).is_err());
    }

    #[test]
    fn test_validate_version_compatible() {
        let plugin = make_manifest("a", vec![("b", ">=0.1.0")]);
        let mut installed = HashMap::new();
        installed.insert("b".to_string(), make_manifest("b", vec![]));
        assert!(DependencyResolver::validate_dependencies(&plugin, &installed).is_ok());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::plugin::manifest::{PluginCapabilities, PluginPermissions};
    use proptest::prelude::*;

    fn make_manifest_simple(name: &str, deps: Vec<String>) -> PluginManifest {
        PluginManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: "test".to_string(),
            min_tool_version: "0.4.0".to_string(),
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: None,
            capabilities: PluginCapabilities::default(),
            dependencies: deps
                .into_iter()
                .map(|d| (d, ">=0.1.0".to_string()))
                .collect(),
            config: HashMap::new(),
            permissions: PluginPermissions::default(),
            priority: 100,
        }
    }

    /// Generate a random DAG (acyclic) with n nodes.
    /// Nodes are named "p0", "p1", ..., "p{n-1}".
    /// Edges only go from lower-index to higher-index nodes (guarantees acyclicity).
    fn arb_dag(n: usize) -> impl Strategy<Value = HashMap<String, PluginManifest>> {
        // For each pair (i, j) where i < j, randomly decide if there's an edge j -> i (j depends on i)
        let num_edges = if n > 1 { n * (n - 1) / 2 } else { 0 };
        prop::collection::vec(any::<bool>(), num_edges..=num_edges).prop_map(move |edge_flags| {
            let mut plugins = HashMap::new();
            let mut deps: Vec<Vec<String>> = vec![vec![]; n];

            let mut idx = 0;
            for j in 1..n {
                for i in 0..j {
                    if edge_flags[idx] {
                        deps[j].push(format!("p{}", i));
                    }
                    idx += 1;
                }
            }

            for i in 0..n {
                let name = format!("p{}", i);
                plugins.insert(name.clone(), make_manifest_simple(&name, deps[i].clone()));
            }
            plugins
        })
    }

    /// Generate a graph that definitely has a cycle by adding a back-edge.
    fn arb_cyclic_graph() -> impl Strategy<Value = HashMap<String, PluginManifest>> {
        // At least 2 nodes, create a cycle among them
        (2usize..6).prop_flat_map(|n| {
            // Pick a back-edge: from a lower-index node to a higher-index node
            (Just(n), 0..n).prop_map(move |(n, back_from)| {
                let mut plugins = HashMap::new();
                // Create a chain: p0 -> p1 -> ... -> p{n-1}
                for i in 0..n {
                    let deps = if i == 0 {
                        // p0 depends on p{back_target} to create cycle
                        // Actually: create chain where each depends on previous
                        // Then add back-edge from back_from to last node
                        vec![]
                    } else {
                        vec![format!("p{}", i - 1)]
                    };
                    let name = format!("p{}", i);
                    plugins.insert(name.clone(), make_manifest_simple(&name, deps));
                }
                // Add back-edge: p{back_from} depends on p{n-1} (creating cycle)
                let back_node = format!("p{}", back_from);
                if let Some(manifest) = plugins.get_mut(&back_node) {
                    manifest
                        .dependencies
                        .insert(format!("p{}", n - 1), ">=0.1.0".to_string());
                }
                plugins
            })
        })
    }

    proptest! {
        /// Property 11: Circular dependency detection
        /// For any acyclic graph, detect_cycle returns None.
        /// **Validates: Requirements 8.3**
        #[test]
        fn prop_no_cycle_in_dag(plugins in arb_dag(5)) {
            prop_assert!(
                DependencyResolver::detect_cycle(&plugins).is_none(),
                "DAG should have no cycles"
            );
        }

        /// Property 11 (cont): For any graph with a cycle, detect_cycle returns Some.
        /// **Validates: Requirements 8.3**
        #[test]
        fn prop_cycle_detected(plugins in arb_cyclic_graph()) {
            prop_assert!(
                DependencyResolver::detect_cycle(&plugins).is_some(),
                "Cyclic graph should be detected"
            );
        }

        /// Property 12: Dependency topological sort correctness
        /// For any DAG, every plugin appears after all its dependencies.
        /// **Validates: Requirements 8.1, 8.4**
        #[test]
        fn prop_topological_order(plugins in arb_dag(5)) {
            let order = DependencyResolver::resolve(&plugins).unwrap();

            // Build position map
            let positions: HashMap<&str, usize> = order
                .iter()
                .enumerate()
                .map(|(i, name)| (name.as_str(), i))
                .collect();

            // Verify: for each plugin, all its dependencies appear before it
            for (name, manifest) in &plugins {
                if let Some(&my_pos) = positions.get(name.as_str()) {
                    for dep_name in manifest.dependencies.keys() {
                        if let Some(&dep_pos) = positions.get(dep_name.as_str()) {
                            prop_assert!(
                                dep_pos < my_pos,
                                "Dependency '{}' (pos {}) should appear before '{}' (pos {})",
                                dep_name, dep_pos, name, my_pos
                            );
                        }
                    }
                }
            }
        }
    }
}
