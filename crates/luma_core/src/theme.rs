use bevy::color::Color;

pub struct EditorTheme;

impl EditorTheme {
    /// Canvas background: deep near-black (#0B0C10)
    pub const BG_CANVAS: Color = Color::srgb(0.043, 0.047, 0.063);
    /// Card fill: vertical gradient from #1A1D24 (top) to #14161B (bottom)
    pub const BG_CARD_TOP: Color = Color::srgb(0.102, 0.114, 0.141);
    pub const BG_CARD_BOTTOM: Color = Color::srgb(0.078, 0.086, 0.106);
    pub const BG_CARD: Color = Color::srgb(0.090, 0.100, 0.124);
    pub const BG_CARD_SOLID: Color = Color::srgb(0.078, 0.086, 0.106);
    pub const BG_CARD_PREVIEW: Color = Color::srgb(0.045, 0.050, 0.065);
    pub const BG_CARD_HEADER: Color = Color::srgb(0.060, 0.065, 0.080);

    /// Screen Chrome Backgrounds (Solid dark opaque fill, 100% opacity)
    pub const WINDOW_BG: Color = Color::srgb(0.078, 0.086, 0.106);
    pub const PANEL_BG: Color = Color::srgb(0.10, 0.11, 0.13);
    pub const BG_HEADER: Color = Color::srgb(0.078, 0.086, 0.106);
    pub const BG_FOOTER: Color = Color::srgb(0.078, 0.086, 0.106);

    /// TYPE COLOR SYSTEM (Consistently applied across headers, ports, and wires)
    /// Asset = amber #FFB020
    pub const PIN_ASSET: Color = Color::srgb(1.0, 0.690, 0.125);
    /// Mesh = cyan #3DD6F5
    pub const PIN_MESH: Color = Color::srgb(0.239, 0.839, 0.961);
    /// Material = magenta-pink #FF4D8D
    pub const PIN_MATERIAL: Color = Color::srgb(1.0, 0.302, 0.553);
    /// Entity = emerald #3DDC84 (full brightness)
    pub const PIN_ENTITY: Color = Color::srgb(0.239, 0.863, 0.518);
    /// Float = sky blue #60A5FA
    pub const PIN_FLOAT: Color = Color::srgb(0.376, 0.647, 0.980);
    /// Vec3 = violet #A78BFA
    pub const PIN_VEC3: Color = Color::srgb(0.655, 0.545, 0.980);
    /// Untyped = soft white #F0F2F5
    pub const PIN_UNTYPED: Color = Color::srgb(0.941, 0.949, 0.961);

    /// Header Gradients: Solid opaque colors
    pub const HEADER_ASSET: Color = Color::srgb(0.35, 0.24, 0.08);
    pub const HEADER_MESH: Color = Color::srgb(0.08, 0.28, 0.34);
    pub const HEADER_MATERIAL: Color = Color::srgb(0.34, 0.10, 0.20);
    pub const HEADER_ENTITY: Color = Color::srgb(0.08, 0.30, 0.18);

    /// Tinted Header Gradient Strip (legacy fallback)
    pub const TINT_ASSET: Color = Color::srgb(0.20, 0.14, 0.05);
    pub const TINT_MESH: Color = Color::srgb(0.05, 0.16, 0.20);
    pub const TINT_MATERIAL: Color = Color::srgb(0.20, 0.06, 0.12);
    pub const TINT_ENTITY: Color = Color::srgb(0.05, 0.18, 0.10);

    /// Header Badge: solid pill with white text
    pub const BADGE_BG_HEADER: Color = Color::srgb(0.12, 0.13, 0.16);
    pub const BADGE_TEXT_HEADER: Color = Color::WHITE;

    /// Type Badge Pill Backgrounds: Solid opaque
    pub const BADGE_BG_ASSET: Color = Color::srgb(0.25, 0.18, 0.05);
    pub const BADGE_BG_MESH: Color = Color::srgb(0.05, 0.20, 0.24);
    pub const BADGE_BG_MATERIAL: Color = Color::srgb(0.25, 0.08, 0.14);
    pub const BADGE_BG_ENTITY: Color = Color::srgb(0.06, 0.22, 0.13);

    /// Borders & Highlights (Zero border - modern seamless aesthetic)
    pub const BORDER_DEFAULT: Color = Color::NONE;
    pub const BORDER_TOP_LIGHT: Color = Color::NONE;
    pub const BORDER_HOVER: Color = Color::NONE;
    pub const BORDER_INNER_BLOCK: Color = Color::NONE;
    pub const BORDER_FOCUSED: Color = Color::srgb(0.239, 0.839, 0.961);
    pub const BORDER_SUBTLE: Color = Color::NONE;

    /// Canvas Grid & Axes (Minor: 2-3% white, Major: every 5th line at 5-6%, Axis: 1px at 25%)
    pub const AXIS_25: Color = Color::srgba(0.239, 0.839, 0.961, 0.25);
    pub const AXIS_BLUE: Color = Self::AXIS_25;
    pub const GRID_MINOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.025);
    pub const GRID_MAJOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.055);

    /// Interaction States: Solid opaque
    pub const HOVER_BG: Color = Color::srgb(0.16, 0.18, 0.22);
    pub const ACTIVE_BG: Color = Color::srgb(0.20, 0.23, 0.28);

    /// General Accents
    pub const ACCENT_PRIMARY: Color = Color::srgb(0.239, 0.839, 0.961);
    pub const ACCENT_SUCCESS: Color = Color::srgb(0.239, 0.863, 0.518);
    pub const ACCENT_WARNING: Color = Color::srgb(1.0, 0.690, 0.125);
    pub const ACCENT_DANGER: Color = Color::srgb(1.0, 0.302, 0.553);

    /// Typography Tokens
    /// Primary text: 12-13px at 90% white
    pub const TEXT_PRIMARY: Color = Color::srgba(1.0, 1.0, 1.0, 0.90);
    /// Port labels: 12px at 80% white
    pub const TEXT_LABEL: Color = Color::srgba(1.0, 1.0, 1.0, 0.80);
    /// Secondary text: 11px at 60% white
    pub const TEXT_MUTED: Color = Color::srgba(1.0, 1.0, 1.0, 0.60);
}
