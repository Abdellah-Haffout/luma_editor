pub use luma_core as core;
pub use luma_dataflow as dataflow;
pub use luma_domain as domain;
pub use luma_commands as commands;
pub use luma_canvas as canvas;
pub use luma_canvas::focus;
pub use luma_canvas::nodes;
pub use luma_canvas::scene;
pub use luma_ui as ui;

// Direct flattened re-exports for full backward compatibility
pub use dataflow::{
    coerce, BoxFuture, DataflowEngine, DataType as DataflowDataType, EngineCacheStore,
    ExecutionError, LinearColor, NodeCache, NodeDescriptor, NodeExecutionStatus, NodeKernel,
    NodeRegistry, PinDescriptor, PrimitiveValue, ResourceId, SlotId, TaskContext, Value,
};

pub use canvas::{
    calculate_wire_bezier, sample_cubic_bezier, sample_wire_spline_points, CanvasTransform,
    EditorDataflowResource, EditorDispatcherResource, EditorGraphResource, EditorHistoryResource,
    EditorIdGenResource, EditorKeybindingsResource, EditorSelectionResource, EditorTransformResource,
    EditorWiringResource, LumaEditorPlugin, SelectionState, WiringState,
};
pub use commands::*;
pub use core::*;
pub use domain::*;
pub use focus::*;
pub use nodes::*;
pub use scene::*;
pub use ui::{
    editor_root_scene, status_bar_scene, top_navbar_scene, EditorContextualToolsResource,
    EditorNavigationResource, EditorTheme, EditorUiCamera, LumaUiPlugin,
};
