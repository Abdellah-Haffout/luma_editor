use bevy::color::Color;

/// Design tokens synchronized with the official Bevy-Editor Figma specification
/// (`https://www.figma.com/design/fkYfFPSBgnGkhbQd3HOMsL/Bevy-Editor?node-id=10853-12187`)
/// and `jackdaw_feathers::tokens`.
pub struct EditorTheme;

impl EditorTheme {
    /// Canvas background: Figma Bevy-Editor deep slate (#18181C)
    pub const BG_CANVAS: Color = Color::srgb(0.094, 0.094, 0.110);
    /// Node Card fill: Figma Bevy-Editor node body (#24262B)
    pub const BG_CARD_TOP: Color = Color::srgb(0.141, 0.149, 0.169);
    pub const BG_CARD_BOTTOM: Color = Color::srgb(0.122, 0.125, 0.145);
    pub const BG_CARD: Color = jackdaw_feathers::tokens::PANEL_BG;
    pub const BG_CARD_SOLID: Color = Color::srgb(0.141, 0.149, 0.169);
    pub const BG_CARD_PREVIEW: Color = Color::srgb(0.102, 0.106, 0.122);
    pub const BG_CARD_HEADER: Color = jackdaw_feathers::tokens::WINDOW_BG;

    /// Screen Chrome Backgrounds (Figma Bevy-Editor #1F1F24 & #2A2A2E)
    pub const WINDOW_BG: Color = jackdaw_feathers::tokens::WINDOW_BG;
    pub const PANEL_BG: Color = jackdaw_feathers::tokens::PANEL_BG;
    pub const BG_HEADER: Color = jackdaw_feathers::tokens::WINDOW_BG;
    pub const BG_FOOTER: Color = jackdaw_feathers::tokens::STATUS_BAR_BG;
    pub const ELEVATED_BG: Color = jackdaw_feathers::tokens::ELEVATED_BG;

    /// TYPE COLOR SYSTEM (Consistently applied across headers, ports, and wires)
    /// Asset = amber #F5A623
    pub const PIN_ASSET: Color = Color::srgb(0.96, 0.65, 0.14);
    /// Mesh = cyan #3DD6F5
    pub const PIN_MESH: Color = jackdaw_feathers::tokens::GUIDE_LINE;
    /// Material = rose-magenta #F04E88
    pub const PIN_MATERIAL: Color = Color::srgb(0.94, 0.31, 0.53);
    /// Entity = emerald #42B983
    pub const PIN_ENTITY: Color = Color::srgb(0.259, 0.725, 0.514);
    /// Float = Bevy accent blue #206EC8 / #4B96F3
    pub const PIN_FLOAT: Color = Color::srgb(0.294, 0.588, 0.953);
    /// Vec3 = violet #9B72F2
    pub const PIN_VEC3: Color = Color::srgb(0.608, 0.447, 0.949);
    /// Untyped = soft silver #DBDBDB
    pub const PIN_UNTYPED: Color = jackdaw_feathers::tokens::DOC_TAB_ACTIVE_LABEL;

    /// Header Gradients: Solid opaque colors (Figma Bevy-Editor dark header with accent)
    pub const HEADER_ASSET: Color = Color::srgb(0.24, 0.18, 0.10);
    pub const HEADER_MESH: Color = Color::srgb(0.10, 0.21, 0.25);
    pub const HEADER_MATERIAL: Color = Color::srgb(0.24, 0.11, 0.17);
    pub const HEADER_ENTITY: Color = Color::srgb(0.10, 0.22, 0.16);

    /// Tinted Header Gradient Strip (legacy fallback)
    pub const TINT_ASSET: Color = Color::srgb(0.20, 0.14, 0.05);
    pub const TINT_MESH: Color = Color::srgb(0.05, 0.16, 0.20);
    pub const TINT_MATERIAL: Color = Color::srgb(0.20, 0.06, 0.12);
    pub const TINT_ENTITY: Color = Color::srgb(0.05, 0.18, 0.10);

    /// Header Badge: solid pill with white text (Figma #36373B)
    pub const BADGE_BG_HEADER: Color = jackdaw_feathers::tokens::ELEVATED_BG;
    pub const BADGE_TEXT_HEADER: Color = jackdaw_feathers::tokens::DOC_TAB_ACTIVE_LABEL;

    /// Type Badge Pill Backgrounds: Solid opaque
    pub const BADGE_BG_ASSET: Color = jackdaw_feathers::tokens::CHIP_CAUTION_BG;
    pub const BADGE_BG_MESH: Color = Color::srgb(0.10, 0.22, 0.26);
    pub const BADGE_BG_MATERIAL: Color = jackdaw_feathers::tokens::CHIP_WARNING_BG;
    pub const BADGE_BG_ENTITY: Color = jackdaw_feathers::tokens::CHIP_SUCCESS_BG;

    /// Borders & Highlights
    pub const BORDER_DEFAULT: Color = Color::NONE;
    pub const BORDER_TOP_LIGHT: Color = Color::NONE;
    pub const BORDER_HOVER: Color = Color::NONE;
    pub const BORDER_INNER_BLOCK: Color = Color::NONE;
    pub const BORDER_FOCUSED: Color = jackdaw_feathers::tokens::ACCENT_BLUE;
    pub const BORDER_SUBTLE: Color = Color::NONE;

    /// Canvas Grid & Axes (Figma Bevy-Editor subtle grid)
    pub const AXIS_25: Color = Color::srgba(0.126, 0.431, 0.784, 0.30);
    pub const AXIS_BLUE: Color = Self::AXIS_25;
    pub const GRID_MINOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.025);
    pub const GRID_MAJOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.055);

    /// Interaction States: Solid opaque (Figma #36373B / #206EC8)
    pub const HOVER_BG: Color = jackdaw_feathers::tokens::ELEVATED_BG;
    pub const ACTIVE_BG: Color = jackdaw_feathers::tokens::ACCENT_BLUE;

    /// General Accents (Figma Bevy-Editor #206EC8)
    pub const ACCENT_PRIMARY: Color = jackdaw_feathers::tokens::ACCENT_BLUE;
    pub const ACCENT_SUCCESS: Color = jackdaw_feathers::tokens::TEXT_SUCCESS;
    pub const ACCENT_WARNING: Color = jackdaw_feathers::tokens::TEXT_WARNING;
    pub const ACCENT_DANGER: Color = jackdaw_feathers::tokens::TEXT_ERROR;

    /// Typography Tokens (Figma Bevy-Editor #DBDBDB / #A8A8A8)
    pub const TEXT_PRIMARY: Color = jackdaw_feathers::tokens::DOC_TAB_ACTIVE_LABEL;
    pub const TEXT_LABEL: Color = Color::srgb(0.80, 0.81, 0.84);
    pub const TEXT_MUTED: Color = jackdaw_feathers::tokens::TAB_INACTIVE_TEXT;
}
