//! Face sizes of cockpit units, in metres, from the manufacturers' documents.

pub struct Face {
    pub width: f32,
    pub height: f32,
}

/// Collins AFD-5220E adaptive flight display, mounted upright.
pub const AFD: Face = Face { width: 0.2499, height: 0.2987 };
/// Collins CDU-6200 control display unit.
pub const CDU: Face = Face { width: 0.146, height: 0.162 };
/// Collins CCP-5060 cursor control panel.
pub const CCP: Face = Face { width: 0.146, height: 0.086 };
/// Collins DBU-5010E data base unit.
pub const DBU: Face = Face { width: 0.146, height: 0.057 };
/// Collins DCP-5060 display control panel.
pub const DCP: Face = Face { width: 0.215, height: 0.048 };
/// Collins FCP-4003 flight control panel.
pub const FCP: Face = Face { width: 0.381, height: 0.048 };

/// Width of a console panel held by quarter-turn fasteners.
pub const CONSOLE_PANEL_WIDTH: f32 = 0.146;
/// Console panel heights come in multiples of this (0.375 in).
pub const CONSOLE_PANEL_STEP: f32 = 0.009525;
