fn generate_graph() -> petgraph::Graph<DosageEvent, i32, Directed> {
    let acyclic_graph: AltGraph = deserialise_from_file::<AltGraph>("data.json").unwrap();

    let mut dag: petgraph::Graph<DosageEvent, i32, Directed> = petgraph::Graph::new();

    let mut node_id_map = HashMap::<String, NodeIndex>::new();

    for node in acyclic_graph.nodes.iter() {
        let idx: NodeIndex = dag.add_node(DosageEvent {
            grade: node.datum.grade,
            dose: node.datum.dose,
        });
        node_id_map.insert(node.id.clone(), idx);
    }

    for edge in acyclic_graph.edges.iter() {
        let target_idx = node_id_map.get(&edge.target);
        let source_idx = node_id_map.get(&edge.source);

        match (target_idx, source_idx) {
            (Some(&target_idx), Some(&source_idx)) => {
                dag.add_edge(source_idx, target_idx, 1);
            }
            _ => {}
        }
    }

    dag
}

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::time::Instant;

use charts::models::{deserialise_from_file, AltGraph, DosageEvent};

use petgraph::algo::toposort;
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};
use petgraph::visit::{EdgeRef, Walker, WalkerIter};
use petgraph::Directed;

enum LayerOrderingMethod {
    // Order nodes in a layer by the average position of their neighbors in the previous layer.
    Barycenter,
    // Order nodes based on the median position of their neighbors.
    Median,
}

#[derive(Debug, Default)]
struct SankeyLayers {
    layers: Vec<Vec<NodeIndex>>,
}
impl SankeyLayers {
    //The layer assignment step can be likened to creating a topological sort of the nodes in the graph,
    //but with additional constraints to create distinct layers. Each layer can be thought of as a set of
    //nodes that don't have any directed edges between them.
    fn new<N: Clone + Display, E: Clone>(graph: &petgraph::Graph<N, E>) -> Self {
        // Create a new layers Vec
        let mut layers: Vec<Vec<NodeIndex>> = Vec::new();

        // Perform topological sort

        match toposort(graph, None) {
            Ok(order) => {
                for node in order {
                    let layer_idx = layers.len(); // naive approach: each node gets its layer

                    if let Some(layer) = layers.get_mut(layer_idx) {
                        layer.push(node);
                    } else {
                        layers.push(vec![node]);
                    }
                }
            }
            Err(err) => {
                graph
                    .node_weight(err.node_id())
                    .map(|weight| println!("Error graph has cycle at node {}", weight));
            }
        }

        // Optionally: merge some layers based on constraints here...

        SankeyLayers { layers }
    }

    fn order_layers<N: Clone + Debug, E: Clone + Debug>(
        &mut self,
        graph: &petgraph::Graph<N, E, Directed>,
        method: LayerOrderingMethod,
    ) {
        for i in 1..self.layers.len() {
            // Starting from the second layer

            let prev_layer = &self.layers[i - 1].clone();

            let current_layer = &mut self.layers[i];

            match method {
                LayerOrderingMethod::Barycenter => {
                    current_layer.sort_by(|&a, &b| {
                        let a_score = Self::average_position(&graph, a, prev_layer);
                        let b_score = Self::average_position(&graph, b, prev_layer);
                        a_score
                            .partial_cmp(&b_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                LayerOrderingMethod::Median => {
                    // Implement similarly using a median_position function
                }
            }
        }
    }

    fn average_position<N: Clone + Debug, E: Clone + Debug>(
        graph: &petgraph::Graph<N, E, Directed>,
        node: NodeIndex,
        prev_layer: &[NodeIndex],
    ) -> f64 {
        let neighbors: Vec<_> = graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
            .collect();
        let sum_positions: usize = neighbors
            .iter()
            .filter_map(|&n| prev_layer.iter().position(|&m| m == n))
            .sum();
        sum_positions as f64 / neighbors.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Graph;

    #[test]
    fn test_order_layers_barycenter() {
        // Create a simple graph.
        //       0
        //     / | \
        //    1  2  3
        //       |
        //       4
        let mut graph = Graph::<&str, (), Directed>::new();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");
        let n4 = graph.add_node("4");

        graph.add_edge(n0, n1, ());
        graph.add_edge(n0, n2, ());
        graph.add_edge(n0, n3, ());
        graph.add_edge(n2, n4, ());

        let mut sankey = SankeyLayers::new(&graph);

        println!("{:?}", &sankey);

        sankey.order_layers(&graph, LayerOrderingMethod::Barycenter);

        println!("{:?}", &sankey);

        assert_eq!(sankey.layers[1], vec![n1, n2, n3]); // Expected order
    }

    #[test]
    fn test_order_layers_sankey() {
        let mut graph = generate_graph();

        let mut sankey = SankeyLayers::new(&graph);

        println!("{:?}", &sankey);

        sankey.order_layers(&graph, LayerOrderingMethod::Barycenter);

        println!("{:?}", &sankey);
    }
}

fn main() {}
