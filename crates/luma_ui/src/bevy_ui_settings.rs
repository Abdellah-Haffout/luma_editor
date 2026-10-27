use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{UiButton as Button, UiInteraction as Interaction};
use bevy_luma::style::prelude::Icon;
use luma_canvas::wiring::WireRoutingStyle;
use crate::bevy_ui_chrome::BevyUiStatusBarRoot;
use crate::bevy_ui_hud::{BevyUiMinimapRoot, EditorAccessibilitySettings};
use crate::jackdaw_widgets::{
    jackdaw_tokens, spawn_checkbox, spawn_chip, spawn_panel_card, spawn_separator,
    spawn_toggle_switch, ChipVariant, SeparatorProps,
};
use crate::theme::EditorTheme;

/// Categorical tabs in the settings sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum SettingsCategory {
    #[default]
    General,
    Appearance,
    Graph,
    Render,
    Shortcuts,
    System,
}

impl SettingsCategory {
    pub const ALL: [SettingsCategory; 6] = [
        SettingsCategory::General,
        SettingsCategory::Appearance,
        SettingsCategory::Graph,
        SettingsCategory::Render,
        SettingsCategory::Shortcuts,
        SettingsCategory::System,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Self::General => "General Workspace",
            Self::Appearance => "Appearance & Colors",
            Self::Graph => "Graph & Dataflow",
            Self::Render => "Render & Auto-Bake",
            Self::Shortcuts => "Shortcuts & Input",
            Self::System => "System & Telemetry",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::General => {
                "Configure canvas navigation, display telemetry, auto-save, and scene baking pipeline."
            }
            Self::Appearance => {
                "Customize high-contrast accessibility, dark theme aesthetics, and UI font scale."
            }
            Self::Graph => {
                "Tune reactive dataflow propagation, pin wiring magnet snap, and cycle detection."
            }
            Self::Render => {
                "Manage hardware-accelerated scene export, BSN bytecode serialization, and baking hooks."
            }
            Self::Shortcuts => {
                "View and configure keyboard hotkeys, modifier key combinations, and modal triggers."
            }
            Self::System => {
                "Inspect Bevy engine runtime metrics, SDF UI framework telemetry, and Lucide vector assets."
            }
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::General => Icon::SLIDERS_VERTICAL,
            Self::Appearance => Icon::CONTRAST,
            Self::Graph => Icon::NETWORK,
            Self::Render => Icon::ZAP,
            Self::Shortcuts => Icon::COMMAND,
            Self::System => Icon::ACTIVITY,
        }
    }
}

/// Identifiers for functional, toggleable settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum SettingKey {
    // Toggles (Switch style)
    ShowGrid,
    WirePulses,
    Colorblind,
    AutoBake,
    ShowMinimap,
    ShowTelemetry,
    // Checkboxes (Check style)
    SnapToGrid,
    SnapPins,
    HighDpiFont,
    ReducedMotion,
}

/// Functional background pattern styles for the infinite 2D canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CanvasPatternStyle {
    #[default]
    Dots,
    Grid,
    Crosses,
    None,
}

impl CanvasPatternStyle {
    pub const ALL: [CanvasPatternStyle; 4] = [
        CanvasPatternStyle::Dots,
        CanvasPatternStyle::Grid,
        CanvasPatternStyle::Crosses,
        CanvasPatternStyle::None,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Dots => "Dot Matrix",
            Self::Grid => "Grid Lines",
            Self::Crosses => "Crosshairs (+)",
            Self::None => "Plain Solid",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Dots => Self::Grid,
            Self::Grid => Self::Crosses,
            Self::Crosses => Self::None,
            Self::None => Self::Dots,
        }
    }
}

/// Color theme palettes for the canvas background and grid elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CanvasColorPalette {
    #[default]
    DarkSlate,
    OledBlack,
    Blueprint,
    MatrixGreen,
    WarmAmber,
    PurpleHaze,
}

impl CanvasColorPalette {
    pub const ALL: [CanvasColorPalette; 6] = [
        CanvasColorPalette::DarkSlate,
        CanvasColorPalette::OledBlack,
        CanvasColorPalette::Blueprint,
        CanvasColorPalette::MatrixGreen,
        CanvasColorPalette::WarmAmber,
        CanvasColorPalette::PurpleHaze,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::DarkSlate => "Dark Slate",
            Self::OledBlack => "OLED Charcoal",
            Self::Blueprint => "Blueprint Navy",
            Self::MatrixGreen => "Cyber Emerald",
            Self::WarmAmber => "Warm Bronze",
            Self::PurpleHaze => "Cyber Violet",
        }
    }

    pub fn canvas_bg(&self) -> Color {
        match self {
            Self::DarkSlate => Color::srgb(0.082, 0.086, 0.098),
            Self::OledBlack => Color::srgb(0.035, 0.039, 0.047),
            Self::Blueprint => Color::srgb(0.043, 0.071, 0.125),
            Self::MatrixGreen => Color::srgb(0.035, 0.078, 0.051),
            Self::WarmAmber => Color::srgb(0.094, 0.075, 0.051),
            Self::PurpleHaze => Color::srgb(0.078, 0.051, 0.122),
        }
    }

    pub fn pattern_primary(&self) -> Color {
        match self {
            Self::DarkSlate => Color::srgba(1.0, 1.0, 1.0, 0.038),
            Self::OledBlack => Color::srgba(0.85, 0.90, 0.95, 0.055),
            Self::Blueprint => Color::srgba(0.25, 0.65, 0.95, 0.075),
            Self::MatrixGreen => Color::srgba(0.15, 0.90, 0.45, 0.065),
            Self::WarmAmber => Color::srgba(0.95, 0.68, 0.22, 0.065),
            Self::PurpleHaze => Color::srgba(0.75, 0.45, 0.98, 0.070),
        }
    }

    pub fn pattern_secondary(&self) -> Color {
        match self {
            Self::DarkSlate => Color::srgba(1.0, 1.0, 1.0, 0.016),
            Self::OledBlack => Color::srgba(0.85, 0.90, 0.95, 0.022),
            Self::Blueprint => Color::srgba(0.25, 0.65, 0.95, 0.030),
            Self::MatrixGreen => Color::srgba(0.15, 0.90, 0.45, 0.025),
            Self::WarmAmber => Color::srgba(0.95, 0.68, 0.22, 0.025),
            Self::PurpleHaze => Color::srgba(0.75, 0.45, 0.98, 0.028),
        }
    }

    #[allow(dead_code)]
    pub fn axis_color(&self) -> Color {
        match self {
            Self::DarkSlate => Color::srgba(1.0, 1.0, 1.0, 0.05),
            Self::OledBlack => Color::srgba(0.85, 0.90, 0.95, 0.06),
            Self::Blueprint => Color::srgba(0.25, 0.65, 0.95, 0.08),
            Self::MatrixGreen => Color::srgba(0.15, 0.90, 0.45, 0.07),
            Self::WarmAmber => Color::srgba(0.95, 0.68, 0.22, 0.07),
            Self::PurpleHaze => Color::srgba(0.75, 0.45, 0.98, 0.07),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::DarkSlate => Self::OledBlack,
            Self::OledBlack => Self::Blueprint,
            Self::Blueprint => Self::MatrixGreen,
            Self::MatrixGreen => Self::WarmAmber,
            Self::WarmAmber => Self::PurpleHaze,
            Self::PurpleHaze => Self::DarkSlate,
        }
    }
}

/// Resource holding runtime configuration and open/closed state of the Settings window.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct EditorSettingsState {
    pub is_open: bool,
    pub active_category: SettingsCategory,
    pub show_grid: bool,
    pub canvas_pattern: CanvasPatternStyle,
    pub canvas_palette: CanvasColorPalette,
    pub wire_pulses: bool,
    pub wire_style: WireRoutingStyle,
    pub colorblind_mode: bool,
    pub auto_bake: bool,
    pub show_minimap: bool,
    pub show_telemetry: bool,
    pub snap_to_grid: bool,
    pub snap_pins: bool,
    pub high_dpi_font: bool,
    pub reduced_motion: bool,
}

impl Default for EditorSettingsState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_category: SettingsCategory::General,
            show_grid: true,
            canvas_pattern: CanvasPatternStyle::Dots,
            canvas_palette: CanvasColorPalette::DarkSlate,
            wire_pulses: true,
            wire_style: WireRoutingStyle::Showcase,
            colorblind_mode: false,
            auto_bake: false,
            show_minimap: true,
            show_telemetry: true,
            snap_to_grid: true,
            snap_pins: true,
            high_dpi_font: true,
            reduced_motion: false,
        }
    }
}

impl EditorSettingsState {
    pub fn get(&self, key: SettingKey) -> bool {
        match key {
            SettingKey::ShowGrid => self.show_grid,
            SettingKey::WirePulses => self.wire_pulses,
            SettingKey::Colorblind => self.colorblind_mode,
            SettingKey::AutoBake => self.auto_bake,
            SettingKey::ShowMinimap => self.show_minimap,
            SettingKey::ShowTelemetry => self.show_telemetry,
            SettingKey::SnapToGrid => self.snap_to_grid,
            SettingKey::SnapPins => self.snap_pins,
            SettingKey::HighDpiFont => self.high_dpi_font,
            SettingKey::ReducedMotion => self.reduced_motion,
        }
    }

    pub fn set(&mut self, key: SettingKey, val: bool) {
        match key {
            SettingKey::ShowGrid => self.show_grid = val,
            SettingKey::WirePulses => self.wire_pulses = val,
            SettingKey::Colorblind => self.colorblind_mode = val,
            SettingKey::AutoBake => self.auto_bake = val,
            SettingKey::ShowMinimap => self.show_minimap = val,
            SettingKey::ShowTelemetry => self.show_telemetry = val,
            SettingKey::SnapToGrid => self.snap_to_grid = val,
            SettingKey::SnapPins => self.snap_pins = val,
            SettingKey::HighDpiFont => self.high_dpi_font = val,
            SettingKey::ReducedMotion => self.reduced_motion = val,
        }
    }

    pub fn toggle(&mut self, key: SettingKey) -> bool {
        let new_val = !self.get(key);
        self.set(key, new_val);
        new_val
    }

    pub fn reset_defaults(&mut self) {
        let is_open = self.is_open;
        let active_category = self.active_category;
        *self = Self::default();
        self.is_open = is_open;
        self.active_category = active_category;
    }
}

/// Marker component for the Settings window root container.
#[derive(Component)]
pub struct BevyUiSettingsRoot;

/// Marker component for buttons that close the Settings window.
#[derive(Component)]
pub struct BevyUiSettingsCloseBtn;

/// Marker component for the top chrome button that opens Settings.
#[derive(Component)]
pub struct BevyUiBtnSettings;

/// Marker component for the Reset Defaults button in settings.
#[derive(Component)]
pub struct BevyUiSettingsResetBtn;

/// Marker component for category navigation buttons in the Settings sidebar.
#[derive(Component)]
pub struct BevyUiSettingsCategoryBtn(pub SettingsCategory);

/// Marker component for the content panel corresponding to a category.
#[derive(Component)]
pub struct BevyUiSettingsCategoryPanel(pub SettingsCategory);

/// Marker component for the breadcrumb label in the settings title bar.
#[derive(Component)]
pub struct BevyUiSettingsBreadcrumbText;

/// Marker component for wire routing style buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiWireStyleBtn(pub WireRoutingStyle);

/// Marker component for canvas pattern style buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiCanvasPatternBtn(pub CanvasPatternStyle);

/// Marker component for canvas color palette buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiCanvasPaletteBtn(pub CanvasColorPalette);

/// Marker component for interactive setting toggle switch buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiSettingToggleBtn(pub SettingKey);

/// Marker component for interactive setting checkbox buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiSettingCheckboxBtn(pub SettingKey);

/// Marker component for setting status indicator text (e.g. "ON" / "OFF" or "Active" / "Inactive").
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiSettingIndicatorText(pub SettingKey);

/// Spawns the screen-space modern Settings window (zero borders, luma_style Lucide icons, functional toggles).
pub fn setup_bevy_ui_settings(
    mut commands: Commands,
    camera_q: Query<Entity, With<crate::EditorUiCamera>>,
    asset_server: Res<AssetServer>,
) {
    let cam_opt = camera_q.iter().next();
    let font = FontSource::default();
    let icon_font = FontSource::Handle(
        asset_server.load("embedded://luma_style/style/assets/icons/Lucide.ttf"),
    );

    // Large Modern Settings Window (880x580, zero borders, sleek elevated dark aesthetic)
    let mut window_cmd = commands.spawn((
        BevyUiSettingsRoot,
        GlobalZIndex(280),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-440.0),
                top: Val::Px(-290.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            width: Val::Px(880.0),
            height: Val::Px(580.0),
            flex_direction: FlexDirection::Column,
            border: UiRect::ZERO,
            border_radius: BorderRadius::all(Val::Px(16.0)),
            display: Display::None,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.070, 0.078, 0.098)),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        window_cmd.insert(UiTargetCamera(cam));
    }

    window_cmd.with_children(|window| {
        // ==========================================
        // 1. Top Titlebar (Seamless, zero borders)
        // ==========================================
        window
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(44.0),
                    padding: UiRect::axes(Val::Px(18.0), Val::Px(0.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    border: UiRect::ZERO,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.055, 0.063, 0.080)),
            ))
            .with_children(|header| {
                // Left: Lucide Icon + Title + Breadcrumbs
                header
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        border: UiRect::ZERO,
                        ..default()
                    })
                    .with_children(|left| {
                        // Cyan Lucide Settings Icon badge
                        left.spawn((
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::ZERO,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.16, 0.22)),
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                Text::new(Icon::SETTINGS),
                                TextFont {
                                    font: icon_font.clone(),
                                    font_size: FontSize::Px(14.0),
                                    ..default()
                                },
                                TextColor(EditorTheme::BORDER_FOCUSED),
                            ));
                        });

                        // Window Title
                        left.spawn((
                            Text::new("Preferences & Settings"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(13.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));

                        // Path separator
                        left.spawn((
                            Text::new("›"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.35, 0.38, 0.45)),
                        ));

                        // Category Pill
                        left.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::ZERO,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.13, 0.18)),
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                BevyUiSettingsBreadcrumbText,
                                Text::new("General Workspace"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: FontSize::Px(10.5),
                                    ..default()
                                },
                                TextColor(EditorTheme::BORDER_FOCUSED),
                            ));
                        });
                    });

                // Right: Close Button with Lucide Icon::X
                header
                    .spawn((
                        BevyUiSettingsCloseBtn,
                        Button,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            border: UiRect::ZERO,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(Icon::X),
                            TextFont {
                                font: icon_font.clone(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));
                    });
            });

        // ==========================================
        // 2. Dual-Column Body (Sidebar + Content Pane)
        // ==========================================
        window
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|body| {
                // --- Left Navigation Sidebar (220px) ---
                body.spawn((
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(16.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.060, 0.068, 0.086)),
                ))
                .with_children(|sidebar| {
                    sidebar.spawn((
                        Text::new("CATEGORIES"),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(9.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.40, 0.44, 0.52)),
                    ));

                    // Category navigation buttons with luma_style icons
                    for cat in SettingsCategory::ALL {
                        let is_active = cat == SettingsCategory::General;
                        let bg_color = if is_active {
                            Color::srgb(0.13, 0.15, 0.20)
                        } else {
                            Color::NONE
                        };
                        let text_color = if is_active {
                            EditorTheme::TEXT_PRIMARY
                        } else {
                            EditorTheme::TEXT_MUTED
                        };
                        let icon_color = if is_active {
                            EditorTheme::BORDER_FOCUSED
                        } else {
                            EditorTheme::TEXT_MUTED
                        };

                        sidebar
                            .spawn((
                                BevyUiSettingsCategoryBtn(cat),
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(32.0),
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(8.0),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    border: UiRect::ZERO,
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                            ))
                            .with_children(|item| {
                                item.spawn((
                                    Text::new(cat.icon()),
                                    TextFont {
                                        font: icon_font.clone(),
                                        font_size: FontSize::Px(13.0),
                                        ..default()
                                    },
                                    TextColor(icon_color),
                                ));
                                item.spawn((
                                    Text::new(cat.title()),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: FontSize::Px(11.5),
                                        ..default()
                                    },
                                    TextColor(text_color),
                                ));
                            });
                    }
                });

                // --- Right Main Content Area (Spacious & Modern) ---
                body.spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        border: UiRect::ZERO,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.078, 0.086, 0.106)),
                ))
                .with_children(|content| {
                    // ==========================================
                    // Category Panel 1: General Workspace
                    // ==========================================
                    content
                        .spawn((
                            BevyUiSettingsCategoryPanel(SettingsCategory::General),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                display: Display::Flex,
                                border: UiRect::ZERO,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            spawn_category_header(
                                p,
                                &font,
                                SettingsCategory::General.title(),
                                SettingsCategory::General.description(),
                            );

                            // Card 1: Canvas & Visual Guides (Jackdaw PanelCard)
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::LAYOUT_GRID),
                                "CANVAS & DISPLAY GUIDES",
                                jackdaw_tokens::ACCENT_CYAN,
                                |card| {
                                    // Toggle 1: Infinite 2D Grid
                                    spawn_toggle_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::ShowGrid,
                                        Icon::LAYOUT_GRID,
                                        "Infinite 2D Canvas Grid",
                                        "Dynamic scale-adaptive major (100px) and minor (20px) grid guides",
                                        true,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Toggle 2: Bézier Cable Pulses
                                    spawn_toggle_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::WirePulses,
                                        Icon::ZAP,
                                        "Hardware Bézier Wire Pulses",
                                        "Dynamic animated flow pulses travelling along active dataflow cables",
                                        true,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Checkbox 1: Snap Node Cards To Grid
                                    spawn_checkbox_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::SnapToGrid,
                                        Icon::CROSSHAIR,
                                        "Snap Node Cards To Grid",
                                        "Auto-align node positions to 20px grid increments when moved",
                                        true,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Checkbox 2: Magnetic Pin Socket Snapping
                                    spawn_checkbox_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::SnapPins,
                                        Icon::CHECK,
                                        "Magnetic Pin Socket Snapping",
                                        "Snap dragging cable ends to compatible target pins within 18px",
                                        true,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Wire Style Multi-Shape Selector
                                    spawn_wire_style_selector_row(card, &font, &icon_font, WireRoutingStyle::Showcase);
                                },
                            );

                            // Card 2: Workspace HUD & Telemetry (Jackdaw PanelCard)
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::ACTIVITY),
                                "WORKSPACE HUD OVERLAY",
                                jackdaw_tokens::ACCENT_BLUE,
                                |card| {
                                    // Toggle 3: Canvas Minimap
                                    spawn_toggle_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::ShowMinimap,
                                        Icon::COMPASS,
                                        "Spatial Minimap Overview",
                                        "Interactive bird's-eye canvas overview at the bottom-right corner",
                                        true,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Toggle 4: Bottom Status Bar
                                    spawn_toggle_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::ShowTelemetry,
                                        Icon::ACTIVITY,
                                        "Live Coordinates & Status Bar",
                                        "Fixed 22px footer showing camera pan, zoom level, and selection count",
                                        true,
                                    );
                                },
                            );
                        });

                    // ==========================================
                    // Category Panel 2: Appearance & Colors
                    // ==========================================
                    content
                        .spawn((
                            BevyUiSettingsCategoryPanel(SettingsCategory::Appearance),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                display: Display::None,
                                border: UiRect::ZERO,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            spawn_category_header(
                                p,
                                &font,
                                SettingsCategory::Appearance.title(),
                                SettingsCategory::Appearance.description(),
                            );

                            // Jackdaw PanelCard: Accessibility & Palette
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::CONTRAST),
                                "ACCESSIBILITY & PALETTE",
                                jackdaw_tokens::TEXT_SUCCESS,
                                |card| {
                                    // Toggle: Colorblind Mode
                                    spawn_toggle_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::Colorblind,
                                        Icon::EYE,
                                        "Colorblind & High-Contrast Mode",
                                        "High-contrast node outlines and shape-coded dataflow pin badges",
                                        false,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Checkbox 1: Subpixel SDF Font
                                    spawn_checkbox_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::HighDpiFont,
                                        Icon::TYPE,
                                        "Subpixel Crisp Text Glyphs",
                                        "Vector SDF font rendering cache with sharp edges on high-DPI screens",
                                        true,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Checkbox 2: Reduced Motion
                                    spawn_checkbox_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::ReducedMotion,
                                        Icon::CIRCLE_PAUSE,
                                        "Reduced Motion Animations",
                                        "Disable cable pulsing animations and instant camera snap",
                                        false,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    spawn_info_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        Icon::CONTRAST,
                                        "Borderless Dark SDF Theme",
                                        "100% solid surfaces with zero border artifacts (Pure GPU Signed Distance Field)",
                                        "Solid Matte",
                                        ChipVariant::Neutral,
                                    );
                                },
                            );

                            // Card 2: Canvas Pattern Style & Color Palette
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::LAYOUT_GRID),
                                "CANVAS BACKGROUND PATTERN & COLOR PALETTE",
                                jackdaw_tokens::ACCENT_CYAN,
                                |card| {
                                    // Canvas Pattern Style Selector (Dots, Grid, Crosshairs, None)
                                    spawn_canvas_pattern_selector_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        CanvasPatternStyle::Dots,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    // Canvas Color Palette Selector
                                    spawn_canvas_palette_selector_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        CanvasColorPalette::DarkSlate,
                                    );
                                },
                            );
                        });

                    // ==========================================
                    // Category Panel 3: Graph & Dataflow
                    // ==========================================
                    content
                        .spawn((
                            BevyUiSettingsCategoryPanel(SettingsCategory::Graph),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                display: Display::None,
                                border: UiRect::ZERO,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            spawn_category_header(
                                p,
                                &font,
                                SettingsCategory::Graph.title(),
                                SettingsCategory::Graph.description(),
                            );

                            // Jackdaw PanelCard: Dataflow Propagation
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::NETWORK),
                                "DATAFLOW PROPAGATION",
                                jackdaw_tokens::ACCENT_BLUE,
                                |card| {
                                    spawn_info_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        Icon::NETWORK,
                                        "Demand-Pull Reactive Engine",
                                        "Dirty-checking evaluation with automatic topological execution order",
                                        "Active",
                                        ChipVariant::Success,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    spawn_info_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        Icon::CHECK,
                                        "Pin Magnet Snap & Type Safety",
                                        "Auto-snapping to compatible typed pin targets within 18px radius",
                                        "18px Snap",
                                        ChipVariant::Neutral,
                                    );
                                },
                            );
                        });

                    // ==========================================
                    // Category Panel 4: Render & Auto-Bake
                    // ==========================================
                    content
                        .spawn((
                            BevyUiSettingsCategoryPanel(SettingsCategory::Render),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                display: Display::None,
                                border: UiRect::ZERO,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            spawn_category_header(
                                p,
                                &font,
                                SettingsCategory::Render.title(),
                                SettingsCategory::Render.description(),
                            );

                            // Jackdaw PanelCard: Scene Pipeline
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::PACKAGE),
                                "SCENE PIPELINE & PERSISTENCE",
                                jackdaw_tokens::TEXT_WARNING,
                                |card| {
                                    // Toggle: Auto-Bake
                                    spawn_toggle_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        SettingKey::AutoBake,
                                        Icon::PACKAGE,
                                        "Auto-Bake On Scene Change",
                                        "Automatically compile .luma project to binary .bsn Bevy scene file on edits",
                                        false,
                                    );

                                    spawn_separator(card, SeparatorProps::horizontal());

                                    spawn_info_row(
                                        card,
                                        &font,
                                        &icon_font,
                                        Icon::SAVE,
                                        "Target Export Format",
                                        "scenes/main.luma (Source) -> scenes/main.bsn (Bevy Scene Bytecode)",
                                        "BSN v1.0",
                                        ChipVariant::Neutral,
                                    );
                                },
                            );
                        });

                    // ==========================================
                    // Category Panel 5: Shortcuts & Input
                    // ==========================================
                    content
                        .spawn((
                            BevyUiSettingsCategoryPanel(SettingsCategory::Shortcuts),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                display: Display::None,
                                border: UiRect::ZERO,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            spawn_category_header(
                                p,
                                &font,
                                SettingsCategory::Shortcuts.title(),
                                SettingsCategory::Shortcuts.description(),
                            );

                            // Jackdaw PanelCard: Core Shortcuts
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::COMMAND),
                                "CORE WORKSPACE SHORTCUTS",
                                jackdaw_tokens::ACCENT_CYAN,
                                |card| {
                                    spawn_info_row(card, &font, &icon_font, Icon::SETTINGS, "Preferences & Settings", "Toggle settings modal", "Ctrl + ,", ChipVariant::Neutral);
                                    spawn_separator(card, SeparatorProps::horizontal());
                                    spawn_info_row(card, &font, &icon_font, Icon::SAVE, "Save & Bake Scene", "Serialize project to .luma & .bsn", "Ctrl + S", ChipVariant::Neutral);
                                    spawn_separator(card, SeparatorProps::horizontal());
                                    spawn_info_row(card, &font, &icon_font, Icon::KEY, "Quick Node Palette", "Open fuzzy search node palette", "Space", ChipVariant::Neutral);
                                    spawn_separator(card, SeparatorProps::horizontal());
                                    spawn_info_row(card, &font, &icon_font, Icon::KEY, "Focus / Dive-In Workspace", "Enter selected node internal subgraph", "Tab", ChipVariant::Neutral);
                                    spawn_separator(card, SeparatorProps::horizontal());
                                    spawn_info_row(card, &font, &icon_font, Icon::ROTATE_CCW, "Undo / Redo History", "Step back or forward in command history", "Ctrl+Z / Ctrl+Y", ChipVariant::Neutral);
                                },
                            );
                        });

                    // ==========================================
                    // Category Panel 6: System & Telemetry
                    // ==========================================
                    content
                        .spawn((
                            BevyUiSettingsCategoryPanel(SettingsCategory::System),
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                display: Display::None,
                                border: UiRect::ZERO,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            spawn_category_header(
                                p,
                                &font,
                                SettingsCategory::System.title(),
                                SettingsCategory::System.description(),
                            );

                            // Jackdaw PanelCard: Engine & Runtime
                            spawn_panel_card(
                                p,
                                &font,
                                Some(&icon_font),
                                Some(Icon::CPU),
                                "ENGINE & RUNTIME ENVIRONMENT",
                                jackdaw_tokens::ACCENT_CYAN,
                                |card| {
                                    spawn_info_row(card, &font, &icon_font, Icon::CPU, "Host Engine", "Bevy 0.19.0 (Entity Component System Architecture)", "v0.19.0", ChipVariant::Neutral);
                                    spawn_separator(card, SeparatorProps::horizontal());
                                    spawn_info_row(card, &font, &icon_font, Icon::ACTIVITY, "UI Architecture", "bevy_luma SDF Vector Hybrid Canvas Renderer", "GPU SDF", ChipVariant::Success);
                                    spawn_separator(card, SeparatorProps::horizontal());
                                    spawn_info_row(card, &font, &icon_font, Icon::INFO, "Vector Icon Library", "luma_style Embedded Lucide Font System", "1,400+ Icons", ChipVariant::Neutral);
                                },
                            );
                        });
                });
            });

        // ==========================================
        // 3. Bottom Action Bar (Seamless, zero borders)
        // ==========================================
        window
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(48.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(0.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    border: UiRect::ZERO,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.055, 0.063, 0.080)),
            ))
            .with_children(|footer| {
                // Shortcut guide on left
                footer.spawn((
                    Text::new("Press Esc or click Done to save and close"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.40, 0.44, 0.52)),
                ));

                // Actions on right
                footer
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        border: UiRect::ZERO,
                        ..default()
                    })
                    .with_children(|actions| {
                        // Reset Defaults Button with Lucide Icon::ROTATE_CCW
                        actions
                            .spawn((
                                BevyUiSettingsResetBtn,
                                Button,
                                Node {
                                    height: Val::Px(28.0),
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(6.0),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    border: UiRect::ZERO,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(Icon::ROTATE_CCW),
                                    TextFont {
                                        font: icon_font.clone(),
                                        font_size: FontSize::Px(11.5),
                                        ..default()
                                    },
                                    TextColor(EditorTheme::TEXT_MUTED),
                                ));
                                btn.spawn((
                                    Text::new("Reset Defaults"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: FontSize::Px(11.0),
                                        ..default()
                                    },
                                    TextColor(EditorTheme::TEXT_MUTED),
                                ));
                            });

                        // Done / Close Accent Button with Lucide Icon::CHECK
                        actions
                            .spawn((
                                BevyUiSettingsCloseBtn,
                                Button,
                                Node {
                                    height: Val::Px(28.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(0.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(6.0),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    border: UiRect::ZERO,
                                    ..default()
                                },
                                BackgroundColor(EditorTheme::ACCENT_PRIMARY),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(Icon::CHECK),
                                    TextFont {
                                        font: icon_font.clone(),
                                        font_size: FontSize::Px(12.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.04, 0.05, 0.07)),
                                ));
                                btn.spawn((
                                    Text::new("Done"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: FontSize::Px(11.5),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.04, 0.05, 0.07)),
                                ));
                            });
                    });
            });
    });
}

// ============================================================================
// Helper UI Builders
// ============================================================================

fn spawn_category_header(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    title: &'static str,
    subtitle: &'static str,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|hdr| {
            hdr.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(EditorTheme::TEXT_PRIMARY),
            ));

            hdr.spawn((
                Text::new(subtitle),
                TextFont {
                    font: font.clone(),
                    font_size: FontSize::Px(11.5),
                    ..default()
                },
                TextColor(EditorTheme::TEXT_MUTED),
            ));
        });
}

fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: &FontSource,
    key: SettingKey,
    item_icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    is_active: bool,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|row| {
            // Left: Icon + Title & Subtitle
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new(item_icon),
                        TextFont {
                            font: icon_font.clone(),
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(EditorTheme::BORDER_FOCUSED),
                    ));
                });

                left.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|info| {
                    info.spawn((
                        Text::new(title),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_PRIMARY),
                    ));
                    info.spawn((
                        Text::new(subtitle),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(10.5),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_MUTED),
                    ));
                });
            });

            // Right: Text Indicator + Jackdaw Toggle Switch
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|right| {
                let status_label = if is_active { "ON" } else { "OFF" };
                let status_color = if is_active {
                    EditorTheme::ACCENT_SUCCESS
                } else {
                    EditorTheme::TEXT_MUTED
                };

                right.spawn((
                    BevyUiSettingIndicatorText(key),
                    Text::new(status_label),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(status_color),
                ));

                spawn_toggle_switch(right, BevyUiSettingToggleBtn(key), is_active);
            });
        });
}

fn spawn_checkbox_row(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: &FontSource,
    key: SettingKey,
    item_icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    is_checked: bool,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|row| {
            // Left: Icon + Title & Subtitle
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new(item_icon),
                        TextFont {
                            font: icon_font.clone(),
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(EditorTheme::BORDER_FOCUSED),
                    ));
                });

                left.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|info| {
                    info.spawn((
                        Text::new(title),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_PRIMARY),
                    ));
                    info.spawn((
                        Text::new(subtitle),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(10.5),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_MUTED),
                    ));
                });
            });

            // Right: Text Indicator + Jackdaw Checkbox
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|right| {
                let status_label = if is_checked { "Active" } else { "Inactive" };
                let status_color = if is_checked {
                    jackdaw_tokens::ACCENT_CYAN
                } else {
                    EditorTheme::TEXT_MUTED
                };

                right.spawn((
                    BevyUiSettingIndicatorText(key),
                    Text::new(status_label),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(status_color),
                ));

                spawn_checkbox(right, icon_font, BevyUiSettingCheckboxBtn(key), is_checked);
            });
        });
}

fn spawn_wire_style_selector_row(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: &FontSource,
    active_style: WireRoutingStyle,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|container| {
            // Header: Icon + Title & Subtitle
            container
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            border: UiRect::ZERO,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(Icon::GIT_BRANCH),
                            TextFont {
                                font: icon_font.clone(),
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(jackdaw_tokens::ACCENT_BLUE),
                        ));
                    });

                    left.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        border: UiRect::ZERO,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new("Dataflow Wire Geometry & Routing Shape"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));
                        info.spawn((
                            Text::new("Choose active wire rendering style or press 'W' in canvas to cycle dynamically"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(10.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_MUTED),
                        ));
                    });
                });

            // Buttons row: All 6 styles
            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|btn_row| {
                    for style in WireRoutingStyle::ALL {
                        let is_active = style == active_style;
                        let bg_color = if is_active {
                            jackdaw_tokens::ACCENT_BLUE
                        } else {
                            Color::srgb(0.10, 0.12, 0.16)
                        };
                        let text_color = if is_active {
                            Color::WHITE
                        } else {
                            Color::srgb(0.65, 0.70, 0.80)
                        };

                        btn_row
                            .spawn((
                                Button,
                                BevyUiWireStyleBtn(style),
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    border: UiRect::ZERO,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(style.label()),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: FontSize::Px(10.5),
                                        ..default()
                                    },
                                    TextColor(text_color),
                                ));
                            });
                    }
                });
        });
}

fn spawn_canvas_pattern_selector_row(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: &FontSource,
    active_pattern: CanvasPatternStyle,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|container| {
            // Header info
            container
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            border: UiRect::ZERO,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(Icon::LAYOUT_GRID),
                            TextFont {
                                font: icon_font.clone(),
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(jackdaw_tokens::ACCENT_CYAN),
                        ));
                    });

                    left.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        border: UiRect::ZERO,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new("Canvas Surface Pattern Style"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));
                        info.spawn((
                            Text::new("Select Dot Matrix, Grid Lines, Crosshairs (+), or Solid. Press 'G' in canvas to cycle"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(10.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_MUTED),
                        ));
                    });
                });

            // Buttons row: All 4 pattern styles
            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|btn_row| {
                    for pat in CanvasPatternStyle::ALL {
                        let is_active = pat == active_pattern;
                        let bg_color = if is_active {
                            jackdaw_tokens::ACCENT_CYAN
                        } else {
                            Color::srgb(0.10, 0.12, 0.16)
                        };
                        let text_color = if is_active {
                            Color::WHITE
                        } else {
                            Color::srgb(0.65, 0.70, 0.80)
                        };

                        btn_row
                            .spawn((
                                Button,
                                BevyUiCanvasPatternBtn(pat),
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    border: UiRect::ZERO,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(pat.label()),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: FontSize::Px(10.5),
                                        ..default()
                                    },
                                    TextColor(text_color),
                                ));
                            });
                    }
                });
        });
}

fn spawn_canvas_palette_selector_row(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: &FontSource,
    active_palette: CanvasColorPalette,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|container| {
            // Header info
            container
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            border: UiRect::ZERO,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(Icon::CONTRAST),
                            TextFont {
                                font: icon_font.clone(),
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(jackdaw_tokens::ACCENT_BLUE),
                        ));
                    });

                    left.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        border: UiRect::ZERO,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new("Canvas Color Palette & Tone"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));
                        info.spawn((
                            Text::new("Select canvas backdrop tone and grid tint. Press 'C' in canvas to cycle palettes"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(10.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_MUTED),
                        ));
                    });
                });

            // Buttons row: All 6 palettes
            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|btn_row| {
                    for pal in CanvasColorPalette::ALL {
                        let is_active = pal == active_palette;
                        let bg_color = if is_active {
                            jackdaw_tokens::ACCENT_BLUE
                        } else {
                            Color::srgb(0.10, 0.12, 0.16)
                        };
                        let text_color = if is_active {
                            Color::WHITE
                        } else {
                            Color::srgb(0.65, 0.70, 0.80)
                        };

                        btn_row
                            .spawn((
                                Button,
                                BevyUiCanvasPaletteBtn(pal),
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    border: UiRect::ZERO,
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                            ))
                            .with_children(|btn| {
                                // Palette preview pip
                                btn.spawn((
                                    Node {
                                        width: Val::Px(8.0),
                                        height: Val::Px(8.0),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        border: UiRect::ZERO,
                                        ..default()
                                    },
                                    BackgroundColor(pal.canvas_bg()),
                                ));

                                btn.spawn((
                                    Text::new(pal.label()),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: FontSize::Px(10.5),
                                        ..default()
                                    },
                                    TextColor(text_color),
                                ));
                            });
                    }
                });
        });
}

fn spawn_info_row(
    parent: &mut ChildSpawnerCommands,
    font: &FontSource,
    icon_font: &FontSource,
    item_icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    badge_label: &'static str,
    badge_variant: ChipVariant,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            ..default()
        })
        .with_children(|row| {
            // Left: Icon + Title & Subtitle
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new(item_icon),
                        TextFont {
                            font: icon_font.clone(),
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(EditorTheme::BORDER_FOCUSED),
                    ));
                });

                left.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    border: UiRect::ZERO,
                    ..default()
                })
                .with_children(|info| {
                    info.spawn((
                        Text::new(title),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_PRIMARY),
                    ));
                    info.spawn((
                        Text::new(subtitle),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(10.5),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_MUTED),
                    ));
                });
            });

            // Right: Jackdaw Chip Badge
            spawn_chip(row, font, badge_label, badge_variant);
        });
}

// ============================================================================
// Systems
// ============================================================================

/// Updates visibility of the settings window based on EditorSettingsState.
pub fn bevy_ui_settings_visibility_system(
    settings_state: Res<EditorSettingsState>,
    mut roots: Query<&mut Node, With<BevyUiSettingsRoot>>,
) {
    if !settings_state.is_changed() {
        return;
    }

    let display = if settings_state.is_open {
        Display::Flex
    } else {
        Display::None
    };

    for mut node in &mut roots {
        node.display = display;
    }
}

/// Handles user interactions: category selection, setting toggle clicks, reset defaults,
/// close button clicks, open button clicks, menu item clicks, and keyboard shortcuts.
pub fn bevy_ui_settings_interaction_system(
    mut settings_state: ResMut<EditorSettingsState>,
    close_clicks: Query<&Interaction, (With<BevyUiSettingsCloseBtn>, Changed<Interaction>)>,
    open_clicks: Query<&Interaction, (With<BevyUiBtnSettings>, Changed<Interaction>)>,
    category_clicks: Query<
        (&Interaction, &BevyUiSettingsCategoryBtn),
        Changed<Interaction>,
    >,
    toggle_clicks: Query<
        (&Interaction, &BevyUiSettingToggleBtn),
        Changed<Interaction>,
    >,
    checkbox_clicks: Query<
        (&Interaction, &BevyUiSettingCheckboxBtn),
        Changed<Interaction>,
    >,
    wire_style_clicks: Query<
        (&Interaction, &BevyUiWireStyleBtn),
        Changed<Interaction>,
    >,
    canvas_pattern_clicks: Query<
        (&Interaction, &BevyUiCanvasPatternBtn),
        Changed<Interaction>,
    >,
    canvas_palette_clicks: Query<
        (&Interaction, &BevyUiCanvasPaletteBtn),
        Changed<Interaction>,
    >,
    reset_clicks: Query<&Interaction, (With<BevyUiSettingsResetBtn>, Changed<Interaction>)>,
    menu_item_clicks: Query<
        (&Interaction, &crate::menu_bar::BevyUiMenuItemButton),
        Changed<Interaction>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Open from top header button
    for interaction in &open_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.is_open = true;
        }
    }

    // Open from Menu Bar (Edit -> Settings)
    for (interaction, btn) in &menu_item_clicks {
        if *interaction == Interaction::Pressed
            && btn.0 == crate::menu_bar::MenuItemAction::EditSettings
        {
            settings_state.is_open = true;
        }
    }

    // Close button
    for interaction in &close_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.is_open = false;
        }
    }

    // Category switching
    for (interaction, cat_btn) in &category_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.active_category = cat_btn.0;
        }
    }

    // Setting toggle switch clicks
    for (interaction, toggle_btn) in &toggle_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.toggle(toggle_btn.0);
        }
    }

    // Setting checkbox clicks
    for (interaction, cb_btn) in &checkbox_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.toggle(cb_btn.0);
        }
    }

    // Setting wire style button clicks
    for (interaction, btn) in &wire_style_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.wire_style = btn.0;
        }
    }

    // Setting canvas pattern button clicks
    for (interaction, btn) in &canvas_pattern_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.canvas_pattern = btn.0;
        }
    }

    // Setting canvas palette button clicks
    for (interaction, btn) in &canvas_palette_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.canvas_palette = btn.0;
        }
    }

    // Reset Defaults
    for interaction in &reset_clicks {
        if *interaction == Interaction::Pressed {
            settings_state.reset_defaults();
        }
    }

    // Keyboard Shortcuts: Ctrl+, to toggle, Escape to close
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::Comma) {
        settings_state.is_open = !settings_state.is_open;
    }

    if settings_state.is_open && keys.just_pressed(KeyCode::Escape) {
        settings_state.is_open = false;
    }

    // Keyboard Shortcut 'W': cycle wire routing style
    if !ctrl
        && !keys.pressed(KeyCode::AltLeft)
        && !keys.pressed(KeyCode::ShiftLeft)
        && keys.just_pressed(KeyCode::KeyW)
    {
        settings_state.wire_style = settings_state.wire_style.next();
    }

    // Keyboard Shortcut 'G': cycle canvas pattern style (Dots -> Grid -> Crosses -> None)
    if !ctrl
        && !keys.pressed(KeyCode::AltLeft)
        && !keys.pressed(KeyCode::ShiftLeft)
        && keys.just_pressed(KeyCode::KeyG)
    {
        settings_state.canvas_pattern = settings_state.canvas_pattern.next();
    }

    // Keyboard Shortcut 'C': cycle canvas color palette
    if !ctrl
        && !keys.pressed(KeyCode::AltLeft)
        && !keys.pressed(KeyCode::ShiftLeft)
        && keys.just_pressed(KeyCode::KeyC)
    {
        settings_state.canvas_palette = settings_state.canvas_palette.next();
    }
}

/// Synchronizes HUD elements (Minimap, Status Bar, Colorblind mode) with settings.
pub fn bevy_ui_settings_hud_sync_system(
    mut settings: ResMut<EditorSettingsState>,
    mut access_res: ResMut<EditorAccessibilitySettings>,
    mut minimap_q: Query<&mut Node, (With<BevyUiMinimapRoot>, Without<BevyUiStatusBarRoot>)>,
    mut status_bar_q: Query<&mut Node, (With<BevyUiStatusBarRoot>, Without<BevyUiMinimapRoot>)>,
) {
    // Two-way sync between AccessibilitySettings and EditorSettingsState for Colorblind mode
    if access_res.is_changed() && !settings.is_changed() {
        if settings.colorblind_mode != access_res.colorblind_mode {
            settings.colorblind_mode = access_res.colorblind_mode;
        }
    } else if settings.is_changed() {
        if access_res.colorblind_mode != settings.colorblind_mode {
            access_res.colorblind_mode = settings.colorblind_mode;
        }
    }

    if !settings.is_changed() {
        return;
    }

    for mut node in &mut minimap_q {
        node.display = if settings.show_minimap {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut status_bar_q {
        node.display = if settings.show_telemetry {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Synchronizes category panels and sidebar buttons whenever settings.active_category changes.
pub fn bevy_ui_settings_category_sync_system(
    settings: Res<EditorSettingsState>,
    mut panel_q: Query<(&BevyUiSettingsCategoryPanel, &mut Node)>,
    mut cat_btn_q: Query<(&BevyUiSettingsCategoryBtn, &mut BackgroundColor)>,
    mut breadcrumb_q: Query<&mut Text, With<BevyUiSettingsBreadcrumbText>>,
) {
    if !settings.is_changed() {
        return;
    }

    for (panel, mut node) in &mut panel_q {
        node.display = if panel.0 == settings.active_category {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (btn, mut bg) in &mut cat_btn_q {
        *bg = if btn.0 == settings.active_category {
            BackgroundColor(Color::srgb(0.13, 0.15, 0.20))
        } else {
            BackgroundColor(Color::NONE)
        };
    }

    for mut text in &mut breadcrumb_q {
        **text = settings.active_category.title().to_string();
    }
}

/// Synchronizes Jackdaw toggle switches and checkboxes whenever settings change.
pub fn bevy_ui_settings_toggle_sync_system(
    settings: Res<EditorSettingsState>,
    mut toggle_track_q: Query<
        (&BevyUiSettingToggleBtn, &mut BackgroundColor, &mut Node),
        (
            Without<BevyUiSettingCheckboxBtn>,
            Without<BevyUiWireStyleBtn>,
            Without<BevyUiCanvasPatternBtn>,
            Without<BevyUiCanvasPaletteBtn>,
        ),
    >,
    mut checkbox_btn_q: Query<
        (&BevyUiSettingCheckboxBtn, &mut BackgroundColor),
        (
            Without<BevyUiSettingToggleBtn>,
            Without<BevyUiWireStyleBtn>,
            Without<BevyUiCanvasPatternBtn>,
            Without<BevyUiCanvasPaletteBtn>,
        ),
    >,
    mut wire_btn_q: Query<
        (&BevyUiWireStyleBtn, &mut BackgroundColor, &Children),
        (
            Without<BevyUiSettingToggleBtn>,
            Without<BevyUiSettingCheckboxBtn>,
            Without<BevyUiCanvasPatternBtn>,
            Without<BevyUiCanvasPaletteBtn>,
        ),
    >,
    mut pattern_btn_q: Query<
        (&BevyUiCanvasPatternBtn, &mut BackgroundColor, &Children),
        (
            Without<BevyUiSettingToggleBtn>,
            Without<BevyUiSettingCheckboxBtn>,
            Without<BevyUiWireStyleBtn>,
            Without<BevyUiCanvasPaletteBtn>,
        ),
    >,
    mut palette_btn_q: Query<
        (&BevyUiCanvasPaletteBtn, &mut BackgroundColor, &Children),
        (
            Without<BevyUiSettingToggleBtn>,
            Without<BevyUiSettingCheckboxBtn>,
            Without<BevyUiWireStyleBtn>,
            Without<BevyUiCanvasPatternBtn>,
        ),
    >,
    mut checkmark_q: Query<
        (&BevyUiSettingCheckboxBtn, &mut Text),
        (
            With<crate::jackdaw_widgets::JackdawCheckmark>,
            Without<BevyUiSettingIndicatorText>,
        ),
    >,
    mut indicator_q: Query<
        (&BevyUiSettingIndicatorText, &mut Text, &mut TextColor),
        Without<crate::jackdaw_widgets::JackdawCheckmark>,
    >,
    mut text_color_q: Query<
        &mut TextColor,
        (
            Without<BevyUiSettingIndicatorText>,
            Without<crate::jackdaw_widgets::JackdawCheckmark>,
        ),
    >,
) {
    if !settings.is_changed() {
        return;
    }

    // 1. Sync Toggle Switches (Track background & thumb sliding position)
    for (btn, mut bg, mut node) in &mut toggle_track_q {
        let is_active = settings.get(btn.0);
        *bg = if is_active {
            BackgroundColor(jackdaw_tokens::ACCENT_BLUE)
        } else {
            BackgroundColor(Color::srgb(0.18, 0.20, 0.25))
        };
        node.justify_content = if is_active {
            JustifyContent::FlexEnd
        } else {
            JustifyContent::FlexStart
        };
    }

    // 2. Sync Checkbox Button Backgrounds
    for (btn, mut bg) in &mut checkbox_btn_q {
        let is_checked = settings.get(btn.0);
        *bg = if is_checked {
            BackgroundColor(jackdaw_tokens::ACCENT_BLUE)
        } else {
            BackgroundColor(Color::srgb(0.16, 0.18, 0.24))
        };
    }

    // 3. Sync Wire Style Button Highlights
    for (btn, mut bg, children) in &mut wire_btn_q {
        let is_selected = btn.0 == settings.wire_style;
        *bg = if is_selected {
            BackgroundColor(jackdaw_tokens::ACCENT_BLUE)
        } else {
            BackgroundColor(Color::srgb(0.10, 0.12, 0.16))
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_q.get_mut(child) {
                *text_color = if is_selected {
                    TextColor(Color::WHITE)
                } else {
                    TextColor(Color::srgb(0.65, 0.70, 0.80))
                };
            }
        }
    }

    // 4. Sync Canvas Pattern Style Button Highlights
    for (btn, mut bg, children) in &mut pattern_btn_q {
        let is_selected = btn.0 == settings.canvas_pattern;
        *bg = if is_selected {
            BackgroundColor(jackdaw_tokens::ACCENT_CYAN)
        } else {
            BackgroundColor(Color::srgb(0.10, 0.12, 0.16))
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_q.get_mut(child) {
                *text_color = if is_selected {
                    TextColor(Color::WHITE)
                } else {
                    TextColor(Color::srgb(0.65, 0.70, 0.80))
                };
            }
        }
    }

    // 5. Sync Canvas Palette Button Highlights
    for (btn, mut bg, children) in &mut palette_btn_q {
        let is_selected = btn.0 == settings.canvas_palette;
        *bg = if is_selected {
            BackgroundColor(jackdaw_tokens::ACCENT_BLUE)
        } else {
            BackgroundColor(Color::srgb(0.10, 0.12, 0.16))
        };
        for child in children.iter() {
            if let Ok(mut text_color) = text_color_q.get_mut(child) {
                *text_color = if is_selected {
                    TextColor(Color::WHITE)
                } else {
                    TextColor(Color::srgb(0.65, 0.70, 0.80))
                };
            }
        }
    }

    // 6. Sync Checkbox Checkmark Glyphs
    for (btn, mut text) in &mut checkmark_q {
        let is_checked = settings.get(btn.0);
        **text = if is_checked {
            Icon::CHECK.to_string()
        } else {
            String::new()
        };
    }

    // 7. Sync Text Status Indicators ("ON" / "OFF" or "Active" / "Inactive")
    for (ind, mut text, mut color) in &mut indicator_q {
        let is_on = settings.get(ind.0);
        match ind.0 {
            SettingKey::SnapToGrid
            | SettingKey::SnapPins
            | SettingKey::HighDpiFont
            | SettingKey::ReducedMotion => {
                **text = if is_on { "Active".to_string() } else { "Inactive".to_string() };
                color.0 = if is_on {
                    jackdaw_tokens::ACCENT_CYAN
                } else {
                    EditorTheme::TEXT_MUTED
                };
            }
            _ => {
                **text = if is_on { "ON".to_string() } else { "OFF".to_string() };
                color.0 = if is_on {
                    EditorTheme::ACCENT_SUCCESS
                } else {
                    EditorTheme::TEXT_MUTED
                };
            }
        }
    }
}

/// Hover visual feedback for settings buttons with modern solid colors.
/// Uses a single query with Option markers to guarantee zero B0001 system parameter conflicts.
pub fn bevy_ui_settings_hover_system(
    mut buttons: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&BevyUiSettingsCloseBtn>,
            Option<&BevyUiSettingsResetBtn>,
        ),
        (With<Button>, Changed<Interaction>),
    >,
) {
    for (interaction, mut bg, close_opt, reset_opt) in &mut buttons {
        if close_opt.is_some() {
            match *interaction {
                Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.40, 0.88, 1.0)),
                Interaction::Pressed => *bg = BackgroundColor(Color::srgb(0.18, 0.75, 0.90)),
                Interaction::None => *bg = BackgroundColor(EditorTheme::ACCENT_PRIMARY),
            }
        } else if reset_opt.is_some() {
            match *interaction {
                Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.26)),
                Interaction::Pressed => *bg = BackgroundColor(Color::srgb(0.14, 0.16, 0.20)),
                Interaction::None => *bg = BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
            }
        }
    }
}
