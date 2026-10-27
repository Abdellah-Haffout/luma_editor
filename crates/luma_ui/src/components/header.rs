use crate::menu_bar::MenuKind;
use crate::theme::EditorTheme;
use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct TopNavHeader;

#[derive(Component, Clone, Default)]
pub struct ActionButtonSpawn;

#[derive(Component, Clone, Default)]
pub struct ActionButtonSaveBake;

#[derive(Component, Clone, Default)]
pub struct ActionButtonUndo;

#[derive(Component, Clone, Default)]
pub struct ActionButtonRedo;

#[derive(Component, Clone, Default)]
pub struct BreadcrumbLabel;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct LumaMenuTrigger(pub MenuKind);

pub fn top_navbar_scene(
    _font: Handle<Font>,
    _active_breadcrumbs: Vec<String>,
    _is_in_focus: bool,
) -> impl Bundle {
    (
        TopNavHeader,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(34.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            border_radius: BorderRadius::bottom(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    )
}
