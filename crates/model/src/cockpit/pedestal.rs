//! The panels and controls on the centre pedestal.
//!
//! Positions are measured from the pedestal in the manufacturer's general
//! arrangement figure, in that figure's coordinates. The scale comes from
//! the pedestal being three console panel columns wide; it is confirmed by
//! the control display units, which come out at their documented height.
//!
//! The figure is small, so which kind of control each mark is has been read
//! by eye and some will be wrong; the positions and panel boundaries are
//! what this is good for.

use crate::part::{Scene, Surface};

use super::controls::{toggle_guard, Faces};
use super::figure::{Controls, Figure};

/// Metres per figure unit.
pub const SCALE: f32 = 0.000521;
/// Figure coordinates of the middle of the pedestal's top edge.
const TOP_MIDDLE: (f32, f32) = (533.0, 85.0);
/// Figure units from the top edge to the bottom.
pub const LENGTH: f32 = 1775.0;
/// Figure units from the top edge to the back of the well the thrust
/// lever quadrant sits in.
pub const WELL_END: f32 = 760.0;
/// Middle of each column of panels, in figure units.
const COLUMNS: [f32; 3] = [250.0, 533.0, 812.0];
const COLUMN_WIDTH: f32 = 280.0;

/// Panel boundaries down each column, as figure y from one edge to the
/// next, below the thrust lever quadrant.
const LEFT_EDGES: [f32; 8] = [850.0, 960.0, 1055.0, 1237.0, 1420.0, 1635.0, 1750.0, 1860.0];
const CENTRE_EDGES: [f32; 9] = [850.0, 945.0, 1087.0, 1190.0, 1292.0, 1385.0, 1650.0, 1750.0, 1860.0];
const RIGHT_EDGES: [f32; 7] = [850.0, 960.0, 1055.0, 1237.0, 1550.0, 1640.0, 1750.0];
/// Panel boundaries in the centre column between the control display units.
const TOP_CENTRE_EDGES: [f32; 3] = [85.0, 300.0, 388.0];

/// Middles of the three control display units.
pub const DISPLAY_UNITS: [(f32, f32); 3] = [(250.0, 238.0), (812.0, 238.0), (812.0, 1397.0)];

const CONTROLS: Controls = Controls {
    toggles: &[
        // Between the upper control display units.
        (440.0, 120.0), (500.0, 185.0), (440.0, 260.0), (500.0, 260.0), (615.0, 350.0),
        // Beside the thrust levers, left.
        (205.0, 650.0), (168.0, 715.0), (292.0, 715.0),
        // Engine control.
        (440.0, 900.0), (535.0, 905.0), (590.0, 900.0), (630.0, 900.0),
        // Two rows of guarded switches.
        (440.0, 990.0), (493.0, 990.0), (533.0, 990.0), (575.0, 990.0), (625.0, 990.0), (440.0, 1065.0), (488.0, 1065.0), (533.0, 1065.0), (578.0, 1065.0), (625.0, 1050.0),
        // Bottom row: lighting.
        (150.0, 1695.0), (600.0, 1695.0), (913.0, 1695.0),
    ],
    switchlights: &[
        (500.0, 115.0), (447.0, 350.0), (500.0, 350.0), (553.0, 350.0),
        (210.0, 727.0), (250.0, 727.0),
        // Trim and yaw damper.
        (460.0, 1232.0), (500.0, 1232.0), (590.0, 1232.0), (563.0, 1335.0), (600.0, 1335.0),
        (490.0, 1440.0), (530.0, 1440.0), (572.0, 1440.0),
    ],
    buttons: &[(555.0, 250.0), (697.0, 1785.0), (732.0, 1782.0), (785.0, 1787.0)],
    knobs: &[
        (440.0, 185.0, 16.0),
        // Row behind the quadrant, left and right units.
        (207.0, 912.0, 16.0), (340.0, 912.0, 16.0), (765.0, 912.0, 16.0), (898.0, 912.0, 16.0),
        // Audio control panels.
        (202.0, 1110.0, 14.0), (348.0, 1103.0, 12.0), (293.0, 1193.0, 16.0), (760.0, 1110.0, 14.0), (907.0, 1103.0, 12.0), (853.0, 1193.0, 16.0),
        // Rudder trim.
        (580.0, 1135.0, 35.0),
        // Yaw damper and the panel behind it.
        (468.0, 1335.0, 20.0), (450.0, 1522.0, 20.0), (533.0, 1522.0, 20.0), (617.0, 1522.0, 20.0), (450.0, 1600.0, 20.0), (617.0, 1600.0, 20.0),
        // Under the third control display unit.
        (728.0, 1603.0, 20.0), (812.0, 1603.0, 20.0), (897.0, 1603.0, 20.0),
        // Bottom row: lighting.
        (195.0, 1695.0, 18.0), (265.0, 1695.0, 16.0), (333.0, 1690.0, 20.0), (460.0, 1692.0, 18.0), (533.0, 1692.0, 18.0), (730.0, 1692.0, 20.0), (800.0, 1692.0, 16.0), (868.0, 1692.0, 18.0),
    ],
    gauges: &[],
    pale: false,
    // Keys, each with its width and height: the audio control panels, left
    // then right, and the two units behind the quadrant. Read from the
    // small arrangement figure, so their count and places are approximate.
    keys: &[
        (150.0, 1075.0, 26.0, 18.0), (186.0, 1075.0, 26.0, 18.0), (222.0, 1075.0, 26.0, 18.0), (258.0, 1075.0, 26.0, 18.0), (294.0, 1075.0, 26.0, 18.0), (330.0, 1075.0, 26.0, 18.0),
        (150.0, 1150.0, 26.0, 18.0), (186.0, 1150.0, 26.0, 18.0), (222.0, 1150.0, 26.0, 18.0), (258.0, 1150.0, 26.0, 18.0), (150.0, 1195.0, 26.0, 18.0), (186.0, 1195.0, 26.0, 18.0), (222.0, 1195.0, 26.0, 18.0), (340.0, 1160.0, 26.0, 18.0),
        (712.0, 1075.0, 26.0, 18.0), (748.0, 1075.0, 26.0, 18.0), (784.0, 1075.0, 26.0, 18.0), (820.0, 1075.0, 26.0, 18.0), (856.0, 1075.0, 26.0, 18.0), (892.0, 1075.0, 26.0, 18.0),
        (712.0, 1150.0, 26.0, 18.0), (748.0, 1150.0, 26.0, 18.0), (784.0, 1150.0, 26.0, 18.0), (820.0, 1150.0, 26.0, 18.0), (712.0, 1195.0, 26.0, 18.0), (748.0, 1195.0, 26.0, 18.0), (784.0, 1195.0, 26.0, 18.0), (902.0, 1160.0, 26.0, 18.0),
        (160.0, 868.0, 26.0, 18.0), (196.0, 868.0, 26.0, 18.0), (160.0, 945.0, 26.0, 18.0), (196.0, 945.0, 26.0, 18.0), (232.0, 945.0, 26.0, 18.0), (275.0, 876.0, 26.0, 18.0), (275.0, 910.0, 26.0, 18.0), (275.0, 944.0, 26.0, 18.0),
        (722.0, 868.0, 26.0, 18.0), (758.0, 868.0, 26.0, 18.0), (722.0, 945.0, 26.0, 18.0), (758.0, 945.0, 26.0, 18.0), (794.0, 945.0, 26.0, 18.0), (837.0, 876.0, 26.0, 18.0), (837.0, 910.0, 26.0, 18.0), (837.0, 944.0, 26.0, 18.0),
    ],
};

/// What is written on the pedestal, at its place on the figure: panel
/// titles, group headings, switch positions and the spoiler lever's scale.
pub const TITLES: [(f32, f32, &str); 45] = [
    (440.0, 95.0, "NOSE STEER"), (600.0, 95.0, "LDG GEAR"), (470.0, 150.0, "ANTI SKID"), (470.0, 225.0, "MLG BAY OVHT TEST"), (560.0, 238.0, "DN LCK REL"),
    (268.0, 425.0, "PITCH DISC"), (760.0, 462.0, "ROLL DISC"), (230.0, 612.0, "SPOILERS"), (230.0, 700.0, "EMER STOW"), (230.0, 775.0, "THRUST REVERSER"), (838.0, 545.0, "FLAPS"),
    (345.0, 470.0, "0"), (345.0, 540.0, "1/4"), (345.0, 610.0, "1/2"), (345.0, 680.0, "3/4"), (345.0, 745.0, "MAX"),
    (675.0, 705.0, "IDLE"), (675.0, 738.0, "SHUT OFF"),
    (533.0, 862.0, "ENGINE CONTROL"), (460.0, 938.0, "APR"), (610.0, 938.0, "ENG SPEED"),
    (487.0, 965.0, "FIREX MONITOR"), (600.0, 965.0, "FIRE DET AURAL"), (487.0, 1037.0, "SPS TEST"), (612.0, 1030.0, "LAMP TEST"),
    (475.0, 1180.0, "AIL TRIM"), (585.0, 1183.0, "RUD TRIM"), (480.0, 1204.0, "STAB TRIM"), (590.0, 1204.0, "MACH TRIM"), (480.0, 1278.0, "ENGAGE"),
    (533.0, 1304.0, "YAW DAMPER"), (468.0, 1372.0, "DISC"), (582.0, 1372.0, "ENGAGE"), (533.0, 1400.0, "TUNE INHIBIT PUSH"),
    (450.0, 1482.0, "L DISPLAYS"), (533.0, 1482.0, "EICAS"), (617.0, 1482.0, "R DISPLAYS"), (450.0, 1562.0, "ADC"), (617.0, 1562.0, "AFCS SEL"),
    (728.0, 1565.0, "L IRS"), (812.0, 1565.0, "R IRS"), (897.0, 1565.0, "FMS 3"),
    (250.0, 1652.0, "LIGHTING"), (533.0, 1662.0, "LIGHTING"), (812.0, 1652.0, "LIGHTING"),
];

/// What is written on the keys of a cursor control panel and of a display
/// control panel. The keys' names are right; which key carries which is
/// approximate, as their places are.
const CURSOR_KEYS: [&str; 14] = ["UPR MENU", "ESC", "MEM 1", "CHART", "JSTK", "ZOOM", "LWR MENU", "TFC", "TR/WX", "RADIO", "SUMRY", "AC ELEC", "HYD", "ATC"];
const DISPLAY_KEYS: [&str; 8] = ["MENU", "ESC", "FRMT", "TFC", "TR/WX", "BRG SRC", "REFS", "RDR MENU"];

/// The legend on a key, by its place in the list of keys: the two cursor
/// control panels, then the two display control panels.
pub fn key_legend(index: usize) -> &'static str {
    match index {
        0..=13 => CURSOR_KEYS[index],
        14..=27 => CURSOR_KEYS[index - 14],
        28..=35 => DISPLAY_KEYS[index - 28],
        _ => DISPLAY_KEYS[(index - 36) % 8],
    }
}

/// Aileron trim switch: middle, width, height.
pub const AILERON_TRIM: (f32, f32, f32, f32) = (475.0, 1138.0, 60.0, 44.0);
/// Landing gear lever, between the upper control display units.
pub const GEAR_LEVER: (f32, f32) = (590.0, 150.0);
/// Parking brake handle, on the base strip.
pub const PARKING_BRAKE: (f32, f32) = (197.0, 1812.0);

/// Toggle switches that sit under a red guard. Photographs of the real
/// pedestal show its two rows of switches bare, so none are guarded here;
/// an earlier reading of a small figure had put guards on them.
pub const GUARDED: [(f32, f32); 0] = [];

/// Levers on the quadrant: thrust left and right, flight spoiler, flap.
pub const THRUST_LEVERS: [(f32, f32); 2] = [(483.0, 620.0), (585.0, 620.0)];
pub const SPOILER_LEVER: (f32, f32) = (372.0, 560.0);
pub const FLAP_LEVER: (f32, f32) = (835.0, 640.0);
/// Middle and size of the lever slots, and of the red handles.
pub const SLOTS: [(f32, f32, f32, f32); 3] = [(383.0, 600.0, 26.0, 330.0), (534.0, 600.0, 215.0, 330.0), (838.0, 680.0, 120.0, 235.0)];
pub const RED_HANDLES: [(f32, f32, f32, f32); 6] =
    [(268.0, 490.0, 26.0, 100.0), (760.0, 507.0, 115.0, 32.0), (505.0, 785.0, 40.0, 30.0), (560.0, 785.0, 40.0, 30.0), (490.0, 1825.0, 130.0, 40.0), (610.0, 1828.0, 75.0, 55.0)];

/// Where a point of the figure is on the pedestal: metres right of the
/// centreline and metres back from the top edge.
pub fn position(x: f32, y: f32) -> (f32, f32) {
    ((x - TOP_MIDDLE.0) * SCALE, (y - TOP_MIDDLE.1) * SCALE)
}

/// The pedestal's two surfaces. `along` is measured back from the top
/// edge: the first `slope_length` of it lies on the slope, the rest on the
/// level part.
pub struct Pedestal<'a> {
    pub slope: &'a Surface,
    pub slope_length: f32,
    /// The floor of the well the quadrant sits in, lower than the level part.
    pub well: &'a Surface,
    /// How far back from the top edge the well ends.
    pub well_end: f32,
    pub level: &'a Surface,
    pub level_length: f32,
}

impl Pedestal<'_> {
    /// The surface a distance back from the top edge, and the v on it.
    pub fn at(&self, along: f32) -> (&Surface, f32) {
        if along < self.slope_length {
            (self.slope, self.slope_length / 2.0 - along)
        } else if along < self.well_end {
            (self.well, self.level_length / 2.0 - (along - self.slope_length))
        } else {
            (self.level, self.level_length / 2.0 - (along - self.slope_length))
        }
    }

    fn face(&self, faces: &mut Faces, x: f32, top: f32, bottom: f32, width: f32) {
        let (u, start) = position(x, top);
        let (_, end) = position(x, bottom);
        let (surface, v) = self.at((start + end) / 2.0);
        faces.add(surface, u, v, width * SCALE, end - start);
    }

    pub fn build(&self, scene: &mut Scene) {
        let mut faces = Faces { scene, count: 0 };
        for (x, edges) in [(COLUMNS[0], &LEFT_EDGES[..]), (COLUMNS[1], &CENTRE_EDGES[..]), (COLUMNS[2], &RIGHT_EDGES[..]), (COLUMNS[1], &TOP_CENTRE_EDGES[..])] {
            for pair in edges.windows(2) {
                // The third control display unit takes the place of a panel.
                if x == COLUMNS[2] && pair[0] == 1237.0 {
                    continue;
                }
                self.face(&mut faces, x, pair[0], pair[1], COLUMN_WIDTH);
            }
        }
        // The small panel left of the thrust levers, and the base strip on the right.
        self.face(&mut faces, 230.0, 600.0, 790.0, 180.0);
        self.face(&mut faces, 812.0, 1750.0, 1860.0, COLUMN_WIDTH);

        let figure = Figure { origin: TOP_MIDDLE, scale: SCALE };
        // Controls go on whichever part of the pedestal they are over: the
        // slope, the well, or the level part.
        let zone_of = |y: f32| {
            let along = (y - TOP_MIDDLE.1) * SCALE;
            if along < self.slope_length {
                0
            } else if along < self.well_end {
                1
            } else {
                2
            }
        };
        for zone in 0..3 {
            let toggles: Vec<_> = CONTROLS.toggles.iter().copied().filter(|p| zone_of(p.1) == zone).collect();
            let switchlights: Vec<_> = CONTROLS.switchlights.iter().copied().filter(|p| zone_of(p.1) == zone).collect();
            let buttons: Vec<_> = CONTROLS.buttons.iter().copied().filter(|p| zone_of(p.1) == zone).collect();
            let knobs: Vec<_> = CONTROLS.knobs.iter().copied().filter(|p| zone_of(p.1) == zone).collect();
            let keys: Vec<_> = CONTROLS.keys.iter().copied().filter(|p| zone_of(p.1) == zone).collect();
            let controls = Controls { toggles: &toggles, switchlights: &switchlights, buttons: &buttons, knobs: &knobs, keys: &keys, ..Default::default() };
            // The figure's top edge is `along` = 0: half a slope length up
            // the slope, and further up still on the two level surfaces.
            let below = self.level_length / 2.0 + self.slope_length;
            let (surface, v, prefix) = match zone {
                0 => (self.slope, self.slope_length / 2.0, "cl650/pedestal/upper"),
                1 => (self.well, below, "cl650/pedestal/quadrant"),
                _ => (self.level, below, "cl650/pedestal"),
            };
            controls.add(scene, surface, figure, 0.0, v, prefix);
        }
        for (index, (x, y, _, _)) in CONTROLS.keys.iter().enumerate() {
            let (u, along) = position(*x, *y);
            let (surface, v) = self.at(along);
            scene.legend(surface, u, v, 0.0016, 0.0052, key_legend(index));
        }
        for (number, (x, y)) in GUARDED.iter().enumerate() {
            let (u, along) = position(*x, *y);
            let (surface, v) = self.at(along);
            toggle_guard(scene, surface, u, v, &format!("cl650/pedestal/guard_{:02}", number + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::{CDU, CONSOLE_PANEL_WIDTH};

    #[test]
    fn the_scale_gives_the_documented_unit_sizes() {
        assert!((COLUMN_WIDTH * SCALE - CONSOLE_PANEL_WIDTH).abs() < 0.002);
        // The third control display unit fills its gap in the right column.
        let gap = (RIGHT_EDGES[4] - RIGHT_EDGES[3]) * SCALE;
        assert!((gap - CDU.height).abs() < 0.004, "{gap}");
        assert!((LENGTH * SCALE - 0.925).abs() < 0.002);
    }

    #[test]
    fn every_control_is_on_the_pedestal_and_none_overlap() {
        let mut all: Vec<(f32, f32)> = CONTROLS.toggles.iter().chain(CONTROLS.switchlights).chain(CONTROLS.buttons).copied().collect();
        all.extend(CONTROLS.knobs.iter().map(|k| (k.0, k.1)));
        all.extend(CONTROLS.keys.iter().map(|k| (k.0, k.1)));
        assert_eq!(all.len(), CONTROLS.count());
        for (x, y) in &all {
            let (u, along) = position(*x, *y);
            assert!(u.abs() < 1.5 * CONSOLE_PANEL_WIDTH && (0.0..LENGTH * SCALE).contains(&along), "({x}, {y})");
        }
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert!(((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt() > 30.0, "{a:?} and {b:?}");
            }
        }
    }

    #[test]
    fn every_key_has_a_legend_and_both_panels_of_a_kind_read_the_same() {
        assert_eq!(CONTROLS.keys.len(), 2 * CURSOR_KEYS.len() + 2 * DISPLAY_KEYS.len());
        assert_eq!(key_legend(0), key_legend(14));
        assert_eq!(key_legend(28), key_legend(36));
        assert_eq!(key_legend(43), "RDR MENU");
    }

    #[test]
    fn every_guard_sits_over_a_toggle_switch() {
        for guard in GUARDED {
            assert!(CONTROLS.toggles.contains(&guard), "{guard:?}");
        }
    }

    #[test]
    fn edges_run_down_the_pedestal_in_order() {
        for edges in [&LEFT_EDGES[..], &CENTRE_EDGES[..], &RIGHT_EDGES[..], &TOP_CENTRE_EDGES[..]] {
            assert!(edges.windows(2).all(|pair| pair[1] > pair[0]));
        }
    }
}
