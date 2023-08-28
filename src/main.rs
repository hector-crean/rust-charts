use charts::{
    create_crs_graph, errors,
    sankey::{LayerOrderingMethod, SankeyLayers},
};

fn main() -> errors::Result<()> {
    let graph = create_crs_graph()?;

    let mut sankey_layers = SankeyLayers::new(&graph);

    println!("{:?}", &sankey_layers);

    let ordered_layers = sankey_layers.ordered_layers(LayerOrderingMethod::Median);

    println!("{:?}", &ordered_layers);

    // sankey.order_layers(&graph, LayerOrderingMethod::Barycenter);

    Ok(())
}
