//! The flight compartment.
//!
//! Documented, and used as given: the bulkhead stations, the constant
//! fuselage section and floor width, the eye being 0.48 m off the
//! centreline, the lines of sight over the nose and under the windshield
//! top, the face sizes of the Collins units, the console panel width, and
//! the seat stop station.
//!
//! Measured from the manufacturer's layout figures, scaled by the known
//! display and console panel sizes: where each display, panel, lever and
//! control sits on the instrument panel, glareshield, side panels,
//! pedestal, overhead panel and side consoles, and the sizes of the panels.
//!
//! Estimated, because no document we hold gives it: the station of the eye
//! and its height above the floor, the distance from eye to panel, the
//! panel's lean, and the shapes of the seats, control columns and pedals.

pub mod controls;
pub mod figure;
pub mod glareshield;
pub mod overhead;
pub mod pedestal;
pub mod shell;

use crate::hardware::{CONSOLE_PANEL_WIDTH, FCP};
use crate::mesh::{add, dot, normalised, scaled, sub, Vec3};
use crate::part::{block, pitched, Manipulator, Motion, Scene, Surface, UPRIGHT};
use crate::shapes::{cylinder, pillow, rounded_slab, smooth, swept, tube};
use crate::stations::{floor_y, station_z, FS_SEAT_PRIMARY_STOP};
use controls::Faces;
use shell::{Eye, FS_WINDSHIELD_TOP, VISION_DOWN_OVER_NOSE};

/// Eye, metres off the centreline. Documented.
pub const EYE_OFFSET: f32 = 0.48;
/// Fuselage station of the eye: documented as 2.72 m aft of the nose,
/// which the three-view drawing puts at station 141.
pub const EYE_STATION: f32 = 248.0;
/// Height of the eye above the floor: documented as 2.71 m above the
/// ground, where the drawing puts the fuselage centreline at 2.19 m, so
/// 0.52 m above the centreline.
pub const EYE_ABOVE_FLOOR: f32 = 1.19;
/// Distance from the eye forward to the middle of the instrument panel.
/// Estimated.
const EYE_TO_PANEL: f32 = 0.74;
/// Lean of the instrument panel, top towards the nose, degrees. Estimated.
const PANEL_LEAN: f32 = 15.0;

// Measured from the layout figures.
const PANEL_WIDTH: f32 = 1.19;
const PANEL_HEIGHT: f32 = 0.377;
const PFD_OFFSET: f32 = 0.452;
const MFD_OFFSET: f32 = 0.192;
const SIDE_CONSOLE_LENGTH: f32 = 0.99;
const SIDE_PANEL_WIDTH: f32 = 0.40;

const PANEL: [u8; 3] = [98, 106, 118];
const SEAT_LEATHER: [u8; 3] = [222, 216, 200];
const YOKE: [u8; 3] = [50, 52, 56];
const BARREL: [u8; 3] = [158, 160, 164];
const PEDESTAL: [u8; 3] = [126, 134, 144];
const QUADRANT: [u8; 3] = [112, 115, 120];
const GLARE: [u8; 3] = [38, 39, 43];
const OVERHEAD: [u8; 3] = [62, 68, 78];
const BELT: [u8; 3] = [44, 52, 66];
const CREAM: [u8; 3] = [198, 198, 194];
const CONSOLE: [u8; 3] = [156, 158, 160];
const CUSHION: [u8; 3] = [124, 118, 124];
const FRAME: [u8; 3] = [58, 66, 80];
const CONTROL: [u8; 3] = [18, 18, 20];
const LEVER: [u8; 3] = [150, 152, 156];
const HANDLE: [u8; 3] = [226, 221, 206];
const RED: [u8; 3] = [176, 40, 36];
const AMBER: [u8; 3] = [214, 160, 40];
const VENT: [u8; 3] = [150, 152, 158];
const DARK: [u8; 3] = [14, 14, 17];
const SCREEN: [u8; 3] = [6, 9, 18];

const X: Vec3 = [1.0, 0.0, 0.0];
const Y: Vec3 = [0.0, 1.0, 0.0];
const Z: Vec3 = [0.0, 0.0, 1.0];

/// The eye on one side: -1 for the left seat, 1 for the right.
pub fn eye(side: f32) -> Vec3 {
    [side * EYE_OFFSET, floor_y() + EYE_ABOVE_FLOOR, station_z(EYE_STATION)]
}

pub fn shell_eye() -> Eye {
    Eye { offset: EYE_OFFSET, y: floor_y() + EYE_ABOVE_FLOOR, station: EYE_STATION }
}

fn axes(surface: &Surface) -> [Vec3; 3] {
    [surface.right, surface.up, surface.normal()]
}

/// A flat rounded piece standing `proud` of a surface.
fn plate(surface: &Surface, u: f32, v: f32, width: f32, height: f32, proud: f32, radius: f32) -> crate::mesh::Mesh {
    let middle = add(surface.point(u, v), scaled(surface.normal(), proud / 2.0));
    rounded_slab(middle, axes(surface), [width / 2.0, height / 2.0, proud / 2.0], radius, 3)
}

/// Rows of circuit breakers, as the manufacturer's panel figures show
/// them: each row has a letter and a number of positions. Rows start at
/// `left` and run right, or end at `from_right` if that is given. Every
/// breaker is a control that pops out.
fn breaker_rows(scene: &mut Scene, surface: &Surface, panel: &str, left: f32, top: f32, rows: &[(char, usize)], from_right: Option<f32>) {
    const PITCH: f32 = 0.019;
    const ROW: f32 = 0.031;
    for (index, (letter, count)) in rows.iter().enumerate() {
        let v = top - index as f32 * ROW;
        let start = from_right.map(|right| right - (*count as f32 - 1.0) * PITCH).unwrap_or(left);
        scene.legend(surface, start - 0.016, v, 0.006, 0.0045, &letter.to_string());
        for position in 0..*count {
            let foot = add(surface.point(start + position as f32 * PITCH, v), scaled(surface.normal(), 0.004));
            scene.add("breaker collars", [150, 152, 158], cylinder(foot, surface.normal(), 0.0068, 0.003, 32));
            scene.control(&format!("cl650/breakers/{panel}_{}{:02}", letter.to_ascii_lowercase(), position + 1));
            let pop = Motion::Slide { along: surface.normal(), metres: (0.0, 0.006) };
            scene.add_moving("breakers", CONTROL, cylinder(foot, surface.normal(), 0.0046, 0.009, 32), foot, pop, Manipulator::Toggle);
            // The white band that shows when a breaker has popped.
            scene.add_moving("breaker bands", [230, 230, 224], cylinder(add(foot, scaled(surface.normal(), 0.0005)), surface.normal(), 0.0048, 0.0018, 32), foot, pop, Manipulator::Toggle);
        }
    }
    scene.naming = None;
}

pub fn build() -> Scene {
    let mut scene = Scene::default();
    let [_, eye_y, eye_z] = eye(1.0);
    let floor = floor_y();
    let around = shell_eye();
    shell::build(&mut scene, around);
    let column = CONSOLE_PANEL_WIDTH;

    // Glareshield: its top edge is what the eye looks over, so the
    // documented line of sight over the nose fixes its height.
    let lean = pitched(PANEL_LEAN);
    let shelf_depth = 0.19;
    let face_width = glareshield::FACE.0 * glareshield::SCALE;
    let face_height = glareshield::FACE.1 * glareshield::SCALE;
    let panel_z = eye_z - EYE_TO_PANEL;
    let lean_run = PANEL_HEIGHT / 2.0 * PANEL_LEAN.to_radians().sin();
    let shelf_back = panel_z - lean_run + shelf_depth;
    let shelf_top = eye_y - (eye_z - shelf_back) * VISION_DOWN_OVER_NOSE.to_radians().tan();
    let panel_top_y = shelf_top - face_height;
    let panel = Surface::new([0.0, panel_top_y - PANEL_HEIGHT / 2.0 * PANEL_LEAN.to_radians().cos(), panel_z], X, lean[1]);

    scene.add("instrument panel", PANEL, panel.slab(0.0, 0.0, PANEL_WIDTH, PANEL_HEIGHT, 0.03));
    for side in [-1.0, 1.0] {
        controls::display(&mut scene, &panel, side * PFD_OFFSET, 0.007, true);
        controls::display(&mut scene, &panel, side * MFD_OFFSET, 0.007, false);
    }
    for side in [-1.0, 1.0] {
        scene.add("hand rests", [36, 31, 28], plate(&panel, side * 0.41, -PANEL_HEIGHT / 2.0 + 0.011, 0.36, 0.020, 0.028, 0.009));
    }
    // The moulded housings each side of the pedestal, from the bottom of
    // the panel to the floor.
    let panel_bottom = panel.point(0.0, -PANEL_HEIGHT / 2.0);
    for side in [-1.0, 1.0] {
        let middle = [side * 0.325, (floor + panel_bottom[1]) / 2.0 - 0.01, panel_bottom[2] + 0.07];
        scene.add("footwell housings", CONSOLE, rounded_slab(middle, [X, Z, Y], [0.085, 0.085, (panel_bottom[1] - floor) / 2.0 - 0.012], 0.03, 8));
    }
    // Between the inboard displays: standby instrument, clock, and the
    // ground wing anti-ice panel with its three switchlights.
    scene.add("instrument bezels", [50, 56, 64], plate(&panel, 0.0, 0.113, 0.080, 0.081, 0.008, 0.006));
    scene.add("displays", SCREEN, plate(&panel, 0.0, 0.117, 0.058, 0.058, 0.0086, 0.003));
    controls::knob(&mut scene, &panel, 0.030, 0.080, 0.005);
    scene.add("instrument bezels", [50, 56, 64], plate(&panel, 0.0, 0.020, 0.082, 0.085, 0.008, 0.012));
    scene.add("displays", SCREEN, plate(&panel, 0.0, 0.022, 0.060, 0.050, 0.0086, 0.004));
    {
        let mut faces = Faces { scene: &mut scene, count: 0 };
        faces.add(&panel, 0.0, -0.057, 0.122, 0.045);
    }
    for (u, name) in [(-0.030, "heat"), (0.0, "temp"), (0.030, "off")] {
        scene.control(&format!("cl650/panel/gnd_wing_anti_ice_{name}"));
        controls::switchlight(&mut scene, &panel, u, -0.057);
    }
    scene.naming = None;
    scene.legend(&panel, 0.0, -0.040, 0.0022, 0.0024, "SUPP GND WING ANTI-ICE");

    // The shelf, rounded at its corners, with a padded lip along the edge
    // the crew look over.
    scene.add(
        "glareshield",
        GLARE,
        rounded_slab(
            [0.0, shelf_top - face_height / 2.0, shelf_back - shelf_depth / 2.0 - 0.01],
            [X, Z, Y],
            [face_width / 2.0 + 0.03, shelf_depth / 2.0 + 0.01, face_height / 2.0],
            0.06,
            6,
        ),
    );
    let lip_y = shelf_top - 0.011;
    scene.add("glareshield lip", GLARE, tube(&[[-face_width / 2.0 + 0.03, lip_y, shelf_back], [0.0, lip_y, shelf_back], [face_width / 2.0 - 0.03, lip_y, shelf_back]], 0.011, 12));
    // The line of stitching along the lip, and the pale trim strip that
    // runs under the face of the glareshield.
    let stitch_y = shelf_top - 0.004;
    scene.add("glareshield stitching", [226, 224, 216], tube(&[[-face_width / 2.0 + 0.03, stitch_y, shelf_back + 0.0105], [0.0, stitch_y, shelf_back + 0.0105], [face_width / 2.0 - 0.03, stitch_y, shelf_back + 0.0105]], 0.0011, 8));
    scene.add("glareshield trim", [214, 212, 204], rounded_slab([0.0, panel_top_y - 0.007, shelf_back - 0.035], [X, Z, Y], [face_width / 2.0 + 0.03, 0.035, 0.007], 0.006, 4));
    let shelf_face = Surface::new([0.0, shelf_top - face_height / 2.0 - 0.004, shelf_back], X, Y);
    {
        let scale = glareshield::SCALE;
        let mut faces = Faces { scene: &mut scene, count: 0 };
        faces.add(&shelf_face, 0.0, 0.0, FCP.width, FCP.height + 0.008);
        for side in [-1.0, 1.0] {
            faces.add(&shelf_face, side * 449.0 * scale, 0.0, 365.0 * scale, FCP.height + 0.008);
        }
        glareshield::CONTROLS.add(&mut scene, &shelf_face, glareshield::FIGURE, 0.0, 0.0, "cl650/glareshield");
        for (middle, width) in [(0.0, FCP.width), (-449.0 * scale, 365.0 * scale), (449.0 * scale, 365.0 * scale)] {
            let (half_width, half_height) = (width / 2.0, (FCP.height + 0.008) / 2.0);
            for (du, dv, w, h) in [(0.0, half_height, width, 0.0032), (0.0, -half_height, width, 0.0032), (-half_width, 0.0, 0.0032, 2.0 * half_height), (half_width, 0.0, 0.0032, 2.0 * half_height)] {
                scene.add("unit bezels", [24, 25, 28], plate(&shelf_face, middle + du, dv, w, h, 0.0078, 0.001));
            }
        }
        // Legends on the keys and over the knobs. The names are the usual
        // ones for this flight control panel; which key carries which has
        // not been checked against the aircraft.
        let key_names = ["FD", "VS", "NAV", "HDG", "APPR", "ALT", "VNAV", "B/C", "FD", "FLC", "1/2 BNK", "YD", "TURB", "AP ENG", "XFR"];
        for ((x, y, _, _), name) in glareshield::CONTROLS.keys.iter().zip(key_names) {
            let (u, v) = glareshield::FIGURE.offset(*x, *y);
            scene.placard(&shelf_face, u, v, 0.0022, 0.0052, name, [24, 24, 26]);
        }
        let knob_names = ["CRS 1", "SPEED", "HDG", "ALT", "CRS 2", "BARO", "BARO"];
        for ((x, y, radius), name) in glareshield::CONTROLS.knobs.iter().zip(knob_names) {
            let (u, v) = glareshield::FIGURE.offset(*x, *y);
            scene.legend(&shelf_face, u, v + radius * glareshield::SCALE * 1.25 + 0.003, 0.002, 0.0024, name);
        }
        let at = |x: f32, y: f32| glareshield::FIGURE.offset(x, y);
        let (u, v) = at(glareshield::WHEEL.0, glareshield::WHEEL.1);
        scene.add("knobs", [32, 32, 36], cylinder(add(shelf_face.point(u - 0.004, v), scaled(shelf_face.normal(), 0.002)), X, 0.011, 0.008, 20));
        for (x, y) in glareshield::MASTER_WARNING {
            let (u, v) = at(x, y);
            scene.add("master warning", RED, plate(&shelf_face, u, v, 0.020, 0.011, 0.007, 0.002));
        }
        for (x, y) in glareshield::MASTER_CAUTION {
            let (u, v) = at(x, y);
            scene.add("master caution", AMBER, plate(&shelf_face, u, v, 0.020, 0.011, 0.007, 0.002));
        }
        for (x, y, width, height) in glareshield::ANNUNCIATORS {
            let (u, v) = at(x, y);
            scene.add("displays", SCREEN, plate(&shelf_face, u, v, width * scale, height * scale, 0.004, 0.002));
            // The autothrottle annunciator: its message, and its name under it.
            scene.labels.push(crate::labels::Label::on(&shelf_face, u, v + 0.002, 0.0046, 0.0044, "FAIL", AMBER, true));
            scene.legend(&shelf_face, u, v - height * scale / 2.0 - 0.004, 0.0022, 0.0024, "ATS");
        }
        for (x, y) in glareshield::MASTER_WARNING {
            let (u, v) = at(x, y);
            scene.legend(&shelf_face, u, v + 0.012, 0.0018, 0.0024, "MASTER");
        }
    }

    // Side panels: the instrument panel turns towards each pilot outboard
    // of the flight display. Audio control panel over the stall protection
    // panel, with the air outlet and airspeed placard outboard of them.
    for side in [-1.0, 1.0] {
        let (sin, cos) = 40.0_f32.to_radians().sin_cos();
        let right: Vec3 = [cos, 0.0, side * sin];
        let up = normalised(sub(lean[1], scaled(right, dot(lean[1], right))));
        let hinge = panel.point(side * PANEL_WIDTH / 2.0, 0.0);
        let wing = Surface::new(add(hinge, scaled(right, side * SIDE_PANEL_WIDTH / 2.0)), right, up);
        scene.add("side panels", PANEL, wing.slab(0.0, 0.0, SIDE_PANEL_WIDTH, 0.30, 0.02));
        let scale = glareshield::SIDE_SCALE;
        // Surface coordinate of a point a distance outboard of the hinge.
        let from_hinge = |distance: f32| side * (distance * scale - SIDE_PANEL_WIDTH / 2.0);
        let box_u = from_hinge(glareshield::SIDE_BOX_FROM_HINGE);
        let edges = glareshield::SIDE_BOX_EDGES;
        let v_of = |y: f32| (glareshield::SIDE_MIDDLE.1 - y) * scale;
        {
            let mut faces = Faces { scene: &mut scene, count: 0 };
            faces.add(&wing, box_u, v_of((edges[0] + edges[1]) / 2.0), column, (edges[1] - edges[0]) * scale);
            faces.add(&wing, box_u, v_of((edges[1] + edges[2]) / 2.0), column, (edges[2] - edges[1]) * scale);
        }
        glareshield::AUDIO.add(&mut scene, &wing, glareshield::SIDE_FIGURE, box_u, 0.0, if side < 0.0 { "cl650/audio/left" } else { "cl650/audio/right" });
        let toggle_dx = (glareshield::STALL_TOGGLE.0 - glareshield::SIDE_MIDDLE.0) * scale;
        let gauge_dx = (glareshield::STALL_GAUGE.0 - glareshield::SIDE_MIDDLE.0) * scale;
        scene.control(if side < 0.0 { "cl650/stall/pusher_left" } else { "cl650/stall/pusher_right" });
        controls::toggle(&mut scene, &wing, box_u - side * toggle_dx, v_of(glareshield::STALL_TOGGLE.1));
        scene.naming = None;
        scene.legend(&wing, box_u, v_of(edges[0] + 9.0), 0.0024, 0.0024, "AUDIO");
        scene.legend(&wing, box_u - side * toggle_dx, v_of(edges[1] + 12.0), 0.0022, 0.0024, "STALL PROT");
        scene.legend(&wing, box_u - side * toggle_dx, v_of(glareshield::STALL_TOGGLE.1 - 18.0), 0.002, 0.0024, "PUSHER");
        controls::gauge(&mut scene, &wing, box_u - side * gauge_dx, v_of(glareshield::STALL_GAUGE.1), glareshield::STALL_GAUGE.2 * scale);
        let (gasper_out, gasper_down, gasper_radius) = glareshield::GASPER;
        let gasper = wing.point(from_hinge(gasper_out), -gasper_down * scale);
        scene.add("air outlets", VENT, cylinder(gasper, wing.normal(), gasper_radius * scale, 0.010, 28));
        scene.add("air outlet nozzles", DARK, cylinder(gasper, wing.normal(), gasper_radius * scale * 0.55, 0.016, 20));
        let (placard_out, placard_up, placard_width, placard_height) = glareshield::PLACARD;
        scene.add("placards", DARK, wing.unit(from_hinge(placard_out), placard_up * scale, placard_width * scale, placard_height * scale, 0.0015));
    }

    // Pedestal: three console columns wide. A sloping part under the panel
    // carries the control display units; the rest is level between the seats.
    let pedestal_width = 3.0 * column + 0.017;
    let slope_length = 0.19;
    let slope_angle = 35.0_f32.to_radians();
    let slope_top = panel.point(0.0, -PANEL_HEIGHT / 2.0);
    let slope_bottom = add(slope_top, [0.0, -slope_length * slope_angle.sin(), slope_length * slope_angle.cos()]);
    let slope = Surface::new(scaled(add(slope_top, slope_bottom), 0.5), X, sub(slope_top, slope_bottom));
    scene.add("pedestal", PEDESTAL, slope.slab(0.0, 0.0, pedestal_width, slope_length, 0.20));
    let pedestal_top = slope_bottom[1];
    let level_length = pedestal::LENGTH * pedestal::SCALE - slope_length;
    let level_middle = slope_bottom[2] + level_length / 2.0;
    // The level part is in two: the thrust lever quadrant sits in a well,
    // lower than the panels behind it, with a wall along each side, as
    // photographs of the real pedestal show.
    let well_depth = 0.018;
    let well_end = pedestal::WELL_END * pedestal::SCALE;
    let well_length = well_end - slope_length;
    let level_front = slope_bottom[2];
    let body = |scene: &mut Scene, from: f32, to: f32, top: f32| {
        scene.add("pedestal", PEDESTAL, rounded_slab([0.0, (floor + top) / 2.0, level_front + (from + to) / 2.0], [X, Z, Y], [pedestal_width / 2.0, (to - from) / 2.0, (top - floor) / 2.0], 0.012, 3));
    };
    body(&mut scene, 0.0, well_length, pedestal_top - well_depth);
    body(&mut scene, well_length, level_length, pedestal_top);
    for side in [-1.0, 1.0] {
        let wall = [side * (pedestal_width / 2.0 - 0.004), pedestal_top - well_depth / 2.0, level_front + well_length / 2.0];
        scene.add("pedestal", PEDESTAL, rounded_slab(wall, [X, Z, Y], [0.004, well_length / 2.0, well_depth / 2.0 + 0.001], 0.002, 2));
    }
    // On the level surfaces, "up" for the crew is towards the nose.
    let level = Surface::new([0.0, pedestal_top, level_middle], X, [0.0, 0.0, -1.0]);
    let well = Surface::new([0.0, pedestal_top - well_depth, level_middle], X, [0.0, 0.0, -1.0]);
    let on_pedestal = pedestal::Pedestal { slope: &slope, slope_length, well: &well, well_end, level: &level, level_length };
    on_pedestal.build(&mut scene);
    for (x, y, text) in pedestal::TITLES {
        let (u, along) = pedestal::position(x, y);
        let (surface, v) = on_pedestal.at(along);
        scene.legend(surface, u, v, 0.0021, 0.0024, text);
    }
    for ((x, y), id) in pedestal::DISPLAY_UNITS.iter().zip(["1", "2", "3"]) {
        let (u, along) = pedestal::position(*x, *y);
        let (surface, v) = on_pedestal.at(along);
        controls::control_display_unit(&mut scene, surface, u, v, id);
    }
    // Thrust lever quadrant, as it is in photographs of the real flight
    // deck: a raised hump the two thrust levers come up through, each a
    // metal arm with a barrel handle and a reverser lever ahead of it; the
    // flight spoiler lever in its own slot to the left; the flap lever in a
    // raised housing to the right; and a red handle in a well each side.
    {
        let on = |x: f32, y: f32| {
            let (u, along) = pedestal::position(x, y);
            let (surface, v) = on_pedestal.at(along);
            surface.point(u, v)
        };
        let hump = on(534.0, 610.0);
        scene.add("thrust quadrant", QUADRANT, rounded_slab(add(hump, [0.0, 0.004, 0.0]), [Z, Y, X], [0.095, 0.022, 0.060], 0.02, 8));
        for side in [-1.0, 1.0] {
            scene.add("lever slots", DARK, rounded_slab(add(hump, [side * 0.026, 0.0265, 0.0]), [Z, X, Y], [0.075, 0.006, 0.0006], 0.005, 4));
            let stem = pitched(-18.0);
            let foot = add(hump, [side * 0.026, 0.02, 0.01]);
            let top = add(foot, scaled(stem[1], 0.105));
            let swing = Motion::Rotate { axis: X, degrees: (-22.0, 22.0) };
            scene.control(if side < 0.0 { "cl650/quadrant/thrust_lever_left" } else { "cl650/quadrant/thrust_lever_right" });
            scene.add_moving("lever arms", LEVER, block(add(foot, scaled(stem[1], 0.0525)), stem, [0.004, 0.0525, 0.011]), foot, swing, Manipulator::Drag);
            let barrel_start = if side < 0.0 { -0.058 } else { 0.002 };
            scene.add_moving("lever barrels", BARREL, cylinder(add(top, [barrel_start - side * 0.026, 0.0, 0.0]), X, 0.021, 0.056, 64), foot, swing, Manipulator::Drag);
            // Reverser lever, riding on the front of the arm.
            let ahead = add(foot, add(scaled(stem[1], 0.07), scaled(stem[2], -0.03)));
            scene.add_moving("reverser levers", BARREL, rounded_slab(ahead, stem, [0.014, 0.024, 0.012], 0.005, 6), foot, swing, Manipulator::Drag);
            scene.add_moving("lever arms", LEVER, block(add(foot, add(scaled(stem[1], 0.045), scaled(stem[2], -0.02))), stem, [0.003, 0.03, 0.006]), foot, swing, Manipulator::Drag);
            scene.naming = None;
            scene.add("red handles", RED, rounded_slab(add(hump, [side * 0.02, 0.012, 0.093]), UPRIGHT, [0.012, 0.012, 0.008], 0.003, 3));
        }

        // Flight spoiler lever: a tall thin blade in a long slot.
        let slot = on(383.0, 600.0);
        scene.add("lever slots", DARK, rounded_slab(add(slot, [0.0, 0.0015, 0.0]), [Z, X, Y], [0.086, 0.007, 0.0015], 0.006, 4));
        let blade = pitched(-10.0);
        let foot = on(376.0, 540.0);
        scene.control("cl650/quadrant/flight_spoiler_lever");
        scene.add_moving("lever handles", HANDLE, rounded_slab(add(foot, scaled(blade[1], 0.06)), blade, [0.006, 0.06, 0.014], 0.005, 4), foot, Motion::Rotate { axis: X, degrees: (-20.0, 20.0) }, Manipulator::Drag);
        scene.naming = None;

        // Flap lever: a raised curved housing with the handle riding on it.
        let housing = on(838.0, 680.0);
        scene.add("flap quadrant", QUADRANT, rounded_slab(add(housing, [0.0, 0.012, 0.0]), [Z, Y, X], [0.066, 0.036, 0.022], 0.034, 10));
        scene.add("lever slots", DARK, rounded_slab(add(housing, [0.0, 0.0485, 0.0]), [Z, X, Y], [0.035, 0.005, 0.0006], 0.004, 3));
        let travel = Motion::Slide { along: Z, metres: (-0.02, 0.035) };
        scene.control("cl650/quadrant/flap_lever");
        scene.add_moving("lever arms", LEVER, block(add(housing, [0.0, 0.058, -0.012]), UPRIGHT, [0.004, 0.012, 0.008]), housing, travel, Manipulator::Drag);
        scene.add_moving("flap handle", [66, 68, 72], rounded_slab(add(housing, [0.0, 0.078, -0.012]), [X, Z, Y], [0.026, 0.017, 0.010], 0.007, 8), housing, travel, Manipulator::Drag);
        scene.naming = None;
        let handle_top = Surface::new(add(housing, [0.0, 0.088, -0.012]), X, [0.0, 0.0, -1.0]);
        scene.placard(&handle_top, 0.0, 0.0, 0.0075, 0.0003, "FLAPS", [236, 236, 230]);
        for (detent, text) in [(-0.045, "0"), (-0.015, "20"), (0.015, "30"), (0.045, "45")] {
            let beside = Surface::new(add(housing, [0.034, -0.0, detent]), X, [0.0, 0.0, -1.0]);
            scene.legend(&beside, 0.0, 0.0, 0.004, 0.0024, text);
        }

        // The red handle in a well each side: turn-and-pull on the left,
        // roll disconnect on the right.
        for (x, y, across) in [(268.0, 490.0, false), (760.0, 507.0, true)] {
            let well = on(x, y);
            scene.add("handle wells", [96, 108, 122], rounded_slab(add(well, [0.0, 0.001, 0.0]), [X, Z, Y], [0.034, 0.040, 0.001], 0.03, 8));
            scene.add("lever arms", LEVER, cylinder(well, Y, 0.007, 0.028, 14));
            let grip = if across { [0.030, 0.010, 0.010] } else { [0.010, 0.010, 0.030] };
            scene.add("red handles", RED, pillow(add(well, [0.0, 0.036, 0.0]), UPRIGHT, grip, 0.8, 0.0, 12, 24, 9));
        }
        // Landing gear manual release and the air driven generator handle, on the base strip.
        for (x, y, width, height) in &pedestal::RED_HANDLES[4..] {
            let (u, along) = pedestal::position(*x, *y);
            let (surface, v) = on_pedestal.at(along);
            scene.add("red handles", RED, plate(surface, u, v, width * pedestal::SCALE, height * pedestal::SCALE, 0.012, 0.006));
        }
    }
    {
        // Aileron trim switch, landing gear lever with its wheel-shaped
        // knob, and the parking brake handle.
        let place = |(x, y): (f32, f32)| {
            let (u, along) = pedestal::position(x, y);
            let (surface, v) = on_pedestal.at(along);
            (*surface, u, v)
        };
        let (x, y, width, height) = pedestal::AILERON_TRIM;
        let (surface, u, v) = place((x, y));
        scene.add("trim switches", CONTROL, plate(&surface, u, v, width * pedestal::SCALE, height * pedestal::SCALE, 0.012, 0.003));
        let (surface, u, v) = place(pedestal::GEAR_LEVER);
        let foot = surface.point(u, v);
        let top = add(foot, scaled(surface.normal(), 0.05));
        let throw = Motion::Rotate { axis: X, degrees: (-18.0, 18.0) };
        scene.control("cl650/gear/lever");
        scene.add_moving("landing gear lever", LEVER, tube(&[foot, top], 0.005, 10), foot, throw, Manipulator::Toggle);
        scene.add_moving("landing gear knob", HANDLE, cylinder(add(top, [-0.009, 0.0, 0.0]), X, 0.019, 0.018, 28), foot, throw, Manipulator::Toggle);
        scene.naming = None;
        let (surface, u, v) = place(pedestal::PARKING_BRAKE);
        scene.add("parking brake", CONTROL, plate(&surface, u, v, 0.075, 0.03, 0.004, 0.012));
        scene.add("parking brake", CONTROL, cylinder(surface.point(u, v), surface.normal(), 0.016, 0.03, 18));
    }

    // Overhead panel: hung under the roof behind the windshield top, three
    // columns of system panels with a strip of lighting panels in front.
    let front_z = station_z(FS_WINDSHIELD_TOP) + 0.02;
    let housing_length = 0.36;
    let back_z = front_z + 0.31;
    let drop = 0.075;
    let front_point = [0.0, around.roof_y(0.0, front_z) - drop, front_z];
    let back_point = [0.0, around.roof_y(0.0, back_z) - drop, back_z];
    let along_roof = normalised(sub(back_point, front_point));
    let overhead = Surface::new(add(front_point, scaled(along_roof, housing_length / 2.0)), X, along_roof);
    scene.add("overhead panel", OVERHEAD, rounded_slab(add(overhead.origin, scaled(overhead.normal(), -0.015)), axes(&overhead), [0.295, housing_length / 2.0, 0.015], 0.03, 5));
    {
        let mut faces = Faces { scene: &mut scene, count: 0 };
        let strip_v = -housing_length / 2.0 + 0.008 + 0.022;
        for column_index in [-1.0, 0.0, 1.0] {
            faces.add(&overhead, column_index * column, strip_v, column, 0.044);
        }
        let top = housing_length / 2.0 - 0.006;
        faces.stack(&overhead, -column, top, column, &overhead::LEFT_PANELS.map(overhead::metres));
        faces.stack(&overhead, 0.0, top, column, &overhead::CENTRE_PANELS.map(overhead::metres));
        faces.stack(&overhead, column, top, column, &overhead::RIGHT_PANELS.map(overhead::metres));
        overhead::controls(&mut scene, &overhead, top, strip_v);
    }
    for side in [-1.0, 1.0] {
        let vent = overhead.point(side * 0.262, 0.0);
        scene.add("air outlets", VENT, cylinder(vent, overhead.normal(), 0.024, 0.010, 24));
        scene.add("air outlet nozzles", DARK, cylinder(vent, overhead.normal(), 0.013, 0.016, 18));
        let light = overhead.point(side * 0.262, -0.10);
        scene.add("map lights", VENT, cylinder(light, overhead.normal(), 0.027, 0.008, 24));
        scene.add("map light lenses", [236, 236, 228], cylinder(light, overhead.normal(), 0.019, 0.010, 20));
    }
    // Standby compass, hanging in front of the lighting strip.
    let compass = add(front_point, [0.0, -0.045, -0.035]);
    scene.add("standby compass", CONTROL, cylinder(compass, Z, 0.032, 0.05, 24));
    scene.add("standby compass card", [226, 226, 220], cylinder(add(compass, [0.0, 0.0, 0.05]), Z, 0.026, 0.001, 24));
    scene.add("standby compass", CONTROL, tube(&[add(compass, [0.0, 0.03, 0.025]), add(compass, [0.0, 0.05, 0.03])], 0.008, 8));

    // Hand holds along the sides of the overhead panel.
    for side in [-1.0, 1.0] {
        let at = |v: f32, off: f32| add(overhead.point(side * 0.325, v), scaled(overhead.normal(), off));
        scene.add("hand holds", FRAME, tube(&smooth(&[at(-0.11, 0.0), at(-0.09, 0.03), at(0.09, 0.03), at(0.11, 0.0)], 5), 0.009, 10));
    }

    // Aft bulkhead and its door.
    let bulkhead_z = station_z(crate::stations::FS_FLIGHT_COMPARTMENT_BULKHEAD) - 0.001;
    let bulkhead = Surface::new([0.0, floor + 0.90, bulkhead_z], [-1.0, 0.0, 0.0], Y);
    scene.add("door", CREAM, plate(&bulkhead, 0.0, -0.02, 0.56, 1.74, 0.012, 0.04));
    scene.add("door handle", VENT, tube(&[add(bulkhead.point(0.21, 0.02), scaled(bulkhead.normal(), 0.012)), add(bulkhead.point(0.21, 0.02), scaled(bulkhead.normal(), 0.045)), add(bulkhead.point(0.13, 0.02), scaled(bulkhead.normal(), 0.045))], 0.008, 10));
    // Circuit breaker panels 1 and 2, on the wall behind each seat. Each has
    // a main block (row A over rows B to H) and a second block (rows J to
    // Q) whose upper rows are shorter, as in the panel figures. The figures
    // show some positions empty; here every position has a breaker.
    for (panel, title, middle, mirrored) in [("cbp1", "CBP-1", 0.63, false), ("cbp2", "CBP-2", -0.63, true)] {
        let top = 0.28 + 0.108;
        scene.add("breaker panels", [30, 32, 38], plate(&bulkhead, middle, 0.28, 0.56, 0.30, 0.004, 0.012));
        scene.legend(&bulkhead, middle, top + 0.026, 0.007, 0.0045, title);
        let main: [(char, usize); 8] = [('A', 13), ('B', 14), ('C', 14), ('D', 14), ('E', 14), ('F', 14), ('G', 14), ('H', 14)];
        let second: [(char, usize); 7] = [('J', 4), ('K', 7), ('L', 11), ('M', 13), ('N', 13), ('P', 13), ('Q', 13)];
        let edge = middle - 0.25;
        if mirrored {
            breaker_rows(&mut scene, &bulkhead, panel, 0.0, top - 0.031, &second, Some(edge + 0.016 + 12.0 * 0.019));
            breaker_rows(&mut scene, &bulkhead, panel, edge + 0.016 + 12.0 * 0.019 + 0.035, top, &main, None);
        } else {
            breaker_rows(&mut scene, &bulkhead, panel, edge + 0.016, top, &main, None);
            breaker_rows(&mut scene, &bulkhead, panel, edge + 0.016 + 13.0 * 0.019 + 0.035, top - 0.031, &second, None);
        }
    }

    // Documented safety equipment: the fire extinguisher behind the
    // copilot seat and the crash axe on the bulkhead behind the pilot seat.
    let extinguisher = [0.66, floor + 0.02, bulkhead_z - 0.10];
    scene.add("fire extinguisher", RED, cylinder(extinguisher, Y, 0.045, 0.30, 20));
    scene.add("fire extinguisher head", CONTROL, cylinder(add(extinguisher, [0.0, 0.30, 0.0]), Y, 0.02, 0.05, 12));
    let axe = bulkhead.point(0.60, -0.42);
    scene.add("crash axe", [178, 150, 96], tube(&[add(axe, [0.0, -0.18, -0.02]), add(axe, [0.0, 0.18, -0.02])], 0.012, 8));
    scene.add("crash axe head", VENT, block(add(axe, [0.03, 0.16, -0.02]), UPRIGHT, [0.06, 0.035, 0.008]));

    for side in [-1.0, 1.0] {
        let x = side * EYE_OFFSET;

        // Side console along the wall, starting beside the panel.
        let console_top = pedestal_top;
        let console_front = panel_z + 0.12;
        let console_middle = console_front + SIDE_CONSOLE_LENGTH / 2.0;
        let (inner, outer) = (0.75, 0.96);
        scene.add(
            "side consoles",
            CONSOLE,
            rounded_slab([side * (inner + outer) / 2.0, console_top - 0.11, console_middle], [X, Z, Y], [(outer - inner) / 2.0, SIDE_CONSOLE_LENGTH / 2.0, 0.11], 0.025, 4),
        );
        let top = Surface::new([side * (inner + outer) / 2.0, console_top, console_middle], X, [0.0, 0.0, -1.0]);
        let back = |along: f32| SIDE_CONSOLE_LENGTH / 2.0 - along;
        {
            let mut faces = Faces { scene: &mut scene, count: 0 };
            faces.add(&top, 0.0, back(0.20), 0.15, 0.10);
            faces.add(&top, 0.0, back(0.38), 0.15, 0.165);
        }
        // Circuit breaker panel 3 (left) or 4 (right): its three rows of
        // thirteen sit on three sloping tiers that step up towards the wall
        // beside the pilot's knees, as photographs show, between stations 220
        // and 235, where the training guide puts them.
        let (breaker_panel, breaker_title) = if side < 0.0 { ("cbp3", "CBP-3") } else { ("cbp4", "CBP-4") };
        for (tier, letter) in ['A', 'B', 'C'].into_iter().enumerate() {
            let step = tier as f32;
            let slope = Surface::new([side * (0.60 + step * 0.055), console_top - 0.12 + step * 0.055, station_z(227.5)], [0.0, 0.0, side], [side * 0.707, 0.707, 0.0]);
            scene.add("breaker tiers", CONSOLE, slope.slab(0.0, 0.0, 0.32, 0.062, 0.05));
            scene.add("breaker panels", [30, 32, 38], plate(&slope, 0.0, 0.0, 0.30, 0.046, 0.004, 0.006));
            breaker_rows(&mut scene, &slope, breaker_panel, -0.114, 0.0, &[(letter, 13)], None);
            if tier == 2 {
                scene.legend(&slope, 0.0, 0.018, 0.004, 0.0045, breaker_title);
            }
        }

        // Tablet on an arm above the front of the console, turned towards the pilot.
        {
            let (sin, cos) = 50.0_f32.to_radians().sin_cos();
            let across: Vec3 = [cos, 0.0, side * sin];
            let tilt = pitched(12.0)[1];
            let upward = normalised(sub(tilt, scaled(across, dot(tilt, across))));
            let tablet = Surface::new([side * 0.84, console_top + 0.20, console_front + 0.20], across, upward);
            scene.add("tablets", [26, 27, 30], plate(&tablet, 0.0, 0.0, 0.250, 0.175, 0.009, 0.014));
            scene.add("tablet screens", SCREEN, plate(&tablet, 0.0, 0.0, 0.222, 0.148, 0.0094, 0.004));
            scene.add("tablets", [26, 27, 30], cylinder(add(tablet.point(side * 0.115, 0.0), scaled(tablet.normal(), 0.009)), tablet.normal(), 0.004, 0.0006, 12));
            let behind = add(tablet.origin, scaled(tablet.normal(), -0.004));
            let arm = [[side * 0.86, console_top, console_front + 0.16], [side * 0.86, console_top + 0.10, console_front + 0.16], add(behind, scaled(tablet.normal(), -0.035)), behind];
            scene.add("tablet arms", VENT, tube(&smooth(&arm, 6), 0.009, 12));
            scene.add("tablet arms", VENT, cylinder([side * 0.86, console_top, console_front + 0.16], Y, 0.022, 0.008, 20));
            scene.add("tablet arms", VENT, plate(&Surface::new(behind, across, upward), 0.0, 0.0, 0.09, 0.07, 0.004, 0.01));
        }

        // Oxygen mask box panel: two switchlights, then a guarded switch.
        let which = if side < 0.0 { "left" } else { "right" };
        scene.control(&format!("cl650/console_{which}/light_1"));
        controls::switchlight(&mut scene, &top, -0.02, back(0.34));
        scene.control(&format!("cl650/console_{which}/light_2"));
        controls::switchlight(&mut scene, &top, 0.02, back(0.34));
        scene.control(&format!("cl650/console_{which}/switch_1"));
        controls::toggle(&mut scene, &top, -0.03, back(0.42));
        scene.naming = None;
        scene.add("red handles", RED, plate(&top, 0.025, back(0.42), 0.022, 0.03, 0.012, 0.004));
        // Cup holder.
        let cup = top.point(0.0, back(0.495));
        scene.add("cup holders", VENT, cylinder(cup, Y, 0.035, 0.006, 24));
        scene.add("cup holder wells", DARK, cylinder(cup, Y, 0.030, 0.007, 24));
        if side < 0.0 {
            // Nose wheel steering tiller, left console only: a hand wheel on a short shaft.
            let foot = top.point(0.0, back(0.09));
            scene.add("tiller", CONTROL, cylinder(foot, Y, 0.012, 0.035, 12));
            scene.add("tiller", CONTROL, cylinder(add(foot, [0.0, 0.035, 0.0]), Y, 0.05, 0.014, 28));
            scene.add("tiller", CONTROL, tube(&[add(foot, [0.0, 0.049, 0.03]), add(foot, [0.0, 0.075, 0.03])], 0.01, 10));
        } else {
            let mut faces = Faces { scene: &mut scene, count: 1 };
            faces.add(&top, 0.0, back(0.09), 0.15, 0.10);
        }

        // Seat. The eye is 0.79 m above the cushion.
        let cushion_top = eye_y - 0.79;
        let back_lean = pitched(-12.0);
        let flat = [X, Z, Y];
        // Sheepskin-covered cushions: soft, bulging, lumpy.
        scene.add("seat cushions", CUSHION, pillow([x, cushion_top - 0.045, eye_z + 0.01], flat, [0.235, 0.14, 0.055], 0.45, 0.0035, 220, 440, 1));
        for pad in [-1.0, 1.0] {
            scene.add("seat cushions", CUSHION, pillow([x + pad * 0.14, cushion_top - 0.050, eye_z - 0.225], flat, [0.092, 0.105, 0.05], 0.5, 0.003, 110, 220, 5));
        }
        scene.add("seat backs", CUSHION, pillow([x, cushion_top + 0.21, eye_z + 0.165], back_lean, [0.235, 0.20, 0.05], 0.45, 0.0035, 190, 380, 2));
        scene.add("seat backs", CUSHION, pillow([x, cushion_top + 0.52, eye_z + 0.230], back_lean, [0.195, 0.14, 0.048], 0.5, 0.0035, 170, 340, 6));
        scene.add("headrests", CUSHION, pillow([x, cushion_top + 0.74, eye_z + 0.275], back_lean, [0.13, 0.09, 0.045], 0.55, 0.003, 160, 320, 3));
        // Harness: two shoulder straps down the back, a lap belt, and the buckle.
        let towards_pilot = scaled(back_lean[2], -1.0);
        for strap in [-1.0, 1.0] {
            let middle = add([x + strap * 0.085, cushion_top + 0.36, eye_z + 0.19], scaled(towards_pilot, 0.062));
            scene.add("seat belts", BELT, rounded_slab(middle, back_lean, [0.022, 0.27, 0.0018], 0.004, 2));
        }
        scene.add("seat belts", BELT, rounded_slab([x, cushion_top + 0.021, eye_z + 0.05], [X, Z, Y], [0.225, 0.022, 0.0018], 0.004, 2));
        scene.add("buckles", VENT, cylinder([x, cushion_top + 0.022, eye_z + 0.05], Y, 0.028, 0.012, 24));
        // Shell behind the back, pan under the cushion, and the pedestal they sit on.
        scene.add("seat shells", SEAT_LEATHER, rounded_slab([x, cushion_top + 0.31, eye_z + 0.235], back_lean, [0.24, 0.33, 0.012], 0.09, 6));
        scene.add("seat shells", SEAT_LEATHER, rounded_slab([x, cushion_top - 0.095, eye_z - 0.08], flat, [0.24, 0.24, 0.015], 0.07, 6));
        let base_top = cushion_top - 0.11;
        for (leg_x, leg_z) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let foot = [x + leg_x * 0.17, floor + 0.02, eye_z - 0.04 + leg_z * 0.17];
            let top = [x + leg_x * 0.14, base_top, eye_z - 0.04 + leg_z * 0.10];
            scene.add("seat frames", FRAME, tube(&[foot, top], 0.014, 16));
        }
        for leg_z in [-1.0, 1.0] {
            let across = eye_z - 0.04 + leg_z * 0.135;
            scene.add("seat frames", FRAME, tube(&[[x - 0.155, (floor + base_top) / 2.0, across], [x + 0.155, (floor + base_top) / 2.0, across]], 0.011, 12));
        }
        scene.add("seat frames", FRAME, rounded_slab([x, floor + 0.035, eye_z - 0.05], flat, [0.19, 0.19, 0.015], 0.03, 3));
        for arm in [-1.0, 1.0] {
            scene.add("armrests", FRAME, pillow([x + arm * 0.245, cushion_top + 0.22, eye_z - 0.03], [Z, X, Y], [0.16, 0.022, 0.02], 0.6, 0.0, 32, 64, 4));
            scene.add("armrests", FRAME, tube(&[[x + arm * 0.245, cushion_top + 0.20, eye_z + 0.10], [x + arm * 0.245, cushion_top - 0.02, eye_z + 0.12]], 0.012, 8));
            // Seat tracks, with the stop on the outboard one.
            let (track_front, track_back) = (station_z(EYE_STATION - 22.0), station_z(EYE_STATION + 14.0));
            let track_x = x + arm * 0.17;
            scene.add("seat tracks", FRAME, block([track_x, floor + 0.01, (track_front + track_back) / 2.0], UPRIGHT, [0.015, 0.01, (track_back - track_front) / 2.0]));
            if arm == side {
                scene.add("seat stops", RED, block([track_x, floor + 0.035, station_z(FS_SEAT_PRIMARY_STOP)], UPRIGHT, [0.02, 0.015, 0.02]));
            }
        }

        // Control column from the floor, with the wheel towards the pilot.
        let column_z = eye_z - 0.52;
        let wheel_y = eye_y - 0.43;
        let wheel_z = column_z + 0.11;
        // The column: a flat post up from the floor with a clamp partway up,
        // and a neck that carries the wheel back towards the pilot.
        let post_top = wheel_y - 0.06;
        scene.add("control columns", CONTROL, rounded_slab([x, (floor + post_top) / 2.0, column_z], UPRIGHT, [0.036, (post_top - floor) / 2.0, 0.012], 0.006, 4));
        scene.add("control columns", CONTROL, rounded_slab([x, floor + 0.30, column_z + 0.016], UPRIGHT, [0.045, 0.045, 0.018], 0.01, 4));
        scene.add("control columns", CONTROL, cylinder([x, floor + 0.30, column_z + 0.034], Z, 0.022, 0.012, 32));
        scene.add("control columns", CONTROL, tube(&smooth(&[[x, post_top - 0.02, column_z], [x, wheel_y - 0.02, column_z + 0.03], [x, wheel_y + 0.01, wheel_z - 0.02]], 8), 0.024, 40));
        scene.add("control column boots", FRAME, cylinder([x, floor, column_z], Y, 0.06, 0.05, 20));
        // The control wheel, modelled from photographs of the real one: a
        // drum-shaped hub facing the pilot with a bright ring round its cap,
        // and an arm each side that leaves the hub low, sweeps out and down,
        // then turns up into a grip that thickens to a rounded top.
        let hub = [x, wheel_y + 0.01, wheel_z];
        let roll = Motion::Rotate { axis: Z, degrees: (-70.0, 70.0) };
        scene.control(if side < 0.0 { "cl650/controls/yoke_roll_left" } else { "cl650/controls/yoke_roll_right" });
        let wheel = |scene: &mut Scene, name: &'static str, colour: [u8; 3], mesh: crate::mesh::Mesh| scene.add_moving(name, colour, mesh, hub, roll, Manipulator::Drag);
        wheel(&mut scene, "control wheels", YOKE, cylinder(add(hub, [0.0, 0.0, -0.022]), Z, 0.055, 0.056, 72));
        wheel(&mut scene, "yoke rings", VENT, cylinder(add(hub, [0.0, 0.0, 0.034]), Z, 0.0505, 0.004, 72));
        wheel(&mut scene, "yoke caps", [44, 45, 49], cylinder(add(hub, [0.0, 0.0, 0.038]), Z, 0.043, 0.004, 72));
        let stitching: Vec<Vec3> = (0..=96)
            .map(|i| {
                let angle = std::f32::consts::TAU * i as f32 / 96.0;
                [hub[0] + 0.0528 * angle.cos(), hub[1] + 0.0528 * angle.sin(), wheel_z + 0.0345]
            })
            .collect();
        wheel(&mut scene, "yoke stitching", [214, 212, 204], tube(&stitching, 0.0009, 6));
        for horn in [-1.0, 1.0] {
            // Along the arm: how far out and up from the hub, then the half
            // width across the arm and the half depth towards the pilot.
            let sections: [(f32, f32, f32, f32); 9] = [
                (0.040, -0.030, 0.026, 0.022),
                (0.085, -0.052, 0.023, 0.019),
                (0.130, -0.052, 0.021, 0.018),
                (0.165, -0.025, 0.020, 0.018),
                (0.178, 0.020, 0.019, 0.020),
                (0.178, 0.065, 0.020, 0.022),
                (0.174, 0.100, 0.022, 0.024),
                (0.172, 0.118, 0.018, 0.020),
                (0.171, 0.127, 0.008, 0.009),
            ];
            let path: Vec<Vec3> = sections.iter().map(|s| [hub[0] + horn * s.0, hub[1] + s.1, wheel_z]).collect();
            let radii: Vec<(f32, f32)> = sections.iter().map(|s| (s.2, s.3)).collect();
            wheel(&mut scene, "control wheels", YOKE, swept(&path, &radii, Z, 40, 6));
        }
        // On the outboard grip: the trim switch pod and the red disconnect button.
        wheel(&mut scene, "yoke switches", [62, 64, 68], cylinder([x + side * 0.172, hub[1] + 0.104, wheel_z + 0.021], Z, 0.013, 0.008, 28));
        wheel(&mut scene, "yoke switches", [62, 64, 68], rounded_slab([x + side * 0.172, hub[1] + 0.104, wheel_z + 0.031], UPRIGHT, [0.004, 0.008, 0.003], 0.002, 3));
        wheel(&mut scene, "red handles", RED, cylinder([x + side * 0.181, hub[1] + 0.072, wheel_z + 0.020], Z, 0.006, 0.006, 20));
        scene.naming = None;
        // Scuff plate on the floor under the pilot's heels.
        scene.add("scuff plates", [104, 106, 112], rounded_slab([x, floor + 0.003, eye_z - 0.72], [X, Z, Y], [0.20, 0.13, 0.003], 0.02, 2));

        // Rudder pedals: a pad on an arm that comes up from the floor.
        for pedal in [-1.0, 1.0] {
            let pad = [x + pedal * 0.11, floor + 0.20, eye_z - 0.92];
            let tilt = pitched(25.0);
            scene.add("rudder pedals", [170, 172, 176], rounded_slab(pad, tilt, [0.056, 0.078, 0.007], 0.02, 8));
            for tread in 0..5 {
                let up = (tread as f32 - 2.0) * 0.028;
                let middle = add(pad, add(scaled(tilt[1], up), scaled(tilt[2], 0.0085)));
                scene.add("pedal treads", [128, 130, 134], rounded_slab(middle, tilt, [0.046, 0.007, 0.0015], 0.0015, 3));
            }
            scene.add("rudder pedal arms", FRAME, tube(&[[pad[0], floor + 0.01, pad[2] + 0.05], [pad[0], floor + 0.10, pad[2] + 0.03], [pad[0], pad[1], pad[2] + 0.012]], 0.012, 8));
        }
    }
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::AFD;

    const SHELL: [&str; 6] = ["wall", "glazing", "window frames", "window seals", "floor", "bulkheads"];

    #[test]
    fn every_fitting_is_inside_the_compartment() {
        let scene = build();
        let around = shell_eye();
        for part in scene.everything().into_iter().filter(|p| !SHELL.contains(&p.name)) {
            for v in &part.mesh.vertices {
                assert!(around.inside(v.position), "{} pokes out at {:?}", part.name, v.position);
            }
        }
    }

    #[test]
    fn every_face_points_the_way_its_normal_does() {
        for part in build().everything() {
            assert!(part.mesh.faces_agree_with_normals(), "{}", part.name);
        }
    }

    #[test]
    fn the_documented_units_and_measured_controls_are_all_there() {
        let scene = build();
        let triangles = |name: &str| scene.part(name).unwrap().mesh.triangle_count();
        let one_bezel = rounded_slab([0.0; 3], UPRIGHT, [AFD.width / 2.0, AFD.height / 2.0, 0.006], 0.008, 4).triangle_count();
        assert_eq!(triangles("display bezels"), 4 * one_bezel);
        // Toggle switches: 31 overhead, 25 pedestal, one on each audio
        // panel, each stall panel and each side console.
        assert_eq!(triangles("switch nuts"), (31 + 25 + 2 + 2 + 2) * cylinder([0.0; 3], X, 0.0055, 0.003, 6).triangle_count());
        // Switchlights: 41 overhead, 14 pedestal, 8 glareshield, 3 anti-ice,
        // 2 on each side console; plus the round buttons, 2 overhead and 4 pedestal.
        let one_rim = rounded_slab([0.0; 3], UPRIGHT, [0.0098, 0.0098, 0.0015], 0.002, 2).triangle_count();
        let button_rim = cylinder([0.0; 3], X, 0.0085, 0.002, 16).triangle_count();
        assert_eq!(triangles("switchlight rims"), (41 + 14 + 8 + 3 + 4) * one_rim + 6 * button_rim);
    }

    #[test]
    fn the_displays_fit_on_the_panel_without_touching() {
        assert!(PFD_OFFSET - MFD_OFFSET > AFD.width, "flight and multifunction displays overlap");
        assert!(2.0 * MFD_OFFSET - AFD.width > 0.122, "no room for the centre instruments");
        assert!(PFD_OFFSET + AFD.width / 2.0 < PANEL_WIDTH / 2.0, "flight display off the panel edge");
        assert!(AFD.height < PANEL_HEIGHT);
    }

    #[test]
    fn the_eye_looks_over_the_glareshield_at_the_documented_angle() {
        let scene = build();
        let [_, y, z] = eye(-1.0);
        let (_, high) = scene.part("glareshield").unwrap().mesh.bounds().unwrap();
        let angle = ((y - high[1]) / (z - high[2])).atan().to_degrees();
        assert!((angle - VISION_DOWN_OVER_NOSE).abs() < 0.2, "{angle}");
        let (_, lip) = scene.part("glareshield lip").unwrap().mesh.bounds().unwrap();
        assert!(lip[1] <= high[1] + 1e-4, "the lip must not stand above the line of sight");
        assert!(shell_eye().roof_y(EYE_OFFSET, z) - y > 0.25, "headroom");
    }

    #[test]
    fn the_seats_clear_the_pedestal_and_the_consoles() {
        let scene = build();
        let bounds = |name: &str| scene.part(name).unwrap().mesh.bounds().unwrap();
        let (pedestal_low, pedestal_high) = bounds("pedestal");
        let (_, armrest_high) = bounds("armrests");
        let inner_edge = EYE_OFFSET - 0.23;
        assert!(pedestal_high[0] < inner_edge && pedestal_low[0] > -inner_edge);
        assert!(armrest_high[0] < 0.75, "armrest into the side console");
    }

    #[test]
    fn every_moving_control_has_its_own_dataref_and_nothing_is_left_named() {
        let scene = build();
        assert!(scene.naming.is_none());
        let mut names: Vec<&str> = scene.moving.iter().map(|m| m.dataref.as_str()).collect();
        let count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count, "two controls share a dataref");
        assert!(count > 800, "{count} controls");
        for expected in ["cl650/overhead/switch_batt_master", "cl650/quadrant/thrust_lever_left", "cl650/gear/lever", "cl650/breakers/cbp1_a01", "cl650/breakers/cbp4_c13", "cl650/cdu_1/key_a_20"] {
            assert!(names.contains(&expected), "{expected}");
        }
    }

    #[test]
    fn the_breaker_panels_have_the_rows_the_figures_show() {
        let scene = build();
        let on = |panel: &str| scene.moving.iter().filter(|m| m.dataref.starts_with(&format!("cl650/breakers/{panel}_"))).count();
        // Panels 3 and 4: three rows of thirteen. Panels 1 and 2: row A of
        // thirteen, seven rows of fourteen, then rows of 4, 7, 11 and four of 13.
        assert_eq!((on("cbp3"), on("cbp4")), (39, 39));
        assert_eq!((on("cbp1"), on("cbp2")), (185, 185));
    }

    #[test]
    fn the_model_has_about_two_million_triangles() {
        let count = build().merged().triangle_count();
        assert!(count >= 1_900_000, "{count}");
        assert!(count <= 2_100_000, "{count} is over the budget");
    }

    #[test]
    fn panels_carry_their_legends() {
        let scene = build();
        let has = |text: &str| scene.labels.iter().any(|l| l.text == text);
        for text in ["ELECTRICAL POWER", "BATT MASTER", "FUEL", "ANTI-ICE", "FLAPS", "YAW DAMPER", "CBP-1", "EXEC", "PITCH DISC", "FIREX MONITOR", "BRG SRC", "UPR MENU", "AFCS SEL"] {
            assert!(has(text), "{text}");
        }
        assert!(scene.labels.len() > 350, "{}", scene.labels.len());
    }

    #[test]
    fn breaker_panels_three_and_four_are_between_their_documented_stations() {
        let scene = build();
        for control in scene.moving.iter().filter(|m| m.dataref.starts_with("cl650/breakers/cbp3_") || m.dataref.starts_with("cl650/breakers/cbp4_")) {
            let station = control.pivot[2] / crate::stations::METRES_PER_INCH;
            assert!((220.0..=235.0).contains(&station), "{} at station {station}", control.dataref);
        }
    }
}
