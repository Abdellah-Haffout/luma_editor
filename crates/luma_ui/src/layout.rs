use luma_canvas::plugin::{EditorSelectionResource, EditorTransformResource};
use luma_canvas::focus::context::NavigationState;
use luma_canvas::focus::tools::ContextualToolRegistry;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct EditorRootUi;

#[derive(Component, Clone, Default)]
pub struct MiddleWorkspaceArea;

pub fn editor_root_scene(
    _font: Handle<Font>,
    _transform: &EditorTransformResource,
    _selection: &EditorSelectionResource,
    nav_state: &NavigationState,
    _tool_registry: &ContextualToolRegistry,
) -> impl Bundle {
    let _breadcrumbs = nav_state.breadcrumbs();
    let _is_in_focus = nav_state.is_in_focus();

    (
        EditorRootUi,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}
