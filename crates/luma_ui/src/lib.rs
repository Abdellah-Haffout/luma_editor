use bevy::prelude::*;

pub mod bevy_ui_chrome;
pub mod bevy_ui_hud;
pub mod bevy_ui_inspector;
pub mod bevy_ui_settings;
pub mod components;
pub mod connection_material;
pub mod jackdaw_widgets;
pub mod layout;
pub mod menu_bar;
pub mod plugin;
pub mod snarl;
pub mod theme;
pub mod web_image;
pub mod wires;

/// Non-deprecated alias to Bevy's UI focus interaction component.
pub use bevy::ui::DeprecatedInteraction as UiInteraction;

/// Interactive button marker for Bevy 0.20 UI that automatically attaches
/// `bevy::ui_widgets::Button`, `FocusPolicy::Block`, and `DeprecatedInteraction`
/// so all mouse hover and click interactions work out of the box without warnings.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[require(
    bevy::ui_widgets::Button,
    bevy::ui::FocusPolicy::Block,
    bevy::ui::DeprecatedInteraction
)]
pub struct UiButton;

pub use bevy_ui_chrome::*;
pub use bevy_ui_hud::*;
pub use bevy_ui_inspector::*;
pub use bevy_ui_settings::*;
pub use components::{
    focus_toolbar::*, header::*, node_card::*, status_bar::*,
};
pub use connection_material::*;
pub use luma_node_macro::{luma_body_node, luma_node, luma_system, LumaResource};
pub use inventory;
pub use jackdaw_widgets::*;
pub use layout::*;
pub use menu_bar::*;
pub use plugin::*;
pub use snarl::*;
pub use theme::*;
pub use web_image::*;
pub use wires::*;
