use crate::database::{Database, PackageCache};
use crate::error_handling::OmniError;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::cmp::Ordering;
use tracing::{info, warn, error, debug};
use semver::{Version, VersionReq};

/// Enhanced dependency with version constraints and conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub version_req: VersionReq,
    pub box_type: String,
    pub optional: bool,
    pub conflicts: Vec<ConflictSpec>,
    pub provides: Vec<ProvideSpec>,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConflictSpec {
    pub package: String,
    pub version_range: VersionReq,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvideSpec {
    pub virtual_package: String,
    pub version: Version,
}

/// Package with resolved version and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: Version,
    pub box_type: String,
    pub dependencies: Vec<Dependency>,
    pub source_url: Option<String>,
    pub install_order: usize,
    pub size: Option<u64>,
    pub is_virtual: bool,
    pub replaces: Vec<String>,
}

impl PartialEq for ResolvedPackage {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version && self.box_type == other.box_type
    }
}

impl Eq for ResolvedPackage {}

impl PartialOrd for ResolvedPackage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResolvedPackage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by install order, then by name for deterministic ordering
        self.install_order.cmp(&other.install_order)
            .then_with(|| self.name.cmp(&other.name))
    }
}

/// Comprehensive resolution plan with conflict analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ResolutionPlan {
    pub packages: Vec<ResolvedPackage>,
    pub conflicts: Vec<ConflictReport>,
    pub warnings: Vec<String>,
    pub total_size: Option<u64>,
    pub dependency_graph: DependencyGraph,
    pub resolution_strategy: ResolutionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub package1: String,
    pub package2: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub suggested_resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    VersionIncompatible,
    ExplicitConflict,
    FileConflict,
    CircularDependency,
    MissingDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    Conservative,  // Prefer existing versions
    Latest,        // Always prefer latest versions
    Minimal,       // Install minimum required packages
    UserGuided,    // Let user choose conflicts
}

/// Dependency graph for cycle detection and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub package: String,
    pub version: Version,
    pub depth: usize,
    pub is_root: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
    pub version_constraint: VersionReq,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    Required,
    Optional,
    Conflicts,
    Provides,
    Replaces,
}

/// Advanced dependency resolver with SAT solving approach
pub struct AdvancedDependencyResolver {
    db: Database,
    strategy: ResolutionStrategy,
    max_depth: usize,
    package_cache: HashMap<String, Vec<ResolvedPackage>>,
}

impl AdvancedDependencyResolver {
    pub async fn new(strategy: ResolutionStrategy) -> Result<Self> {
        let db = Database::new().await?;
        
        Ok(Self {
            db,
            strategy,
            max_depth: 50, // Prevent infinite recursion
            package_cache: HashMap::new(),
        })
    }
    
    /// Resolve dependencies with advanced conflict detection and resolution
    pub async fn resolve_dependencies(
        &mut self,
        package_names: &[String],
        constraints: HashMap<String, VersionReq>,
    ) -> Result<ResolutionPlan> {
        info!("Starting advanced dependency resolution for {} packages", package_names.len());
        
        // Create initial dependency graph
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        
        // Track resolution state
        let mut resolution_state = ResolutionState::new();
        
        // Add root packages
        for package_name in package_names {
            self.add_root_package(&mut graph, &mut resolution_state, package_name, &constraints).await?;
        }
        
        // Perform iterative resolution with backtracking
        let resolved_packages = self.resolve_with_backtracking(&mut graph, &mut resolution_state).await?;
        
        // Detect cycles
        let cycles = self.detect_cycles(&graph)?;
        if !cycles.is_empty() {
            return Err(OmniError::InstallationFailed {
                package: "multiple".to_string(),
                box_type: "unknown".to_string(),
                reason: format!("Circular dependencies detected: {:?}", cycles),
            }.into());
        }
        
        // Check for conflicts
        let conflicts = self.detect_conflicts(&resolved_packages)?;
        
        // Calculate install order using topological sort
        let ordered_packages = self.topological_sort(&graph, &resolved_packages)?;
        
        // Calculate total download size
        let total_size = ordered_packages.iter()
            .filter_map(|p| p.size)
            .sum::<u64>();
        
        let plan = ResolutionPlan {
            packages: ordered_packages,
            conflicts,
            warnings: resolution_state.warnings,
            total_size: Some(total_size),
            dependency_graph: graph,
            resolution_strategy: self.strategy.clone(),
        };
        
        info!("Dependency resolution completed: {} packages, {} conflicts", 
              plan.packages.len(), plan.conflicts.len());
        
        Ok(plan)
    }
    
    async fn add_root_package(
        &mut self,
        graph: &mut DependencyGraph,
        state: &mut ResolutionState,
        package_name: &str,
        constraints: &HashMap<String, VersionReq>,
    ) -> Result<()> {
        // Get available versions for the package
        let available_versions = self.get_available_versions(package_name).await?;
        if available_versions.is_empty() {
            return Err(OmniError::PackageNotFound {
                package: package_name.to_string(),
            }.into());
        }
        
        // Select best version based on constraints and strategy
        let selected_version = self.select_best_version(
            &available_versions,
            constraints.get(package_name),
        )?;
        
        // Add to graph as root node
        graph.nodes.insert(package_name.to_string(), GraphNode {
            package: package_name.to_string(),
            version: selected_version.version.clone(),
            depth: 0,
            is_root: true,
        });
        
        // Add to resolution state
        state.selected_packages.insert(
            package_name.to_string(),
            selected_version.clone()
        );
        
        Ok(())
    }
    
    async fn resolve_with_backtracking(
        &mut self,
        graph: &mut DependencyGraph,
        state: &mut ResolutionState,
    ) -> Result<Vec<ResolvedPackage>> {
        let mut stack = Vec::new();
        let mut processed = HashSet::new();
        
        // Initialize stack with root packages
        for node in graph.nodes.values() {
            if node.is_root {
                stack.push((node.package.clone(), 0));
            }
        }
        
        while let Some((package_name, depth)) = stack.pop() {
            if processed.contains(&package_name) {
                continue;
            }
            
            if depth > self.max_depth {
                return Err(OmniError::InstallationFailed {
                    package: package_name,
                    box_type: "unknown".to_string(),
                    reason: "Maximum dependency depth exceeded".to_string(),
                }.into());
            }
            
            processed.insert(package_name.clone());
            
            // Get the selected package version
            let package = state.selected_packages.get(&package_name)
                .ok_or_else(|| anyhow!("Package not in resolution state: {}", package_name))?;
            
            // Process dependencies
            for dependency in &package.dependencies {
                if dependency.optional && !state.include_optional {
                    continue;
                }
                
                match self.resolve_dependency(graph, state, dependency, depth + 1).await {
                    Ok(Some(dep_package)) => {
                        // Add edge to graph
                        graph.edges.push(GraphEdge {
                            from: package_name.clone(),
                            to: dependency.name.clone(),
                            dependency_type: if dependency.optional {
                                DependencyType::Optional
                            } else {
                                DependencyType::Required
                            },
                            version_constraint: dependency.version_req.clone(),
                        });
                        
                        // Add to stack for further processing
                        stack.push((dependency.name.clone(), depth + 1));
                    }
                    Ok(None) => {
                        // Dependency already satisfied
                        continue;
                    }
                    Err(e) => {
                        // Try alternatives if available
                        if let Some(resolved) = self.try_alternatives(graph, state, dependency, depth + 1).await? {
                            stack.push((resolved, depth + 1));
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
        }
        
        Ok(state.selected_packages.values().cloned().collect())
    }
    
    async fn resolve_dependency(
        &mut self,
        graph: &mut DependencyGraph,
        state: &mut ResolutionState,
        dependency: &Dependency,
        depth: usize,
    ) -> Result<Option<ResolvedPackage>> {
        // Check if already resolved
        if let Some(existing) = state.selected_packages.get(&dependency.name) {
            // Verify version compatibility
            if dependency.version_req.matches(&existing.version) {
                return Ok(None); // Already satisfied
            } else {
                // Version conflict - try to resolve
                return self.resolve_version_conflict(state, dependency, existing).await;
            }
        }
        
        // Get available versions
        let available_versions = self.get_available_versions(&dependency.name).await?;
        if available_versions.is_empty() {
            return Err(OmniError::PackageNotFound {
                package: dependency.name.clone(),
            }.into());
        }
        
        // Filter versions that satisfy the requirement
        let compatible_versions: Vec<_> = available_versions
            .into_iter()
            .filter(|pkg| dependency.version_req.matches(&pkg.version))
            .collect();
        
        if compatible_versions.is_empty() {
            return Err(OmniError::InstallationFailed {
                package: dependency.name.clone(),
                box_type: dependency.box_type.clone(),
                reason: format!("No version satisfies requirement: {}", dependency.version_req),
            }.into());
        }
        
        // Select best compatible version
        let selected = self.select_best_version(&compatible_versions, Some(&dependency.version_req))?;
        
        // Add to graph and state
        graph.nodes.insert(dependency.name.clone(), GraphNode {
            package: dependency.name.clone(),
            version: selected.version.clone(),
            depth,
            is_root: false,
        });
        
        state.selected_packages.insert(dependency.name.clone(), selected.clone());
        
        Ok(Some(selected))
    }
    
    async fn try_alternatives(
        &mut self,
        graph: &mut DependencyGraph,
        state: &mut ResolutionState,
        dependency: &Dependency,
        depth: usize,
    ) -> Result<Option<String>> {
        for alternative in &dependency.alternatives {
            match self.resolve_dependency(graph, state, &Dependency {
                name: alternative.clone(),
                version_req: dependency.version_req.clone(),
                box_type: dependency.box_type.clone(),
                optional: dependency.optional,
                conflicts: dependency.conflicts.clone(),
                provides: dependency.provides.clone(),
                alternatives: vec![], // Prevent infinite recursion
            }, depth).await {
                Ok(Some(_)) => return Ok(Some(alternative.clone())),
                Ok(None) => return Ok(Some(alternative.clone())),
                Err(_) => continue, // Try next alternative
            }
        }
        
        Ok(None)
    }
    
    async fn resolve_version_conflict(
        &mut self,
        state: &mut ResolutionState,
        dependency: &Dependency,
        existing: &ResolvedPackage,
    ) -> Result<Option<ResolvedPackage>> {
        warn!("Version conflict for {}: need {} but have {}", 
              dependency.name, dependency.version_req, existing.version);
        
        match self.strategy {
            ResolutionStrategy::Conservative => {
                // Keep existing version if possible
                if dependency.optional {
                    state.warnings.push(format!(
                        "Skipping optional dependency {} due to version conflict",
                        dependency.name
                    ));
                    return Ok(None);
                } else {
                    return Err(OmniError::InstallationFailed {
                        package: dependency.name.clone(),
                        box_type: dependency.box_type.clone(),
                        reason: format!("Version conflict: need {} but have {}",
                                      dependency.version_req, existing.version),
                    }.into());
                }
            }
            ResolutionStrategy::Latest => {
                // Try to upgrade to a compatible version
                let available = self.get_available_versions(&dependency.name).await?;
                let compatible: Vec<_> = available
                    .into_iter()
                    .filter(|pkg| dependency.version_req.matches(&pkg.version))
                    .filter(|pkg| pkg.version > existing.version)
                    .collect();
                
                if let Some(upgraded) = compatible.into_iter().max_by_key(|pkg| &pkg.version) {
                    state.selected_packages.insert(dependency.name.clone(), upgraded.clone());
                    state.warnings.push(format!(
                        "Upgraded {} from {} to {} to resolve conflict",
                        dependency.name, existing.version, upgraded.version
                    ));
                    return Ok(Some(upgraded));
                }
            }
            _ => {}
        }
        
        Err(OmniError::InstallationFailed {
            package: dependency.name.clone(),
            box_type: dependency.box_type.clone(),
            reason: "Unresolvable version conflict".to_string(),
        }.into())
    }
    
    async fn get_available_versions(&mut self, package_name: &str) -> Result<Vec<ResolvedPackage>> {
        if let Some(cached) = self.package_cache.get(package_name) {
            return Ok(cached.clone());
        }
        
        // This would query the actual package repositories
        // For now, return a mock version
        let mock_package = ResolvedPackage {
            name: package_name.to_string(),
            version: Version::parse("1.0.0").unwrap(),
            box_type: "apt".to_string(),
            dependencies: vec![],
            source_url: None,
            install_order: 0,
            size: Some(1024 * 1024), // 1MB
            is_virtual: false,
            replaces: vec![],
        };
        
        let versions = vec![mock_package];
        self.package_cache.insert(package_name.to_string(), versions.clone());
        
        Ok(versions)
    }
    
    fn select_best_version(
        &self,
        versions: &[ResolvedPackage],
        constraint: Option<&VersionReq>,
    ) -> Result<ResolvedPackage> {
        if versions.is_empty() {
            return Err(anyhow!("No versions available"));
        }
        
        let mut candidates = versions.to_vec();
        
        // Filter by constraint if provided
        if let Some(req) = constraint {
            candidates.retain(|pkg| req.matches(&pkg.version));
        }
        
        if candidates.is_empty() {
            return Err(anyhow!("No versions satisfy constraints"));
        }
        
        // Select based on strategy
        match self.strategy {
            ResolutionStrategy::Latest => {
                candidates.sort_by(|a, b| b.version.cmp(&a.version));
            }
            ResolutionStrategy::Conservative => {
                candidates.sort_by(|a, b| a.version.cmp(&b.version));
            }
            ResolutionStrategy::Minimal => {
                // Prefer versions with fewer dependencies
                candidates.sort_by_key(|pkg| pkg.dependencies.len());
            }
            ResolutionStrategy::UserGuided => {
                // Would prompt user in interactive mode
                candidates.sort_by(|a, b| b.version.cmp(&a.version));
            }
        }
        
        Ok(candidates.into_iter().next().unwrap())
    }
    
    fn detect_cycles(&self, graph: &DependencyGraph) -> Result<Vec<Vec<String>>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();
        
        for node_name in graph.nodes.keys() {
            if !visited.contains(node_name) {
                self.dfs_detect_cycle(
                    graph,
                    node_name,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }
        
        Ok(cycles)
    }
    
    fn dfs_detect_cycle(
        &self,
        graph: &DependencyGraph,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        // Find all outgoing edges
        for edge in &graph.edges {
            if edge.from == node {
                if rec_stack.contains(&edge.to) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|n| n == &edge.to).unwrap();
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                } else if !visited.contains(&edge.to) {
                    self.dfs_detect_cycle(graph, &edge.to, visited, rec_stack, path, cycles);
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
    }
    
    fn detect_conflicts(&self, packages: &[ResolvedPackage]) -> Result<Vec<ConflictReport>> {
        let mut conflicts = Vec::new();
        
        // Check explicit conflicts
        for package in packages {
            for dependency in &package.dependencies {
                for conflict in &dependency.conflicts {
                    if let Some(conflicting_pkg) = packages.iter()
                        .find(|p| p.name == conflict.package && conflict.version_range.matches(&p.version))
                    {
                        conflicts.push(ConflictReport {
                            package1: package.name.clone(),
                            package2: conflicting_pkg.name.clone(),
                            conflict_type: ConflictType::ExplicitConflict,
                            description: conflict.reason.clone(),
                            suggested_resolution: Some(format!(
                                "Remove {} or choose a different version",
                                conflicting_pkg.name
                            )),
                        });
                    }
                }
            }
        }
        
        // Check version incompatibilities
        let mut version_groups: HashMap<String, Vec<&ResolvedPackage>> = HashMap::new();
        for package in packages {
            version_groups.entry(package.name.clone())
                .or_insert_with(Vec::new)
                .push(package);
        }
        
        for (package_name, versions) in version_groups {
            if versions.len() > 1 {
                conflicts.push(ConflictReport {
                    package1: versions[0].name.clone(),
                    package2: versions[1].name.clone(),
                    conflict_type: ConflictType::VersionIncompatible,
                    description: format!("Multiple versions of {} requested", package_name),
                    suggested_resolution: Some("Choose a single compatible version".to_string()),
                });
            }
        }
        
        Ok(conflicts)
    }
    
    fn topological_sort(
        &self,
        graph: &DependencyGraph,
        packages: &[ResolvedPackage],
    ) -> Result<Vec<ResolvedPackage>> {
        let mut in_degree = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize in-degree and adjacency list
        for package in packages {
            in_degree.insert(package.name.clone(), 0);
            adj_list.insert(package.name.clone(), Vec::new());
        }
        
        // Build graph and calculate in-degrees
        for edge in &graph.edges {
            if matches!(edge.dependency_type, DependencyType::Required | DependencyType::Optional) {
                if let Some(deps) = adj_list.get_mut(&edge.from) {
                    deps.push(edge.to.clone());
                }
                if let Some(degree) = in_degree.get_mut(&edge.to) {
                    *degree += 1;
                }
            }
        }
        
        // Kahn's algorithm for topological sorting
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        // Start with nodes that have no incoming edges
        for (package, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(package.clone());
            }
        }
        
        while let Some(package_name) = queue.pop_front() {
            // Find the package object
            if let Some(package) = packages.iter().find(|p| p.name == package_name) {
                result.push(package.clone());
            }
            
            // Reduce in-degree of adjacent nodes
            if let Some(neighbors) = adj_list.get(&package_name) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        // Set install order
        let mut ordered_packages = result;
        for (i, package) in ordered_packages.iter_mut().enumerate() {
            package.install_order = i;
        }
        
        Ok(ordered_packages)
    }
}

/// Internal state tracking during resolution
struct ResolutionState {
    selected_packages: HashMap<String, ResolvedPackage>,
    warnings: Vec<String>,
    include_optional: bool,
}

impl ResolutionState {
    fn new() -> Self {
        Self {
            selected_packages: HashMap::new(),
            warnings: Vec::new(),
            include_optional: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_advanced_resolver_creation() {
        let resolver = AdvancedDependencyResolver::new(ResolutionStrategy::Latest).await;
        assert!(resolver.is_ok());
    }
    
    #[tokio::test]
    async fn test_cycle_detection() {
        let resolver = AdvancedDependencyResolver::new(ResolutionStrategy::Latest).await.unwrap();
        
        // Create a graph with a cycle
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: vec![
                GraphEdge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    dependency_type: DependencyType::Required,
                    version_constraint: VersionReq::parse("*").unwrap(),
                },
                GraphEdge {
                    from: "B".to_string(),
                    to: "C".to_string(),
                    dependency_type: DependencyType::Required,
                    version_constraint: VersionReq::parse("*").unwrap(),
                },
                GraphEdge {
                    from: "C".to_string(),
                    to: "A".to_string(),
                    dependency_type: DependencyType::Required,
                    version_constraint: VersionReq::parse("*").unwrap(),
                },
            ],
        };
        
        // Add nodes
        for name in &["A", "B", "C"] {
            graph.nodes.insert(name.to_string(), GraphNode {
                package: name.to_string(),
                version: Version::parse("1.0.0").unwrap(),
                depth: 0,
                is_root: false,
            });
        }
        
        let cycles = resolver.detect_cycles(&graph).unwrap();
        assert!(!cycles.is_empty());
    }
    
    #[test]
    fn test_version_requirement_parsing() {
        let req = VersionReq::parse(">=1.0.0, <2.0.0").unwrap();
        let version1 = Version::parse("1.5.0").unwrap();
        let version2 = Version::parse("2.1.0").unwrap();
        
        assert!(req.matches(&version1));
        assert!(!req.matches(&version2));
    }
}