use luma_canvas::selection::SelectionState;
use luma_canvas::transform::CanvasTransform;
use crate::theme::EditorTheme;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct StatusBarFooter;

#[derive(Component, Clone, Default)]
pub struct StatusCoordsLabel;

#[derive(Component, Clone, Default)]
pub struct StatusSelectionLabel;

#[derive(Component, Clone, Default)]
pub struct StatusSceneLabel;

pub fn status_bar_scene(
    _font: Handle<Font>,
    _transform: &CanvasTransform,
    _selection: &SelectionState,
    _baked_scene_path: &str,
) -> impl Bundle {
    (
        StatusBarFooter,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    )
}
