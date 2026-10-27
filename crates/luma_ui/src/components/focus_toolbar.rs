use luma_canvas::focus::tools::ContextualTool;
use crate::theme::EditorTheme;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct FocusExitButton;

#[derive(Component, Clone, Default)]
pub struct ContextualToolButton {
    pub tool_id: &'static str,
}

pub fn focus_toolbar_dock_scene(
    _font: Handle<Font>,
    _focused_node_title: &str,
    _focused_node_kind: &str,
    _tools: &[ContextualTool],
    _is_in_focus: bool,
) -> impl Bundle {
    (
        FocusExitButton,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(28.0),
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
