//! The controls on the overhead panel.
//!
//! Every position is measured from the manufacturer's overhead panel
//! figure. The numbers are that figure's own coordinates, kept as measured
//! so they can be checked against it; `SCALE` turns them into metres using
//! the known width of a console panel column.

use crate::font;
use crate::hardware::CONSOLE_PANEL_WIDTH;
use crate::labels::Label;
use crate::part::{Scene, Surface};

use super::controls::{button, gauge, knob, switchlight, switchlight_cover, toggle};

/// Figure units across one column of panels.
const COLUMN_IN_FIGURE: f32 = 249.3;
/// Metres per figure unit.
pub const SCALE: f32 = CONSOLE_PANEL_WIDTH / COLUMN_IN_FIGURE;
/// The figure's coordinates of the middle of the panel, across, and of the
/// top edge of the system panels.
const MIDDLE_X: f32 = 880.0;
const TOP_Y: f32 = 225.0;
/// Middle of the strip of lighting panels along the front.
const STRIP_Y: f32 = 832.0;

/// Panel heights down each column, in figure units from `TOP_Y`.
/// Left: electrical power, APU, hydraulic, lights.
pub const LEFT_PANELS: [f32; 4] = [212.0, 63.0, 85.0, 145.0];
/// Centre: engine start and ignition, fuel, air conditioning.
pub const CENTRE_PANELS: [f32; 3] = [115.0, 195.0, 195.0];
/// Right: cabin pressurization, bleed air, anti-ice.
pub const RIGHT_PANELS: [f32; 3] = [212.0, 130.0, 163.0];

const TOGGLES: [(f32, f32); 31] = [
    // Electrical power: battery master, generator 1, APU generator, generator 2.
    (577.0, 272.0), (547.0, 335.0), (630.0, 333.0), (713.0, 335.0),
    // Hydraulic pumps 1B, 3A, 3B, 2B.
    (592.0, 537.0), (635.0, 537.0), (672.0, 537.0), (715.0, 537.0),
    // Miscellaneous lights: boarding, cabin, indicator lights.
    (540.0, 618.0), (583.0, 620.0), (637.0, 620.0),
    // External lights: navigation, beacon, anti-collision, logo, wing inspection.
    (540.0, 690.0), (590.0, 690.0), (625.0, 690.0), (660.0, 690.0), (715.0, 690.0),
    // Air conditioning: cabin temperature control transfer.
    (880.0, 697.0),
    // Cabin pressurization: manual altitude.
    (1065.0, 282.0),
    // Anti-ice: wing, ice detector test, windshield left and right, probes left and right.
    (1048.0, 613.0), (1210.0, 613.0), (1048.0, 677.0), (1117.0, 677.0), (1170.0, 677.0), (1215.0, 677.0),
    // Bleed air: duct monitor test side of the selector is a knob; no toggles.
    // Front strip, measured from `STRIP_Y`: landing lights left, nose, right; recognition and taxi;
    // locator transmitter; no smoking, seat belts, emergency lights.
    (552.0, 840.0), (596.0, 840.0), (640.0, 840.0), (710.0, 840.0), (1040.0, 832.0), (1095.0, 832.0), (1147.0, 832.0),
];

const SWITCHLIGHTS: [(f32, f32); 36] = [
    // Electrical power: external AC and DC, auto transfer left and right,
    // essential power, AC/DC utility, DC power.
    (662.0, 265.0), (705.0, 267.0), (590.0, 358.0), (672.0, 358.0), (557.0, 405.0), (630.0, 405.0), (705.0, 405.0),
    // APU: power and fuel, start and stop.
    (598.0, 465.0), (665.0, 465.0),
    // Engine start and ignition: start left and right, stop left and right,
    // ignition A and B, continuous.
    (802.0, 262.0), (847.0, 262.0), (802.0, 305.0), (847.0, 305.0), (900.0, 262.0), (900.0, 305.0), (970.0, 283.0),
    // Fuel: boost pumps, crossflow row, tail tank transfer row.
    (815.0, 378.0), (945.0, 378.0), (815.0, 432.0), (880.0, 432.0), (945.0, 432.0), (802.0, 492.0), (853.0, 492.0), (907.0, 492.0), (958.0, 492.0),
    // Air conditioning: packs left and right, ram air.
    (860.0, 578.0), (902.0, 578.0), (880.0, 632.0),
    // Cabin pressurization: pressure control, emergency depressurization.
    (1065.0, 402.0), (1168.0, 402.0),
    // Bleed air: 14th stage row of three. The 10th stage row is in `MORE_SWITCHLIGHTS`.
    (1143.0, 482.0), (1180.0, 482.0), (1217.0, 482.0),
    // Anti-ice: wing, cowl left and right.
    (1090.0, 613.0), (1132.0, 613.0), (1170.0, 613.0),
];

/// Bleed air 10th stage row, and the locator transmitter light on the strip.
const MORE_SWITCHLIGHTS: [(f32, f32); 5] = [(1105.0, 537.0), (1143.0, 537.0), (1180.0, 537.0), (1217.0, 537.0), (833.0, 830.0)];

/// Knobs with their radius in figure units: overhead lights dimmer, cockpit
/// and cabin standby, cockpit and cabin temperature, manual rate, duct
/// monitor.
const KNOBS: [(f32, f32, f32); 7] = [(708.0, 620.0, 21.0), (803.0, 592.0, 20.0), (960.0, 592.0, 20.0), (818.0, 668.0, 24.0), (943.0, 668.0, 24.0), (1065.0, 335.0, 17.0), (1048.0, 492.0, 20.0)];

/// Cabin altitude and differential pressure indicator.
const GAUGE: (f32, f32, f32) = (1155.0, 310.0, 40.0);
/// Windshield test and flight data recorder event buttons.
const BUTTONS: [(f32, f32); 2] = [(1082.0, 677.0), (1210.0, 832.0)];

/// Converts a height in figure units to metres.
pub fn metres(figure_units: f32) -> f32 {
    figure_units * SCALE
}

/// Names of the toggle switches, in the order of `TOGGLES`.
const TOGGLE_NAMES: [&str; 31] = [
    "BATT MASTER", "GEN 1", "APU GEN", "GEN 2", "HYD 1B", "HYD 3A", "HYD 3B", "HYD 2B", "BOARD", "CABIN", "IND LTS", "NAV", "BEACON", "A/COLL", "LOGO", "WING INSP", "CONT XFER", "MAN ALT",
    "WING A/I", "DET TEST", "WSHLD L", "WSHLD R", "PROBES L", "PROBES R", "LDG LEFT", "LDG NOSE", "LDG RIGHT", "RECOG/TAXI", "NO SMKG", "SEAT BLTS", "EMER LTS",
];
/// Names of the switchlights, in the order of `SWITCHLIGHTS` then `MORE_SWITCHLIGHTS`.
const SWITCHLIGHT_NAMES: [&str; 41] = [
    "EXT AC", "EXT DC", "AUTOXFER 1", "AUTOXFER 2", "ESS POWER", "AC/DC UTIL", "DC POWER", "PWR FUEL", "START/STOP", "START L", "START R", "STOP L", "STOP R", "IGN A", "IGN B", "CONT",
    "BOOST L", "BOOST R", "L TO AUX", "GRAVITY", "R TO AUX", "XFER PRI", "MAN OVRD", "XFER SEC", "DUMP", "PACK L", "PACK R", "RAM AIR", "PRESS CONT", "EMER DEPR", "14TH L", "14TH ISOL", "14TH R",
    "WING OVHT", "COWL L", "COWL R", "10TH L", "APU LCV", "10TH ISOL", "10TH R", "ELT",
];
/// What is written on each switchlight's face, in the order of the
/// switchlights; a `/` separates the upper line from the lower.
const CAPTIONS: [&str; 41] = [
    "AVAIL/IN USE", "IN USE", "FAIL/OFF", "FAIL/OFF", "ALTN", "OFF", "CLOSED", "PUMP/SOV", "START/AVAIL", "START", "START", "STOP", "STOP", "ARM/ON", "ARM/ON", "ON",
    "INOP/ON", "INOP/ON", "ON", "OPEN", "ON", "FAIL/ON", "MANUAL", "FAIL/ON", "OPEN", "FAIL/OFF", "FAIL/OFF", "OPEN", "FAIL/MANUAL", "ON", "FAIL/CLOSED", "OPEN", "FAIL/CLOSED",
    "L HEAT/R HEAT", "ON", "ON", "FAIL/CLOSED", "FAIL/OPEN", "OPEN", "FAIL/CLOSED", "ON",
];

/// What is marked under a toggle switch, for its lower position.
fn lower_position(name: &str) -> &'static str {
    match name {
        "GEN 1" | "APU GEN" | "GEN 2" => "OFF/RESET",
        "IND LTS" => "DIM",
        "MAN ALT" => "DN",
        "CONT XFER" => "CABIN",
        "WING A/I" => "NORM",
        "DET TEST" => "TEST",
        _ => "OFF",
    }
}

const KNOB_NAMES: [&str; 7] = ["OVHD", "CKPT STBY", "CABIN STBY", "CKPT TEMP", "CABIN TEMP", "MAN RATE", "DUCT MON"];
const BUTTON_NAMES: [&str; 2] = ["WSHLD TEST", "FDR EVENT"];
/// Panel titles down each column, matching the panel height lists.
const LEFT_TITLES: [&str; 4] = ["ELECTRICAL POWER", "APU", "HYDRAULIC", "LIGHTS"];
const CENTRE_TITLES: [&str; 3] = ["ENGINE START", "FUEL", "AIR CONDITIONING"];
const RIGHT_TITLES: [&str; 3] = ["CABIN PRESSURIZATION", "BLEED AIR", "ANTI-ICE"];

/// A dataref name for a control on this panel.
pub fn dataref(kind: &str, name: &str) -> String {
    let slug: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    format!("cl650/overhead/{kind}_{slug}")
}

const LEGEND_HEIGHT: f32 = 0.0021;
const LEGEND_PROUD: f32 = 0.0024;
const WHITE: [u8; 3] = [238, 238, 232];
const AMBER: [u8; 3] = [244, 172, 44];
const GREEN: [u8; 3] = [84, 222, 112];

/// The colour a caption lights in: amber for a fault, green for running
/// normally, white for a plain state.
fn caption_colour(word: &str) -> [u8; 3] {
    match word {
        "FAIL" | "INOP" | "PUMP" | "SOV" => AMBER,
        "ON" | "AVAIL" | "ARM" | "L HEAT" | "R HEAT" => GREEN,
        _ => WHITE,
    }
}

/// Sub-group headings between the rows of controls, at their places on the
/// figure.
const HEADINGS: [(f32, f32, &str); 20] = [
    (683.0, 246.0, "EXT POWER"), (630.0, 308.0, "AC POWER"), (825.0, 236.0, "L - ENG - R"), (930.0, 236.0, "IGNITION"), (880.0, 357.0, "BOOST PUMP"),
    (880.0, 409.0, "XFLOW"), (880.0, 465.0, "TAIL TANK TRANSFER"), (880.0, 556.0, "PACK"), (1165.0, 252.0, "SELECTOR"), (1180.0, 456.0, "14TH STAGE"),
    (1160.0, 512.0, "10TH STAGE"), (1070.0, 588.0, "WING"), (1150.0, 588.0, "COWL"), (1082.0, 650.0, "WSHLD/WIND"), (1192.0, 650.0, "PROBES"),
    (558.0, 589.0, "MISC LTS"), (625.0, 661.0, "EXTERNAL LTS"), (595.0, 808.0, "LANDING LTS"), (1067.0, 802.0, "PASS SIGNS"), (880.0, 800.0, "EMER LOCATOR"),
];

/// Switchlights with a hinged cover over them, and the colour of its frame.
const COVERED: [(&str, [u8; 3]); 3] = [("DUMP", [226, 190, 40]), ("EMER DEPR", [176, 40, 36]), ("ELT", [176, 40, 36])];

/// Keeps legends from being put on top of each other or on a control.
#[derive(Default)]
struct Placer {
    taken: Vec<[f32; 4]>,
}

impl Placer {
    fn reserve(&mut self, u: f32, v: f32, half_u: f32, half_v: f32) {
        self.taken.push([u - half_u, v - half_v, u + half_u, v + half_v]);
    }

    /// Adds a legend if nothing is in its way, and says whether it did.
    fn place(&mut self, scene: &mut Scene, surface: &Surface, u: f32, v: f32, height: f32, text: &str, colour: [u8; 3]) -> bool {
        let half = (height * font::aspect(text) / 2.0 + 0.0007, height / 2.0 + 0.0005);
        let wanted = [u - half.0, v - half.1, u + half.0, v + half.1];
        if self.taken.iter().any(|t| wanted[0] < t[2] && t[0] < wanted[2] && wanted[1] < t[3] && t[1] < wanted[3]) {
            return false;
        }
        self.taken.push(wanted);
        scene.labels.push(Label::on(surface, u, v, height, LEGEND_PROUD, text, colour, true));
        true
    }

    /// A ruled line from one u to another, if nothing is in its way.
    fn rule(&mut self, scene: &mut Scene, surface: &Surface, from: f32, to: f32, v: f32) {
        let height = 0.0016;
        let count = ((to - from) / (height * 5.5 / 7.0)) as usize;
        if count >= 3 {
            self.place(scene, surface, (from + to) / 2.0, v, height, &"_".repeat(count), WHITE);
        }
    }
}

/// Adds the controls, each a named moving control, with everything that is
/// written on the panel: titles, sub-group headings and their rules, the
/// name over each control, switch positions, and the captions on the
/// switchlights in the colours they light in. `top` is the surface
/// coordinate of the top edge of the system panels and `strip` that of the
/// middle of the front strip.
pub fn controls(scene: &mut Scene, surface: &Surface, top: f32, strip: f32) {
    let place = |x: f32, y: f32| {
        let u = (x - MIDDLE_X) * SCALE;
        // Anything drawn below the system panels is on the front strip.
        let v = if y > 780.0 { strip - (y - STRIP_Y) * SCALE } else { top - (y - TOP_Y) * SCALE };
        (u, v)
    };
    let mut placer = Placer::default();
    let all_lights = || SWITCHLIGHTS.iter().chain(&MORE_SWITCHLIGHTS).zip(SWITCHLIGHT_NAMES).zip(CAPTIONS);

    // Every control keeps its own patch of panel clear.
    for (x, y) in TOGGLES {
        let (u, v) = place(x, y);
        placer.reserve(u, v + 0.003, 0.0058, 0.0085);
    }
    for (((x, y), _), _) in all_lights() {
        let (u, v) = place(*x, *y);
        placer.reserve(u, v, 0.0102, 0.0102);
    }
    for (x, y, radius) in KNOBS {
        let (u, v) = place(x, y);
        placer.reserve(u, v, radius * SCALE * 1.28, radius * SCALE * 1.28);
    }
    for (x, y) in BUTTONS {
        let (u, v) = place(x, y);
        placer.reserve(u, v, 0.0088, 0.0088);
    }
    let (gauge_u, gauge_v) = place(GAUGE.0, GAUGE.1);
    placer.reserve(gauge_u, gauge_v, GAUGE.2 * SCALE * 1.16, GAUGE.2 * SCALE * 1.16);

    // Panel titles along the top of each panel, with a rule each side where
    // there is room; moved sideways if another title is in the way. Titles and
    // headings are placed before the controls' names, which give way to them.
    let titles: [(f32, &[f32], &[&str]); 3] = [(-1.0, &LEFT_PANELS, &LEFT_TITLES), (0.0, &CENTRE_PANELS, &CENTRE_TITLES), (1.0, &RIGHT_PANELS, &RIGHT_TITLES)];
    for (column, heights, names) in titles {
        let mut down = 0.0;
        for (height, text) in heights.iter().zip(names) {
            let (middle, v) = (column * CONSOLE_PANEL_WIDTH, top - metres(down) - 0.0040);
            let half = 0.0026 * font::aspect(text) / 2.0;
            for shift in [0.0, -0.03, 0.03, -0.045, 0.045] {
                if placer.place(scene, surface, middle + shift, v, 0.0026, text, WHITE) {
                    let edge = CONSOLE_PANEL_WIDTH / 2.0 - 0.012;
                    placer.rule(scene, surface, middle - edge, middle + shift - half - 0.003, v);
                    placer.rule(scene, surface, middle + shift + half + 0.003, middle + edge, v);
                    break;
                }
            }
            down += height;
        }
    }
    for (x, y, text) in HEADINGS {
        let (u, v) = place(x, y);
        if placer.place(scene, surface, u, v, 0.0019, text, WHITE) {
            let half = 0.0019 * font::aspect(text) / 2.0;
            placer.rule(scene, surface, u - half - 0.012, u - half - 0.002, v);
            placer.rule(scene, surface, u + half + 0.002, u + half + 0.012, v);
        }
    }
    // A name goes over its control, or under it if something is in the way.
    let name = |placer: &mut Placer, scene: &mut Scene, u: f32, v: f32, above: f32, text: &str| {
        let _ = placer.place(scene, surface, u, v + above, LEGEND_HEIGHT, text, WHITE) || placer.place(scene, surface, u, v - above - 0.001, LEGEND_HEIGHT, text, WHITE);
    };
    for ((x, y), text) in TOGGLES.iter().zip(TOGGLE_NAMES) {
        let (u, v) = place(*x, *y);
        scene.control(&dataref("switch", text));
        toggle(scene, surface, u, v);
        name(&mut placer, scene, u, v, 0.0135, text);
        placer.place(scene, surface, u, v - 0.0082, 0.0016, lower_position(text), WHITE);
    }
    for (((x, y), text), caption) in all_lights() {
        let (u, v) = place(*x, *y);
        scene.control(&dataref("light", text));
        switchlight(scene, surface, u, v);
        // The caption on the face of the cap, in one line or two.
        let on_cap = |scene: &mut Scene, dv: f32, height: f32, word: &str| scene.labels.push(Label::on(surface, u, v + dv, height, 0.0083, word, caption_colour(word), true));
        match caption.split_once('/') {
            Some((upper, lower)) => {
                on_cap(scene, 0.0031, 0.0019, upper);
                on_cap(scene, -0.0031, 0.0019, lower);
            }
            None => on_cap(scene, 0.0, 0.0021, caption),
        }
        if let Some((_, frame)) = COVERED.iter().find(|(covered, _)| *covered == text) {
            switchlight_cover(scene, surface, u, v, *frame, &format!("{}_cover", dataref("light", text)));
        }
        name(&mut placer, scene, u, v, 0.0128, text);
    }
    for ((x, y, radius), text) in KNOBS.iter().zip(KNOB_NAMES) {
        let (u, v) = place(*x, *y);
        scene.control(&dataref("knob", text));
        knob(scene, surface, u, v, radius * SCALE);
        name(&mut placer, scene, u, v, radius * SCALE * 1.28 + 0.0026, text);
    }
    for ((x, y), text) in BUTTONS.iter().zip(BUTTON_NAMES) {
        let (u, v) = place(*x, *y);
        scene.control(&dataref("button", text));
        button(scene, surface, u, v, 0.007);
        name(&mut placer, scene, u, v, 0.0112, text);
    }
    scene.naming = None;
    gauge(scene, surface, gauge_u, gauge_v, GAUGE.2 * SCALE);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_column_of_panels_is_the_same_length() {
        let total = |heights: &[f32]| heights.iter().sum::<f32>();
        assert_eq!(total(&LEFT_PANELS), 505.0);
        assert_eq!(total(&CENTRE_PANELS), 505.0);
        assert_eq!(total(&RIGHT_PANELS), 505.0);
        assert!((metres(505.0) - 0.2957).abs() < 0.001);
    }

    #[test]
    fn every_control_is_on_the_panel_and_none_overlap() {
        let half_width = 1.5 * COLUMN_IN_FIGURE;
        let mut all: Vec<(f32, f32)> = TOGGLES.iter().chain(&SWITCHLIGHTS).chain(&MORE_SWITCHLIGHTS).chain(&BUTTONS).copied().collect();
        all.extend(KNOBS.iter().map(|k| (k.0, k.1)));
        all.push((GAUGE.0, GAUGE.1));
        for (x, y) in &all {
            assert!((x - MIDDLE_X).abs() < half_width, "{x} is off the side");
            assert!((TOP_Y..=730.0).contains(y) || (795.0..=870.0).contains(y), "{y} is off the panels");
        }
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                let apart = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
                assert!(apart > 28.0, "{a:?} and {b:?} are on top of each other");
            }
        }
    }

    #[test]
    fn every_control_has_a_name_and_no_two_share_a_dataref() {
        assert_eq!(SWITCHLIGHTS.len() + MORE_SWITCHLIGHTS.len(), SWITCHLIGHT_NAMES.len());
        let mut all: Vec<String> = TOGGLE_NAMES.iter().map(|n| dataref("switch", n)).collect();
        all.extend(SWITCHLIGHT_NAMES.iter().map(|n| dataref("light", n)));
        all.extend(KNOB_NAMES.iter().map(|n| dataref("knob", n)));
        let count = all.len();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), count);
        assert_eq!(dataref("switch", "A/COLL"), "cl650/overhead/switch_a_coll");
    }

    #[test]
    fn no_two_legends_on_the_panel_overlap_and_captions_are_colour_coded() {
        let mut scene = Scene::default();
        let surface = Surface::new([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        controls(&mut scene, &surface, 0.15, -0.16);
        // Legends lying on the panel itself; captions stand on the caps.
        let on_panel: Vec<[f32; 4]> = scene.labels.iter().filter(|l| (l.corners[0][2] - LEGEND_PROUD).abs() < 1e-5).map(|l| [l.corners[0][0], l.corners[0][1], l.corners[2][0], l.corners[2][1]]).collect();
        assert!(on_panel.len() > 120, "{}", on_panel.len());
        for (i, a) in on_panel.iter().enumerate() {
            for b in &on_panel[i + 1..] {
                assert!(!(a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]), "{a:?} overlaps {b:?}");
            }
        }
        let colour_of = |text: &str| scene.labels.iter().find(|l| l.text == text).map(|l| l.colour);
        assert_eq!(colour_of("FAIL"), Some(AMBER));
        assert_eq!(colour_of("AVAIL"), Some(GREEN));
        assert_eq!(colour_of("IN USE"), Some(WHITE));
        for heading in ["BOOST PUMP", "XFLOW", "EXTERNAL LTS"] {
            assert!(colour_of(heading).is_some(), "{heading}");
        }
        assert!(scene.labels.iter().any(|l| l.text.starts_with("___")), "rules are drawn");
        let covers = scene.moving.iter().filter(|m| m.dataref.ends_with("_cover")).count();
        assert_eq!(covers, 3);
    }
}
