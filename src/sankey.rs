use std::collections::HashMap;

use svg::{
    node::{
        self,
        element::{path, Element, Group, Path, Rectangle, Style, Text, SVG},
    },
    Node,
};

pub struct SankeyStyle<F: Fn(f64) -> String> {
    pub number_format: Option<F>,
    pub node_separation: Option<f64>,
    pub node_width: Option<f64>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_color: Option<String>,
    pub border: Option<f64>,
}

impl<F: Fn(f64) -> String> Default for SankeyStyle<F> {
    fn default() -> Self {
        SankeyStyle {
            number_format: None,
            node_separation: None,
            node_width: None,
            font_family: None,
            font_size: None,
            font_color: None,
            border: None,
        }
    }
}

pub struct Sankey {
    nodes: HashMap<SankeyNodeID, SankeyNode>,
    edges: Vec<SankeyEdge>,
}

impl Sankey {
    pub fn new() -> Sankey {
        Sankey {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn node(
        &mut self,
        node_id: &SankeyNodeID,
        value: Option<f64>,
        label: Option<String>,
        color: Option<String>,
    ) -> Option<SankeyNode> {
        self.nodes.insert(
            node_id.clone(),
            SankeyNode::new(node_id.clone(), value, label, color),
        )
    }

    pub fn edge(
        &mut self,
        source: SankeyNodeID,
        target: SankeyNodeID,
        value: f64,
        label: Option<String>,
        color: Option<String>,
    ) {
        self.edges.push(SankeyEdge {
            source: source.clone(),
            target: target.clone(),
            value,
            label,
            color,
        });

        match self.nodes.get_mut(&source.clone()) {
            Some(source) => {
                source.current_output += value;
            }
            _ => {}
        }
        match self.nodes.get_mut(&target.clone()) {
            Some(target) => {
                target.current_output += value;
            }
            _ => {}
        }
    }

    pub fn value(&self, node: SankeyNodeID) -> Option<f64> {
        self.nodes
            .get(&node)
            .and_then(|node: &SankeyNode| node.value)
    }

    pub fn current_input(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().current_input
    }

    pub fn current_output(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().current_output
    }

    pub fn required_input(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().required_input()
    }

    pub fn required_output(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().required_output()
    }

    pub fn remaining_input(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().remaining_input()
    }

    pub fn remaining_output(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().remaining_output()
    }

    pub fn flow(&self, node: SankeyNodeID) -> f64 {
        self.nodes.get(&node).unwrap().flow()
    }

    pub fn draw<F: Fn(f64) -> String>(
        &self,
        width: f64,
        height: f64,
        style: SankeyStyle<F>,
    ) -> SVG {
        let node_separation = style.node_separation.unwrap_or(height / 30.0);
        let node_width = style.node_width.unwrap_or(width / 100.0);
        let font_family: String = style.font_family.unwrap_or("sans-serif".to_string());
        let font_size: f64 = style.font_size.unwrap_or(height / 50.0);
        let font_color: String = style.font_color.unwrap_or("#000".to_string());
        let border: f64 = style.border.unwrap_or(height / 10.0);

        // Initialise SVG

        let mut document = SVG::new();

        document.assign("viewBox", (0.0, 0.0, width, height));

        document.append(Style::new(format!(
            "rect.node {{
	fill: #000F;
}}

.edge > path {{
	fill: #0004;
}}

text.node, .edge > text {{
	fill: {font_color};
	text-anchor: middle;
	dominant-baseline: central;
	font-family: {font_family};
	font-size: {font_size}px;
}}

.edge:not(:hover) > text {{
	display: none;
}}"
        )));

        // Pre-process graph

        #[derive(Copy, Clone, Debug)]
        struct SankeyEdgeID(usize);

        #[derive(Clone, Debug)]
        struct Dependencies {
            inputs: Vec<SankeyEdgeID>,
            outputs: Vec<SankeyEdgeID>,
        }

        let mut dependency_counts = HashMap::<SankeyNodeID, i32>::new();

        let mut node_edges = HashMap::<SankeyNodeID, Dependencies>::new();

        for (
            id,
            &SankeyEdge {
                ref source,
                ref target,
                ..
            },
        ) in self.edges.iter().enumerate()
        {
            node_edges
                .entry(source.clone())
                .and_modify(|v| v.outputs.push(SankeyEdgeID(id)));

            node_edges
                .entry(target.clone())
                .and_modify(|v| v.inputs.push(SankeyEdgeID(id)));

            dependency_counts
                .entry(target.clone())
                .and_modify(|v| *v += 1);
        }
        let node_edges = node_edges;

        // Split into layers

        let mut layers = Vec::new();
        let mut next_layer = Vec::new();

        for (id, &count) in dependency_counts.iter() {
            if count == 0 {
                next_layer.push(id);
            }
        }

        let mut min_scale = f64::INFINITY;

        while !next_layer.is_empty() {
            layers.push(next_layer);
            next_layer = Vec::new();
            let current_layer = layers.last().unwrap();

            let mut total_value = 0.0;

            // for node_id in current_layer {
            //     let node = &self.nodes.get(&node_id).unwrap();
            //     total_value += node.flow();
            //     for edge in &node_edges[node_id.0].outputs {
            //         let target = self.edges[edge.0].target;
            //         dependency_counts[target.0] -= 1;
            //         if dependency_counts[target.0] == 0 {
            //             next_layer.push(target);
            //         }
            //     }
            // }

            let scale =
                (height - border * 2.0 - node_separation * ((current_layer.len() - 1) as f64))
                    / total_value;
            if scale < min_scale {
                min_scale = scale;
            }
        }

        // Generate nodes

        let mut svg_nodes = Vec::new();
        let mut svg_node_labels = Vec::new();

        let mut positions = HashMap::<SankeyNodeID, (f64, f64, f64)>::new();

        let layer_width = (width - border * 2.0 - (layers.len() as f64) * node_width)
            / ((layers.len() - 1) as f64);

        let mut x = border;

        for layer in layers {
            let mut total_height = -node_separation;
            for node_id in &layer {
                total_height +=
                    self.nodes.get(&node_id).unwrap().flow() * min_scale + node_separation;
            }
            let total_height = total_height;
            let mut y = (height - total_height) / 2.0;
            for node_id in layer {
                let node = self.nodes.get(&node_id).unwrap();
                positions.insert(node_id.clone(), (x, y, y));

                let mut rect = Rectangle::new();
                rect.assign("x", x);
                rect.assign("y", y);
                rect.assign("width", node_width);
                rect.assign("height", node.flow() * min_scale);
                rect.assign("class", "node");
                if let Some(color) = node.color.as_deref() {
                    rect.assign("style", format!("fill:{color}"));
                }
                svg_nodes.push(rect);

                let mid_x = x + node_width / 2.0;
                let mid_y = y + node.flow() * min_scale / 2.0;

                let mut text = Text::new();
                text.assign("x", mid_x);
                text.assign("y", mid_y);
                text.assign("class", "node");
                let number = style
                    .number_format
                    .as_ref()
                    .map_or(node.flow().to_string(), |f| f(node.flow()));
                if let Some(label) = &node.label {
                    let mut top = Element::new("tspan");
                    top.assign("x", mid_x);
                    top.assign("dy", -font_size / 2.0);
                    top.append(node::Text::new(label));
                    text.append(top);
                    let mut bottom = Element::new("tspan");
                    bottom.assign("x", mid_x);
                    bottom.assign("dy", font_size);
                    bottom.append(node::Text::new(number));
                    text.append(bottom);
                } else {
                    text.append(node::Text::new(number));
                }
                svg_node_labels.push(text);

                y += node.flow() * min_scale + node_separation;
            }
            x += node_width + layer_width;
        }

        // Generate edges

        let mut svg_edges = Vec::new();

        for edge in &self.edges {
            let thickness = edge.value * min_scale;

            let from_x = positions.get(&edge.source).unwrap().0 + node_width;
            let from_y_start = positions.get(&edge.source).unwrap().2;
            let from_y_end = from_y_start + thickness;
            let to_x = positions.get(&edge.target).unwrap().0;
            let to_y_start = positions.get(&edge.target).unwrap().1;
            let to_y_end = to_y_start + thickness;
            let mid_x = (from_x + to_x) / 2.0;
            let mid_y = (from_y_start + to_y_end) / 2.0;

            positions
                .entry(edge.source.clone())
                .and_modify(|v| v.2 = from_y_end);

            positions
                .entry(edge.target.clone())
                .and_modify(|v| v.1 = to_y_end);

            let mut group = Group::new();
            group.assign("class", "edge");

            let mut path = Path::new();
            path.assign(
                "d",
                path::Data::new()
                    .move_to((from_x, from_y_start))
                    .cubic_curve_to((mid_x, from_y_start, mid_x, to_y_start, to_x, to_y_start))
                    .line_to((to_x, to_y_end))
                    .cubic_curve_to((mid_x, to_y_end, mid_x, from_y_end, from_x, from_y_end))
                    .close(),
            );
            if let Some(color) = edge.color.as_deref() {
                path.assign("style", format!("fill:{color}"));
            }
            group.append(path);

            let mut text = Text::new();
            text.assign("x", mid_x);
            text.assign("y", mid_y);
            let number = style
                .number_format
                .as_ref()
                .map_or(edge.value.to_string(), |f| f(edge.value));
            if let Some(label) = &edge.label {
                let mut top = Element::new("tspan");
                top.assign("x", 0);
                top.assign("dy", -font_size);
                top.append(node::Text::new(label));
                text.append(top);
                let mut bottom = Element::new("tspan");
                bottom.assign("x", 0);
                bottom.assign("dy", font_size);
                bottom.append(node::Text::new(number));
                text.append(bottom);
            } else {
                text.append(node::Text::new(number));
            }
            group.append(text);

            svg_edges.push((edge.value, group));
        }

        // Add to SVG

        svg_edges.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        for node in svg_nodes {
            document.append(node);
        }

        for (_, edge) in svg_edges {
            document.append(edge);
        }

        for label in svg_node_labels {
            document.append(label);
        }

        document
    }
}

pub struct SankeyNode {
    id: SankeyNodeID,
    value: Option<f64>,
    label: Option<String>,
    color: Option<String>,
    current_input: f64,
    current_output: f64,
}

impl SankeyNode {
    pub fn new(
        node_id: SankeyNodeID,
        value: Option<f64>,
        label: Option<String>,
        color: Option<String>,
    ) -> SankeyNode {
        SankeyNode {
            id: node_id,
            value,
            label,
            color,
            current_input: 0.0,
            current_output: 0.0,
        }
    }

    pub fn required_input(&self) -> f64 {
        self.value.unwrap_or(self.current_output)
    }

    pub fn required_output(&self) -> f64 {
        self.value.unwrap_or(self.current_input)
    }

    pub fn remaining_input(&self) -> f64 {
        self.required_input() - self.current_input
    }

    pub fn remaining_output(&self) -> f64 {
        self.required_output() - self.current_output
    }

    pub fn flow(&self) -> f64 {
        self.value
            .unwrap_or(f64::max(self.current_input, self.current_output))
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct SankeyNodeID(pub String);

pub struct SankeyEdge {
    source: SankeyNodeID,
    target: SankeyNodeID,
    value: f64,
    label: Option<String>,
    color: Option<String>,
}
