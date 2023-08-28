use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display};

use crate::models::{deserialise_from_file, save_to_file, AltGraph, DosageEvent};

use petgraph::algo::toposort;
use petgraph::dot::{Config, Dot};
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{EdgeRef, Topo, Walker};
use petgraph::Directed;

pub enum LayerOrderingMethod {
    // Order nodes in a layer by the average position of their neighbors in the previous layer.
    Barycenter,
    // Order nodes based on the median position of their neighbors.
    Median,
}

type LayerId = usize;

#[derive(Debug, Default)]
pub struct SankeyLayers<N: Clone + Display, E: Clone> {
    graph: petgraph::Graph<N, E>,
    pub layer_ids: HashMap<NodeIndex, LayerId>,
}
impl<N: Clone + Display, E: Clone> SankeyLayers<N, E> {
    //The layer assignment step can be likened to creating a topological sort of the nodes in the graph,
    //but with additional constraints to create distinct layers. Each layer can be thought of as a set of
    //nodes that don't have any directed edges between them.
    pub fn new(graph: &petgraph::Graph<N, E>) -> Self {
        let mut layer_ids: HashMap<NodeIndex, LayerId> = HashMap::new();

        // let order = match toposort(graph, None) {
        //     Ok(order) => order,
        //     Err(err) => {
        //         if let Some(weight) = graph.node_weight(err.node_id()) {
        //             println!("Error graph has cycle at node {}", weight)
        //         }
        //         return SankeyLayers { layers };
        //     }
        // };

        let mut topo = Topo::new(&graph);

        while let Some(node) = topo.next(&graph) {
            // Determine the layer of this node. If it has no predecessors, it's in layer 0.
            let layer = graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .map(|neighbor| {
                    // If the neighbor isn't assigned a layer, it defaults to 0. This shouldn't
                    // really happen with a proper topological sort, but just in case...
                    *layer_ids.get(&neighbor).unwrap_or(&0) + 1
                })
                .max()
                .unwrap_or(0);

            layer_ids.insert(node, layer);
        }

        SankeyLayers {
            graph: graph.clone(),
            layer_ids,
        }
    }

    pub fn collect_by_layer(&self) -> BTreeMap<LayerId, Vec<NodeIndex>> {
        let mut layer_map: BTreeMap<LayerId, Vec<NodeIndex>> = BTreeMap::new();

        for (node, layer) in &self.layer_ids {
            layer_map.entry(layer.clone()).or_default().push(*node);
        }

        layer_map
    }

    fn average_position(
        &self,
        graph: &petgraph::Graph<N, E, Directed>,
        node: NodeIndex,
        prev_nodes: &[NodeIndex],
    ) -> f64 {
        let neighbors: Vec<_> = graph.neighbors_directed(node, petgraph::Incoming).collect();
        if neighbors.is_empty() {
            return 0.0;
        }

        let total: f64 = neighbors
            .iter()
            .filter_map(|&neighbor| prev_nodes.iter().position(|&x| x == neighbor))
            .map(|pos| pos as f64)
            .sum();

        total / neighbors.len() as f64
    }

    fn median_position(
        &self,
        graph: &petgraph::Graph<N, E, Directed>,
        node: NodeIndex,
        prev_nodes: &[NodeIndex],
    ) -> f64 {
        let mut positions: Vec<_> = graph
            .neighbors_directed(node, petgraph::Incoming)
            .filter_map(|neighbor| prev_nodes.iter().position(|&x| x == neighbor))
            .collect();

        positions.sort_unstable();

        let len = positions.len();
        if len == 0 {
            return 0.0;
        }
        if len % 2 == 1 {
            positions[len / 2] as f64
        } else {
            (positions[len / 2 - 1] + positions[len / 2]) as f64 / 2.0
        }
    }

    pub fn ordered_layers(
        &mut self,
        method: LayerOrderingMethod,
    ) -> BTreeMap<LayerId, Vec<NodeIndex>> {
        let layers = self.collect_by_layer();

        let keys: Vec<_> = layers.keys().cloned().collect();
        for i in 0..keys.len() {
            let layer_id = keys[i];

            if let Some(node_ids) = layers.clone().get_mut(&layer_id) {
                if i > 0 {
                    let prev_layer_id = keys[i - 1];
                    if let Some(prev_node_ids) = layers.get(&prev_layer_id) {
                        match method {
                            LayerOrderingMethod::Barycenter => {
                                node_ids.sort_by(|&a, &b| {
                                    let a_score =
                                        self.average_position(&self.graph, a, prev_node_ids);
                                    let b_score =
                                        self.average_position(&self.graph, b, prev_node_ids);
                                    a_score
                                        .partial_cmp(&b_score)
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                });
                            }
                            LayerOrderingMethod::Median => {
                                node_ids.sort_by(|&a, &b| {
                                    let a_score =
                                        self.median_position(&self.graph, a, prev_node_ids);
                                    let b_score =
                                        self.median_position(&self.graph, b, prev_node_ids);
                                    a_score
                                        .partial_cmp(&b_score)
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                });
                            }
                        }
                    }
                }
                // No need for else block if we don't have specific logic for the first layer
            }
        }

        layers
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

        let ordered_layers = sankey.ordered_layers(LayerOrderingMethod::Barycenter);

        println!("{:?}", &ordered_layers);

        let ordered_layer_1 = ordered_layers.get(&0).unwrap();
        let ordered_layer_2 = ordered_layers.get(&1).unwrap();
        let ordered_layer_3 = ordered_layers.get(&2).unwrap();

        assert_eq!(*ordered_layer_1, vec![n0]);
        assert_eq!(*ordered_layer_2, vec![n3, n2, n1]);
        assert_eq!(*ordered_layer_3, vec![n4]);
    }
}
