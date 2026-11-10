//! The controls on the face of the glareshield and on the side panels.
//!
//! Positions are measured from the manufacturer's glareshield and side
//! panels figure, in that figure's coordinates. The glareshield's scale
//! comes from the documented width of the flight control panel; the side
//! panels' from the audio control panel being one console panel wide.

use crate::hardware::{CONSOLE_PANEL_WIDTH, FCP};

use super::figure::{Controls, Figure};

/// Edges of the flight control panel across the figure.
const FCP_EDGES: (f32, f32) = (555.0, 1068.0);
/// Metres per figure unit on the glareshield.
pub const SCALE: f32 = FCP.width / (FCP_EDGES.1 - FCP_EDGES.0);
/// Figure coordinates of the middle of the glareshield face.
pub const MIDDLE: (f32, f32) = (811.0, 265.0);
/// Width and height of the face in figure units.
pub const FACE: (f32, f32) = (1350.0, 94.0);

pub const FIGURE: Figure = Figure { origin: MIDDLE, scale: SCALE };

/// Flight control panel and the groups either side of it.
pub const CONTROLS: Controls = Controls {
    toggles: &[],
    // Left: two lights and a pair; right: two pairs.
    switchlights: &[(455.0, 248.0), (455.0, 287.0), (500.0, 248.0), (500.0, 283.0), (1113.0, 248.0), (1158.0, 248.0), (1113.0, 287.0), (1158.0, 287.0)],
    buttons: &[],
    // Course 1, speed, heading, altitude, course 2; then the two barometric knobs.
    knobs: &[(603.0, 281.0, 12.0), (742.0, 281.0, 12.0), (815.0, 281.0, 13.0), (888.0, 281.0, 13.0), (1018.0, 281.0, 12.0), (412.0, 268.0, 13.0), (1202.0, 268.0, 13.0)],
    gauges: &[],
    pale: true,
    // Mode keys: upper row, lower row, then the autopilot bar and the key above it.
    keys: &[
        (603.0, 252.0, 20.0, 12.0), (640.0, 252.0, 20.0, 12.0), (742.0, 252.0, 20.0, 12.0), (780.0, 252.0, 20.0, 12.0), (815.0, 252.0, 20.0, 12.0),
        (852.0, 252.0, 20.0, 12.0), (888.0, 252.0, 20.0, 12.0), (925.0, 252.0, 20.0, 12.0), (1018.0, 252.0, 20.0, 12.0),
        (640.0, 281.0, 20.0, 12.0), (780.0, 281.0, 20.0, 12.0), (852.0, 281.0, 20.0, 12.0), (925.0, 281.0, 20.0, 12.0),
        (970.0, 279.0, 45.0, 22.0), (970.0, 250.0, 22.0, 10.0),
    ],
};

/// Vertical speed wheel.
pub const WHEEL: (f32, f32) = (678.0, 266.0);
/// Master warning (upper) and master caution (lower), each side.
pub const MASTER_WARNING: [(f32, f32); 2] = [(370.0, 259.0), (1244.0, 259.0)];
pub const MASTER_CAUTION: [(f32, f32); 2] = [(370.0, 277.0), (1244.0, 277.0)];
/// The small annunciator display each side: middle, width, height.
pub const ANNUNCIATORS: [(f32, f32, f32, f32); 2] = [(315.0, 271.0, 60.0, 38.0), (1300.0, 271.0, 60.0, 38.0)];

/// Side panel: metres per figure unit, from the audio control panel.
pub const SIDE_SCALE: f32 = CONSOLE_PANEL_WIDTH / 190.0;
/// Figure coordinates of the middle of the left audio and stall panel box.
pub const SIDE_MIDDLE: (f32, f32) = (588.0, 774.0);
pub const SIDE_FIGURE: Figure = Figure { origin: SIDE_MIDDLE, scale: SIDE_SCALE };
/// Figure units from the panel's inboard hinge line to the middle of the box.
pub const SIDE_BOX_FROM_HINGE: f32 = 122.0;
/// The audio control panel and stall protection panel faces: top, split, bottom.
pub const SIDE_BOX_EDGES: [f32; 3] = [663.0, 800.0, 885.0];

/// Audio control panel, the same way round on both sides.
pub const AUDIO: Controls = Controls {
    toggles: &[(522.0, 783.0)],
    switchlights: &[],
    buttons: &[],
    knobs: &[
        (535.0, 690.0, 7.0), (561.0, 690.0, 7.0), (587.0, 690.0, 7.0), (613.0, 690.0, 7.0), (640.0, 690.0, 7.0),
        (588.0, 727.0, 13.0),
        (522.0, 727.0, 7.0), (548.0, 727.0, 7.0), (627.0, 727.0, 7.0), (653.0, 727.0, 7.0), (522.0, 754.0, 7.0), (548.0, 754.0, 7.0), (627.0, 754.0, 7.0), (653.0, 754.0, 7.0),
        (575.0, 783.0, 7.0), (600.0, 783.0, 7.0), (627.0, 783.0, 7.0), (653.0, 783.0, 7.0),
    ],
    gauges: &[],
    keys: &[],
    pale: false,
};

/// Stall protection panel on the left: pusher switch and the angle of
/// attack test indicator. On the right the two swap sides.
pub const STALL_TOGGLE: (f32, f32) = (527.0, 845.0);
pub const STALL_GAUGE: (f32, f32, f32) = (618.0, 840.0, 33.0);
/// Air outlet: figure units outboard of the hinge, below the box middle, radius.
pub const GASPER: (f32, f32, f32) = (312.0, 64.0, 48.0);
/// Airspeed limits placard: outboard of the hinge, above the box middle, width, height.
pub const PLACARD: (f32, f32, f32, f32) = (319.0, 18.0, 192.0, 58.0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_face_comes_out_at_the_size_measured_elsewhere() {
        assert!((FACE.0 * SCALE - 1.003).abs() < 0.01, "{}", FACE.0 * SCALE);
        assert!((FACE.1 * SCALE - 0.070).abs() < 0.003);
    }

    #[test]
    fn everything_is_on_the_face() {
        let half = (FACE.0 / 2.0, FACE.1 / 2.0);
        let mut all: Vec<(f32, f32)> = CONTROLS.switchlights.to_vec();
        all.extend(CONTROLS.knobs.iter().map(|k| (k.0, k.1)));
        all.extend(CONTROLS.keys.iter().map(|k| (k.0, k.1)));
        all.extend(MASTER_WARNING.iter().chain(&MASTER_CAUTION));
        for (x, y) in all {
            assert!((x - MIDDLE.0).abs() < half.0 && (y - MIDDLE.1).abs() < half.1, "({x}, {y})");
        }
    }
}
