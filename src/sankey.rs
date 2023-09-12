use itertools::{Itertools, Permutations};
use petgraph::stable_graph::{EdgeIndex, NodeIndex};
use petgraph::visit::{EdgeRef, Topo, Walker};
use petgraph::Directed;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::iter::Iterator;
use std::ops::Range;
use std::slice::Iter;

struct SankeyDiagram<'layers> {
    layers: &'layers [Vec<NodeIndex>],
}
impl<'layers> SankeyDiagram<'layers> {
    fn permutations(&self) -> Vec<Permutations<Iter<NodeIndex>>> {
        self.layers
            .into_iter()
            .map(|slots| {
                let k = slots.len();
                slots.iter().permutations(k)
            })
            .collect::<Vec<_>>()
    }
}

struct IVec2 {
    x: i32, // essentially the 'depth' or 'layer' of the graph
    y: i32, // we imagine each layer as a verical series of slots. The y refers to the slot index. Each node will take a contiguous range of slot indices
}

fn is_intersecting(p1: &IVec2, q1: &IVec2, p2: &IVec2, q2: &IVec2) -> bool {
    let det = (q1.x - p1.x) * (q2.y - p2.y) - (q1.y - p1.y) * (q2.x - p2.x);
    if det.abs() < 0 {
        return false; // Lines are parallel
    }

    let lambda = ((q2.y - p2.y) * (q2.x - p1.x) + (p2.x - q2.x) * (q2.y - p1.y)) / det;
    let gamma = ((p1.y - q1.y) * (q2.x - p1.x) + (q1.x - p1.x) * (q2.y - p1.y)) / det;

    (0 <= lambda && lambda <= 1) && (0 <= gamma && gamma <= 1)
}

/**
 * A sankey diagram will always be represented as a directed acyclic graph (DAG). In general, sankey diagrams represent aggregates, so
 * we don't care about individual characteristics, but pfizer have requested that they can see the trajectory of individual patients
 * through the sankey diagram. As such we can either:
 *
 * 1. imagein the acyclic graph as a set of 'folded' nodes, where each node contains subnodes. In the folded state, the individual subnodes are aggregated
 *  into one supernode, and their links are aggregated into individual links to other supernodes
 * 2. use multuple links to connected between nodes, where each link has a patient specific id
 *
 *
 * For representation 1, we can initially order 'layers' using the supernodes, and then organise the subnode coordinates
 *
 */

type SlotIndex = i32;

struct NodeSlots {
    incoming: Range<SlotIndex>,
    outgoing: Range<SlotIndex>,
}
type NodeSlotsContraint = HashMap<NodeIndex, NodeSlots>;

// We're not enfocing at this stage that an incoming edge has to be at the same slot position of an outgoing edge of the same type subject_id...

type EdgeSlots = (i32, i32);
struct SankeySolver<N: Clone + Display, E: Clone> {
    pub slot_coordinates: HashMap<EdgeIndex, (IVec2, IVec2)>,
    layer_ids: HashMap<NodeIndex, LayerId>,
    graph: petgraph::Graph<N, E>,
    node_slots_contraint: NodeSlotsContraint,
}

impl<N: Clone + Display, E: Clone> SankeySolver<N, E> {
    fn new(&self, graph: petgraph::Graph<N, E>, node_slots_contraint: NodeSlotsContraint) -> Self {
        let layer_ids = SankeySolver::<N, E>::layer_ids(&graph);

        let node_slots_contraint = HashMap::<NodeIndex, NodeSlots>::new();

        let slot_coordinates = HashMap::<EdgeIndex, (IVec2, IVec2)>::new();

        for edge_ref in self.graph.edge_references() {
            let (source, target) = (edge_ref.source(), edge_ref.target());
        }

        Self {
            slot_coordinates: HashMap::new(),
            layer_ids,
            graph,
            node_slots_contraint,
        }
    }
    fn layer_ids(graph: &petgraph::Graph<N, E>) -> HashMap<NodeIndex, usize> {
        let mut layer_ids: HashMap<NodeIndex, LayerId> = HashMap::new();

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

        layer_ids
    }
    fn permute(&mut self) -> Self {
        todo!()
    }
    fn score(&self) -> f64 {
        let mut H = 0.0;
        for i in self.graph.edge_references() {
            for j in self.graph.edge_references() {
                let edge_i: EdgeIndex = i.id();
                let edge_j: EdgeIndex = j.id();

                let slot_i = self.slot_coordinates.get(&edge_i);
                let slot_j = self.slot_coordinates.get(&edge_j);

                match (slot_i, slot_j) {
                    (Some((p1, q1)), Some((p2, q2))) => match is_intersecting(p1, q1, p2, q2) {
                        true => {
                            H += 1.0;
                        }
                        false => {
                            H -= 1.0;
                        }
                    },
                    _ => {
                        print!(
                            "Could not find slots for {:?}, {:?}, {:?}, {:?}",
                            i.source(),
                            i.target(),
                            j.source(),
                            j.target()
                        )
                    }
                }

                let (source_node_id, target_node_id) = (i.source(), i.target());
            }
        }
        H
    }
}

// type NodeCoordinates = HashMap<NodeIndex, IVec2>;

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
            layer_map.entry(*layer).or_default().push(*node);
        }

        layer_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Graph;

    #[test]
    fn order_layers_barycenter() {
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
    }
    #[test]
    fn order_layers_median() {
        // Using the same graph structure
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
    }
    #[test]
    fn collect_by_layer() {
        let mut graph = Graph::<&str, (), Directed>::new();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");

        graph.add_edge(n0, n1, ());
        graph.add_edge(n0, n2, ());

        let sankey = SankeyLayers::new(&graph);
        let layers = sankey.collect_by_layer();

        assert_eq!(layers.get(&0).unwrap(), &vec![n0]);
        assert_eq!(layers.get(&1).unwrap(), &vec![n1, n2]); // Order might differ; we just want to check membership
    }
    #[test]
    fn cycle_handling() {
        let mut graph = Graph::<&str, (), Directed>::new();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");

        graph.add_edge(n0, n1, ());
        graph.add_edge(n1, n2, ());
        graph.add_edge(n2, n0, ());

        let sankey = SankeyLayers::new(&graph);
        let layers = sankey.collect_by_layer();

        // This test is more for observation. Depending on how your code reacts, you might decide
        // to place assertions here, or even better, handle cycles more gracefully in your main code.
    }
}
