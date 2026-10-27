//! Direct integration and re-exports of `jackdaw_widgets`, `jackdaw_feathers`, and `jackdaw_panels`.
//!
//! All widgets, tokens, separators, segmented controls, collapsible sections, and docking panels
//! are sourced directly from the upstream Jackdaw crates without duplicating crate implementations.

use bevy::prelude::*;
use crate::UiButton as Button;
use bevy_luma::style::prelude::Icon;

// Direct crate re-exports
pub use jackdaw_feathers;
pub use jackdaw_panels;
pub use jackdaw_widgets as upstream_widgets;

// Direct re-exports of Jackdaw Feathers & Widgets primitives
pub use bevy::feathers::controls::{
    FeathersButton, FeathersCheckbox, FeathersDisclosureToggle, FeathersRadio, FeathersSlider,
    FeathersToggleSwitch, FeathersToolButton,
};
pub use jackdaw_feathers::button::{
    ButtonClickEvent, ButtonOperatorCall, ButtonProps, ButtonVariant, button, icon_button,
    spawn_inert_checkbox,
};
pub use jackdaw_feathers::options_bar::{spawn_hint, spawn_scrub_chip};
pub use jackdaw_feathers::panel_card::{
    DisclosureSection, PanelCard, PanelCardCollapseState, PanelCardKey, PanelCardProps,
};
pub use jackdaw_feathers::segmented::{
    segment, segment_background, segment_chrome, segment_label, segment_node, segmented_bar,
    segmented_bar_chrome, segmented_bar_node, set_segment_checked,
};
pub use jackdaw_feathers::separator::{
    EditorSeparator, SeparatorDirection, SeparatorProps, separator,
};
pub use jackdaw_feathers::status_bar::{
    StatusBar, StatusBarCenter, StatusBarLeft, StatusBarRight, status_bar,
};
pub use jackdaw_feathers::tab_strip::{TabStrip, TabStripItem, TabStripOrientation, TabStripTab};
pub use jackdaw_panels::{
    ActiveDockWindow, DockArea, DockAreaStyle, DockPlugin, DockSidebarContainer, DockSidebarIcon,
    DockTab, DockTabBar, DockTabContent, DockWindow, DockWindowDescriptor, LayoutState, Panel,
    PanelGroup, PanelHandle, WindowRegistry, WorkspaceDescriptor, WorkspaceRegistry, panel,
    panel_group, panel_handle,
};
pub use jackdaw_widgets::collapsible::{
    CollapsibleBody, CollapsibleHeader, CollapsiblePlugin, CollapsibleSection, ToggleCollapsible,
};
pub use jackdaw_widgets::tree_view::{
    TreeNode, TreeNodeExpandToggle, TreeRoot, TreeRowContent, TreeRowLabel, TreeViewPlugin,
};

/// Re-exports all upstream `jackdaw_feathers::tokens` directly, plus convenience aliases.
pub mod jackdaw_tokens {
    use bevy::prelude::*;
    pub use jackdaw_feathers::tokens::*;

    pub const ACCENT_CYAN: Color = jackdaw_feathers::tokens::GUIDE_LINE;
    pub const TEXT_MUTED: Color = Color::Srgba(jackdaw_feathers::tokens::TEXT_MUTED_COLOR);
    pub const CHIP_TEXT: Color = jackdaw_feathers::tokens::TEXT_PRIMARY;
    pub const DOC_TAB_INACTIVE_BG: Color = jackdaw_feathers::tokens::PANEL_HEADER_BG;
    pub const HEADER_BAR_HEIGHT: f32 = jackdaw_feathers::tokens::WINDOW_TITLE_BAR_HEIGHT;
    pub const BUTTON_DESTRUCTIVE_BG: Color = Color::srgb(0.35, 0.10, 0.12);
    pub const BUTTON_DESTRUCTIVE_HOVER: Color = Color::srgb(0.48, 0.12, 0.15);
    pub const BUTTON_PRIMARY_BG: Color = jackdaw_feathers::tokens::ACCENT_BLUE;
    pub const BUTTON_SECONDARY_BG: Color = Color::srgb(0.18, 0.22, 0.35);
}

/// Spawns a `jackdaw_feathers::separator::separator` directly under `parent`.
pub fn spawn_separator(parent: &mut ChildSpawnerCommands, props: SeparatorProps) -> Entity {
    parent.spawn(separator(props)).id()
}

/// Semantic variant for status chips using `jackdaw_feathers::tokens::CHIP_*` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChipVariant {
    #[default]
    Neutral,
    Success,
    Warning,
    Caution,
}

impl ChipVariant {
    pub fn colors(&self) -> (Color, Color) {
        match self {
            Self::Neutral => (jackdaw_tokens::CHIP_BG, jackdaw_tokens::CHIP_TEXT),
            Self::Success => (
                jackdaw_tokens::CHIP_SUCCESS_BG,
                jackdaw_tokens::CHIP_SUCCESS_TEXT,
            ),
            Self::Warning => (
                jackdaw_tokens::CHIP_WARNING_BG,
                jackdaw_tokens::CHIP_WARNING_TEXT,
            ),
            Self::Caution => (
                jackdaw_tokens::CHIP_CAUTION_BG,
                jackdaw_tokens::CHIP_CAUTION_TEXT,
            ),
        }
    }
}

/// Spawns a status chip using `jackdaw_feathers::tokens::CHIP_*` directly.
pub fn spawn_chip(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    label: impl Into<String>,
    variant: ChipVariant,
) -> Entity {
    let (bg_color, text_color) = variant.colors();
    parent
        .spawn((
            Node {
                padding: UiRect::axes(
                    Val::Px(jackdaw_tokens::SPACING_MD),
                    Val::Px(jackdaw_tokens::SPACING_XS),
                ),
                border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                border: UiRect::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(bg_color),
            children![(
                Text::new(label.into()),
                TextFont {
                    font: font.clone(),
                    font_size: jackdaw_tokens::TEXT_SIZE_SM,
                    ..default()
                },
                TextColor(text_color),
            )],
        ))
        .id()
}

/// Spawns a `jackdaw_widgets::collapsible::CollapsibleSection` card using `jackdaw_feathers::tokens`.
pub fn spawn_panel_card<F: FnOnce(&mut ChildSpawnerCommands)>(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: Option<&FontSource>,
    icon: Option<&'static str>,
    title: &str,
    accent: Color,
    spawn_children: F,
) -> Entity {
    let mut section_cmd = parent.spawn((
        CollapsibleSection { collapsed: false },
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(jackdaw_tokens::SPACING_LG)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(jackdaw_tokens::SPACING_MD),
            border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::COMPONENT_CARD_RADIUS)),
            border: UiRect::ZERO,
            ..default()
        },
        BackgroundColor(jackdaw_tokens::COMPONENT_CARD_BG),
    ));

    section_cmd.with_children(|card| {
        card.spawn((
            CollapsibleHeader,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(jackdaw_tokens::SPACING_MD),
                border: UiRect::ZERO,
                ..default()
            },
        ))
        .with_children(|hdr| {
            if let (Some(ic), Some(ic_font)) = (icon, icon_font) {
                hdr.spawn((
                    Text::new(ic),
                    TextFont {
                        font: ic_font.clone(),
                        font_size: jackdaw_tokens::TEXT_SIZE,
                        ..default()
                    },
                    TextColor(accent),
                ));
            }
            hdr.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: jackdaw_tokens::TEXT_SIZE_SM,
                    ..default()
                },
                TextColor(accent),
            ));
        });

        card.spawn((
            CollapsibleBody,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(jackdaw_tokens::SPACING_MD),
                border: UiRect::ZERO,
                ..default()
            },
        ))
        .with_children(spawn_children);
    });

    section_cmd.id()
}

/// Marker for interactive checkmark inside a checkbox control.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct JackdawCheckmark;

/// Spawns a checkbox control styled with `jackdaw_feathers::tokens`.
pub fn spawn_checkbox<M: Component + Clone>(
    parent: &mut ChildSpawnerCommands,
    icon_font: &FontSource,
    marker: M,
    is_checked: bool,
) -> Entity {
    let bg_color = if is_checked {
        jackdaw_tokens::ACCENT_BLUE
    } else {
        jackdaw_tokens::INPUT_BG
    };
    let check_text = if is_checked { Icon::CHECK } else { "" };

    parent
        .spawn((
            marker.clone(),
            Button,
            Node {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                border: UiRect::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(Color::NONE),
            children![(
                marker,
                JackdawCheckmark,
                Text::new(check_text),
                TextFont {
                    font: icon_font.clone(),
                    font_size: jackdaw_tokens::TEXT_SIZE_SM,
                    ..default()
                },
                TextColor(Color::WHITE),
            )],
        ))
        .id()
}

/// Spawns a toggle switch control styled with `jackdaw_feathers::tokens`.
pub fn spawn_toggle_switch<M: Component + Clone>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    is_active: bool,
) -> Entity {
    let (bg_color, justify) = if is_active {
        (jackdaw_tokens::ACCENT_BLUE, JustifyContent::FlexEnd)
    } else {
        (jackdaw_tokens::INPUT_BG, JustifyContent::FlexStart)
    };

    parent
        .spawn((
            marker.clone(),
            Button,
            Node {
                width: Val::Px(38.0),
                height: Val::Px(20.0),
                padding: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                border: UiRect::ZERO,
                flex_direction: FlexDirection::Row,
                justify_content: justify,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(Color::NONE),
            children![(
                marker,
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    border: UiRect::ZERO,
                    ..default()
                },
                BackgroundColor(jackdaw_tokens::TEXT_PRIMARY),
                BorderColor::all(Color::NONE),
            )],
        ))
        .id()
}
