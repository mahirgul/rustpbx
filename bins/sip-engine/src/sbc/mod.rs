pub mod nat_traversal;
pub mod topology_hiding;

pub use nat_traversal::NatTraversal;
pub use topology_hiding::TopologyHiding;

#[allow(dead_code)]
pub struct SbcPipeline {
    pub topology_hiding: TopologyHiding,
    pub nat_traversal: NatTraversal,
}

impl Default for SbcPipeline {
    fn default() -> Self {
        SbcPipeline {
            topology_hiding: TopologyHiding,
            nat_traversal: NatTraversal,
        }
    }
}
