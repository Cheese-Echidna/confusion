//! Viewport display styles share the same exact face-picking pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewStyle {
    #[default]
    Shaded,
    ShadedEdges,
    ShadedHiddenEdges,
    Wireframe,
    VisibleEdges,
}
