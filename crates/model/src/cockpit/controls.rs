//! Individual controls and units, mounted on a surface.
//!
//! Sizes of the small controls (toggle levers, switchlight caps, knobs) are
//! measured from the panel figures, where a switchlight cap comes out at
//! about 17 mm square. Their exact part families are not yet identified.

use crate::hardware::{AFD, CDU};
use crate::mesh::{add, scaled};
use crate::labels::Label;
use crate::part::{block, Manipulator, Motion, Scene, Surface};
use crate::shapes::{cylinder, fluted, pillow, rounded_slab, tube};

const TOGGLE: [u8; 3] = [208, 208, 214];
const NUT: [u8; 3] = [120, 122, 128];
const CAP: [u8; 3] = [22, 22, 24];
const CAP_RIM: [u8; 3] = [104, 106, 110];
const KNOB: [u8; 3] = [74, 76, 82];
const POINTER: [u8; 3] = [220, 220, 220];
const BEZEL: [u8; 3] = [58, 60, 64];
const SCREEN: [u8; 3] = [6, 9, 18];
const KEY: [u8; 3] = [30, 32, 36];
const DIAL: [u8; 3] = [12, 12, 14];

fn axes(surface: &Surface) -> [[f32; 3]; 3] {
    [surface.right, surface.up, surface.normal()]
}

/// A toggle switch: a nut on the panel and a lever leaning up the panel.
pub fn toggle(scene: &mut Scene, surface: &Surface, u: f32, v: f32) {
    let foot = surface.point(u, v);
    scene.add("switch nuts", NUT, cylinder(foot, surface.normal(), 0.0055, 0.003, 6));
    let lean = add(scaled(surface.normal(), 0.94), scaled(surface.up, 0.34));
    let path = [add(foot, scaled(lean, 0.002)), add(foot, scaled(lean, 0.011)), add(foot, scaled(lean, 0.019))];
    let flip = Motion::Rotate { axis: surface.right, degrees: (-40.0, 0.0) };
    scene.add("switch boots", [26, 26, 28], cylinder(add(foot, scaled(surface.normal(), 0.003)), surface.normal(), 0.0041, 0.0035, 24));
    scene.add_moving("toggle levers", TOGGLE, tube(&path, 0.0022, 24), foot, flip, Manipulator::Toggle);
    // The rounded tip of the lever.
    scene.add_moving("toggle levers", TOGGLE, pillow(path[2], axes(surface), [0.0033, 0.0033, 0.0033], 1.0, 0.0, 12, 24, 0), foot, flip, Manipulator::Toggle);
}

/// A square switchlight: a rim on the panel and a cap standing out of it.
pub fn switchlight(scene: &mut Scene, surface: &Surface, u: f32, v: f32) {
    let middle = |proud: f32| add(surface.point(u, v), scaled(surface.normal(), proud));
    scene.add("switchlight rims", CAP_RIM, rounded_slab(middle(0.0015), axes(surface), [0.0098, 0.0098, 0.0015], 0.002, 2));
    let press = Motion::Slide { along: scaled(surface.normal(), -1.0), metres: (0.0, 0.002) };
    scene.add_moving("switchlight caps", CAP, rounded_slab(middle(0.004), axes(surface), [0.0085, 0.0085, 0.004], 0.0015, 8), middle(0.0), press, Manipulator::Push);
}

/// A round push button.
pub fn button(scene: &mut Scene, surface: &Surface, u: f32, v: f32, radius: f32) {
    scene.add("switchlight rims", CAP_RIM, cylinder(surface.point(u, v), surface.normal(), radius + 0.0015, 0.002, 16));
    let press = Motion::Slide { along: scaled(surface.normal(), -1.0), metres: (0.0, 0.002) };
    scene.add_moving("switchlight caps", CAP, cylinder(surface.point(u, v), surface.normal(), radius, 0.005, 16), surface.point(u, v), press, Manipulator::Push);
}

/// A rotary knob with a skirt and a pointer line.
pub fn knob_in(scene: &mut Scene, surface: &Surface, u: f32, v: f32, radius: f32, colour: [u8; 3]) {
    let foot = surface.point(u, v);
    scene.add("knob skirts", NUT, cylinder(foot, surface.normal(), radius * 1.25, 0.003, 64));
    let turn = Motion::Rotate { axis: surface.normal(), degrees: (-150.0, 150.0) };
    scene.add_moving("knobs", colour, fluted(foot, surface.normal(), radius, 0.016, 18, 144), foot, turn, Manipulator::Drag);
    let tip = add(add(foot, scaled(surface.normal(), 0.0165)), scaled(surface.up, radius * 0.5));
    scene.add_moving("knob pointers", POINTER, rounded_slab(tip, axes(surface), [0.001, radius * 0.45, 0.0006], 0.0005, 1), foot, turn, Manipulator::Drag);
}

/// A round instrument: a bezel ring with a dial inside it.
pub fn gauge(scene: &mut Scene, surface: &Surface, u: f32, v: f32, radius: f32) {
    scene.add("instrument bezels", BEZEL, cylinder(surface.point(u, v), surface.normal(), radius * 1.14, 0.005, 32));
    scene.add("instrument dials", DIAL, cylinder(surface.point(u, v), surface.normal(), radius, 0.0056, 32));
}

/// An adaptive flight display: the bezel at its documented size with the
/// screen set into it. The screen's share of the face is measured from the
/// instrument panel figure.
pub fn display(scene: &mut Scene, surface: &Surface, u: f32, v: f32, flight: bool) {
    let middle = |proud: f32| add(surface.point(u, v), scaled(surface.normal(), proud));
    scene.add("display bezels", BEZEL, rounded_slab(middle(0.006), axes(surface), [AFD.width / 2.0, AFD.height / 2.0, 0.006], 0.008, 4));
    scene.add("displays", SCREEN, rounded_slab(middle(0.0064), axes(surface), [AFD.width * 0.44, AFD.height * 0.45, 0.0064], 0.003, 2));
    display_stand_in(scene, surface, u, v, AFD.width * 0.88, AFD.height * 0.90, 0.0129, flight);
}

/// A control display unit: the body at its documented size, a screen with
/// six line keys each side, and the keyboard below, every key lettered and
/// a moving control of its own. The key layout is the usual one for this
/// kind of unit, not measured from this one. `id` names the unit in its
/// datarefs.
pub fn control_display_unit(scene: &mut Scene, surface: &Surface, u: f32, v: f32, id: &str) {
    const FUNCTIONS: [&str; 12] = ["DIR", "FPLN", "LEGS", "DEP", "PERF", "DSPL", "MFD", "PREV", "NEXT", "IDX", "EXEC", "MSG"];
    const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
    const OTHERS: [&str; 18] = [".", "/", "+", "-", "SP", "DEL", "CLR", "TUN", "BRT", "DIM", "ATC", "NAV", "COM", "HF", "ADF", "DME", "TCAS", "TAWS"];
    let at = |du: f32, dv: f32, proud: f32| add(surface.point(u + du, v + dv), scaled(surface.normal(), proud));
    let frame = axes(surface);
    let press = Motion::Slide { along: scaled(surface.normal(), -1.0), metres: (0.0, 0.0015) };
    scene.naming = None;
    scene.add("control units", BEZEL, rounded_slab(at(0.0, 0.0, 0.004), frame, [CDU.width / 2.0, CDU.height / 2.0, 0.004], 0.005, 3));
    let screen_v = CDU.height / 2.0 - 0.012 - 0.031;
    scene.add("displays", SCREEN, rounded_slab(at(0.0, screen_v, 0.0044), frame, [0.046, 0.031, 0.0044], 0.002, 2));
    // A stand-in page on the screen until the real one is drawn.
    for (line, (words, colour)) in [("ACT FPLN        1/2", [90u8, 214, 236]), ("ORIGIN     DIST", [236, 236, 230]), ("VIA        TO", [236, 236, 230]), ("DIRECT", [70, 224, 96]), ("PERF INIT", [70, 224, 96]), ("----------", [236, 236, 230])].iter().enumerate() {
        scene.labels.push(Label::on(surface, u, v + screen_v + 0.024 - line as f32 * 0.0096, 0.0046, 0.0089, words, *colour, true));
    }
    for (side, name) in [(-1.0, "l"), (1.0, "r")] {
        for line in 0..6 {
            let key_v = screen_v + 0.024 - line as f32 * 0.0096;
            scene.control(&format!("cl650/cdu_{id}/lsk_{name}{}", line + 1));
            scene.add_moving("unit keys", KEY, rounded_slab(at(side * 0.059, key_v, 0.0052), frame, [0.0055, 0.003, 0.0052], 0.001, 1), at(side * 0.059, key_v, 0.0), press, Manipulator::Push);
        }
    }
    let keyboard_top = screen_v - 0.031 - 0.010;
    let mut letters = LETTERS.chars();
    let mut others = OTHERS.iter();
    for row in 0..8 {
        let columns = if row < 2 { 6 } else { 9 };
        let pitch = 0.126 / columns as f32;
        for c in 0..columns {
            let key_u = -0.063 + pitch * (c as f32 + 0.5);
            let key_v = keyboard_top - row as f32 * 0.0118;
            let legend = if row < 2 {
                FUNCTIONS[row * 6 + c].to_string()
            } else {
                letters.next().map(|l| l.to_string()).or_else(|| others.next().map(|o| o.to_string())).unwrap_or_default()
            };
            let slug: String = legend.chars().map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' }).collect();
            scene.control(&format!("cl650/cdu_{id}/key_{slug}_{row}{c}"));
            scene.add_moving("unit keys", KEY, rounded_slab(at(key_u, key_v, 0.0052), frame, [pitch * 0.36, 0.0042, 0.0052], 0.001, 4), at(key_u, key_v, 0.0), press, Manipulator::Push);
            let height = if legend.chars().count() > 2 { 0.0019 } else { 0.0034 };
            scene.legend(surface, u + key_u, v + key_v, height, 0.0106, &legend);
        }
    }
    scene.naming = None;
}

const GUARD_RED: [u8; 3] = [186, 36, 32];

/// A red guard over a toggle switch: two cheeks and a roof, hinged above
/// the switch so it flips up out of the way. It moves on its own dataref.
pub fn toggle_guard(scene: &mut Scene, surface: &Surface, u: f32, v: f32, dataref: &str) {
    let frame = axes(surface);
    let hinge = surface.point(u, v + 0.0125);
    let lift = Motion::Rotate { axis: surface.right, degrees: (0.0, -105.0) };
    let at = |du: f32, dv: f32, proud: f32| add(surface.point(u + du, v + dv), scaled(surface.normal(), proud));
    scene.control(dataref);
    for side in [-1.0, 1.0] {
        scene.add_moving("switch guards", GUARD_RED, rounded_slab(at(side * 0.0072, 0.0015, 0.0125), frame, [0.0009, 0.011, 0.0125], 0.0008, 2), hinge, lift, Manipulator::Toggle);
    }
    scene.add_moving("switch guards", GUARD_RED, rounded_slab(at(0.0, 0.0015, 0.0255), frame, [0.0081, 0.011, 0.0009], 0.002, 4), hinge, lift, Manipulator::Toggle);
    scene.add_moving("switch guards", GUARD_RED, cylinder(at(-0.0081, 0.0125, 0.002), surface.right, 0.0018, 0.0162, 16), hinge, lift, Manipulator::Toggle);
    scene.naming = None;
}

/// A hinged cover over a switchlight: a coloured frame round it and a
/// clear window over its face. It moves on its own dataref.
pub fn switchlight_cover(scene: &mut Scene, surface: &Surface, u: f32, v: f32, frame_colour: [u8; 3], dataref: &str) {
    let frame = axes(surface);
    let hinge = surface.point(u, v + 0.0122);
    let lift = Motion::Rotate { axis: surface.right, degrees: (0.0, -110.0) };
    let at = |du: f32, dv: f32, proud: f32| add(surface.point(u + du, v + dv), scaled(surface.normal(), proud));
    scene.control(dataref);
    for (du, dv, half) in [(0.0, 0.0113, [0.0122, 0.0009, 0.0052]), (0.0, -0.0113, [0.0122, 0.0009, 0.0052]), (0.0113, 0.0, [0.0009, 0.0122, 0.0052]), (-0.0113, 0.0, [0.0009, 0.0122, 0.0052])] {
        scene.add_moving("switch guards", frame_colour, rounded_slab(at(du, dv, 0.0052), frame, half, 0.0006, 2), hinge, lift, Manipulator::Toggle);
    }
    scene.add_moving("guard windows", [206, 214, 222], rounded_slab(at(0.0, 0.0, 0.0101), frame, [0.0118, 0.0118, 0.0005], 0.0015, 4), hinge, lift, Manipulator::Toggle);
    scene.naming = None;
}

const KEY_CAP: [u8; 3] = [52, 54, 60];
const FASTENER: [u8; 3] = [146, 148, 154];
const FACE_A: [u8; 3] = [62, 72, 88];
const FACE_B: [u8; 3] = [68, 78, 94];

/// A flat rectangular push key.
pub fn key_in(scene: &mut Scene, surface: &Surface, u: f32, v: f32, width: f32, height: f32, colour: [u8; 3]) {
    let middle = add(surface.point(u, v), scaled(surface.normal(), 0.0025));
    let press = Motion::Slide { along: scaled(surface.normal(), -1.0), metres: (0.0, 0.0015) };
    scene.add_moving("keys", colour, rounded_slab(middle, axes(surface), [width / 2.0, height / 2.0, 0.0025], 0.0012, 5), middle, press, Manipulator::Push);
}

/// Adds panel faces, alternating two shades so neighbours can be told
/// apart, each with a quarter-turn fastener in every corner.
pub struct Faces<'a> {
    pub scene: &'a mut Scene,
    pub count: usize,
}

impl Faces<'_> {
    pub fn add(&mut self, surface: &Surface, u: f32, v: f32, width: f32, height: f32) {
        let (name, colour) = if self.count % 2 == 0 { ("panel faces a", FACE_A) } else { ("panel faces b", FACE_B) };
        self.count += 1;
        self.scene.add(name, colour, surface.unit(u, v, width - 0.003, height - 0.003, 0.002));
        if width > 0.05 && height > 0.03 {
            // A fastener in each corner, and more down both edges of a tall
            // panel, about every four centimetres.
            let (reach_u, reach_v) = (width / 2.0 - 0.007, height / 2.0 - 0.007);
            let rows = ((2.0 * reach_v / 0.04).round() as usize).max(1);
            for row in 0..=rows {
                for su in [-1.0, 1.0] {
                    let at = surface.point(u + su * reach_u, v - reach_v + 2.0 * reach_v * row as f32 / rows as f32);
                    self.scene.add("fasteners", FASTENER, cylinder(add(at, scaled(surface.normal(), 0.002)), surface.normal(), 0.003, 0.001, 24));
                    // The slot a screwdriver turns it by.
                    self.scene.add("fastener slots", [38, 38, 42], block(add(at, scaled(surface.normal(), 0.0031)), axes(surface), [0.0023, 0.0004, 0.00015]));
                }
            }
        }
    }

    /// A column of panels down a surface: heights in order from `top`.
    pub fn stack(&mut self, surface: &Surface, u: f32, top: f32, width: f32, heights: &[f32]) {
        let mut v = top;
        for height in heights {
            self.add(surface, u, v - height / 2.0, width, *height);
            v -= height;
        }
    }
}

/// A rotary knob in the usual dark finish.
pub fn knob(scene: &mut Scene, surface: &Surface, u: f32, v: f32, radius: f32) {
    knob_in(scene, surface, u, v, radius, KNOB);
}

/// A flat rectangular push key in the usual dark finish.
pub fn key(scene: &mut Scene, surface: &Surface, u: f32, v: f32, width: f32, height: f32) {
    key_in(scene, surface, u, v, width, height, KEY_CAP);
}

const SKY: [u8; 3] = [44, 112, 226];
const GROUND: [u8; 3] = [150, 96, 42];
const WHITE: [u8; 3] = [236, 236, 230];
const GREEN: [u8; 3] = [70, 224, 96];
const AMBER: [u8; 3] = [244, 172, 44];
const MAGENTA: [u8; 3] = [226, 80, 220];
const CYAN: [u8; 3] = [90, 214, 236];

/// A stand-in picture on a display, so the screens are not blank before the
/// avionics exist to draw them: an attitude and heading picture for a
/// flight display, or engine gauges and a map arc for a multifunction
/// display. It is my own simple drawing, a placeholder, and comes off when
/// the real displays are drawn. `width` and `height` are the screen's.
pub fn display_stand_in(scene: &mut Scene, surface: &Surface, u: f32, v: f32, width: f32, height: f32, proud: f32, flight: bool) {
    let lifted = Surface { origin: add(surface.origin, scaled(surface.normal(), proud)), right: surface.right, up: surface.up };
    let fill = |scene: &mut Scene, colour: [u8; 3], du: f32, dv: f32, w: f32, h: f32| scene.add("display graphics", colour, lifted.unit(u + du, v + dv, w, h, 0.0003));
    let text = |scene: &mut Scene, du: f32, dv: f32, size: f32, words: &str, colour: [u8; 3]| scene.labels.push(Label::on(&lifted, u + du, v + dv, size, 0.0006, words, colour, true));
    let ring = |scene: &mut Scene, colour: [u8; 3], du: f32, dv: f32, radius: f32, from: f32, to: f32| {
        let path: Vec<[f32; 3]> = (0..=48)
            .map(|i| {
                let angle = (from + (to - from) * i as f32 / 48.0).to_radians();
                add(lifted.point(u + du + radius * angle.sin(), v + dv + radius * angle.cos()), scaled(lifted.normal(), 0.0004))
            })
            .collect();
        scene.add("display graphics", colour, tube(&path, 0.0007, 6));
    };
    let (w, h) = (width, height);
    if flight {
        // Attitude: sky over ground, the horizon, and the aircraft symbol.
        let (top, band) = (h * 0.26, h * 0.40);
        fill(scene, SKY, 0.0, top + band * 0.25, w * 0.94, band * 0.5);
        fill(scene, GROUND, 0.0, top - band * 0.25, w * 0.94, band * 0.5);
        fill(scene, WHITE, 0.0, top, w * 0.94, 0.0008);
        for side in [-1.0, 1.0] {
            fill(scene, [250, 220, 60], side * w * 0.14, top, w * 0.12, 0.0022);
            // Speed tape on the left, altitude tape on the right.
            // Stands a little higher than the sky and ground it lies over.
            scene.add("display graphics", [44, 46, 54], lifted.unit(u + side * w * 0.40, v + top, w * 0.12, band * 0.86, 0.0005));
        }
        text(scene, -w * 0.40, top, 0.0052, "250", WHITE);
        text(scene, w * 0.40, top, 0.0052, "10000", WHITE);
        text(scene, 0.0, h * 0.475, 0.0042, "AP  FMS1  ALTS", GREEN);
        // Heading: the compass arc and the course line.
        ring(scene, WHITE, 0.0, -h * 0.30, w * 0.34, -100.0, 100.0);
        ring(scene, WHITE, 0.0, -h * 0.30, w * 0.20, 0.0, 360.0);
        fill(scene, MAGENTA, 0.0, -h * 0.22, 0.0012, h * 0.26);
        text(scene, 0.0, -h * 0.045, 0.0048, "279", WHITE);
        text(scene, -w * 0.36, -h * 0.10, 0.0036, "FMS1", MAGENTA);
        text(scene, -w * 0.34, -h * 0.44, 0.0032, "GS 254", WHITE);
        text(scene, w * 0.34, -h * 0.44, 0.0032, "TAT -5", WHITE);
    } else {
        // Engine gauges: three pairs of dials with their readings.
        for (row, name) in ["N1", "ITT", "N2"].iter().enumerate() {
            let dv = h * 0.36 - row as f32 * h * 0.115;
            for side in [-1.0, 1.0] {
                ring(scene, GREEN, -w * 0.24 + side * w * 0.11, dv, w * 0.075, -130.0, 100.0);
                text(scene, -w * 0.24 + side * w * 0.11, dv - 0.003, 0.0036, ["86.4", "700", "87.5"][row], GREEN);
            }
            text(scene, -w * 0.24, dv, 0.0030, name, WHITE);
        }
        for (line, (words, colour)) in [("OIL PRESS  73", GREEN), ("OIL TEMP   94", GREEN), ("FUEL QTY 11280", WHITE), ("GEAR   DN", GREEN), ("FLAPS  20", CYAN), ("TRIM   5.2", WHITE)].iter().enumerate() {
            text(scene, w * 0.22, h * 0.40 - line as f32 * h * 0.052, 0.0034, words, *colour);
        }
        fill(scene, [70, 72, 80], 0.0, h * 0.025, w * 0.92, 0.0007);
        // Map: range arc, the track line, and the aircraft.
        ring(scene, WHITE, 0.0, -h * 0.40, w * 0.36, -62.0, 62.0);
        ring(scene, WHITE, 0.0, -h * 0.40, w * 0.18, -62.0, 62.0);
        fill(scene, MAGENTA, 0.0, -h * 0.24, 0.0012, h * 0.30);
        text(scene, -w * 0.34, -h * 0.03, 0.0036, "FMS1", MAGENTA);
        text(scene, w * 0.32, -h * 0.03, 0.0032, "20 NM", CYAN);
        text(scene, 0.0, -h * 0.44, 0.0040, "279", WHITE);
        let _ = AMBER;
    }
}
