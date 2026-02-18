use crate::graph::BuildGraph;
use crate::cache::HybridCache;
use crate::remote_cache::RemoteCache;
use anyhow::Result;

pub fn execute_graph<R: RemoteCache>(graph: &mut BuildGraph, cache: &mut HybridCache<R>) -> Result<()> {
    let order = graph.topological_order();
    
    for &node_id in &order {
        if node_id >= graph.nodes.len() { continue; }
        
        let node_hash = graph.nodes[node_id].hash.clone();
        
        // 1. Check if we have it in the hybrid cache
        if let Some(_data) = cache.get_artifact(&node_hash)? {
            println!("⚡ Cache HIT: {} [{}]", graph.nodes[node_id].name, &node_hash[..8]);
            graph.nodes[node_id].dirty = false;
            graph.nodes[node_id].cache_hit = true;
            continue;
        }

        // 2. If node is dirty or cache miss, execute
        if graph.nodes[node_id].dirty {
            println!("🔧 Rebuilding node: {}...", graph.nodes[node_id].name);
            
            // Simulation: produce some "artifact" data
            let artifact_data = format!("artifact for {}: {}", graph.nodes[node_id].name, graph.nodes[node_id].content).into_bytes();
            
            // 3. Store the produced artifact in the hybrid cache (local + remote)
            cache.put_artifact(&node_hash, &artifact_data)?;
            
            graph.nodes[node_id].dirty = false;
            graph.nodes[node_id].cache_hit = false;
        } else {
            // This case shouldn't really happen if hashing and propagation are correct
            // but we'll print it for debugging
            println!("⏩ Skipping clean node: {}", graph.nodes[node_id].name);
        }
    }
    
    Ok(())
}
