//! Where things are along the fuselage. Stations are in inches, as the
//! manufacturer's documents give them; everything else is in metres.

pub const METRES_PER_INCH: f32 = 0.0254;

/// Forward pressure bulkhead.
pub const FS_FORWARD_PRESSURE_BULKHEAD: f32 = 202.75;
/// Aft limit of crew seat travel on the outboard track (approximate).
pub const FS_SEAT_PRIMARY_STOP: f32 = 251.0;
/// Bulkhead at the back of the flight compartment.
pub const FS_FLIGHT_COMPARTMENT_BULKHEAD: f32 = 280.0;
/// Aft pressure bulkhead.
pub const FS_AFT_PRESSURE_BULKHEAD: f32 = 621.0;

/// Inside diameter of the constant fuselage section, inches.
pub const CABIN_INSIDE_DIAMETER_IN: f32 = 97.65;
/// Width of the floor in the constant section, inches.
pub const FLOOR_WIDTH_IN: f32 = 82.0;

/// Crew seat adjustment ranges.
pub const SEAT_HEIGHT_TRAVEL_IN: f32 = 6.5;
pub const SEAT_TRACK_STEP_IN: f32 = 1.0;
pub const SEAT_RECLINE_LIMIT_DEG: f32 = 30.0;

pub fn inches(value: f32) -> f32 {
    value * METRES_PER_INCH
}

/// The z coordinate of a fuselage station.
pub fn station_z(station: f32) -> f32 {
    inches(station)
}

/// Inside radius of the constant section, metres.
pub fn cabin_radius() -> f32 {
    inches(CABIN_INSIDE_DIAMETER_IN / 2.0)
}

/// Height of the floor relative to the centreline, metres (negative: below).
/// Follows from the floor being a chord of the inside circle.
pub fn floor_y() -> f32 {
    let radius = CABIN_INSIDE_DIAMETER_IN / 2.0;
    let half_width = FLOOR_WIDTH_IN / 2.0;
    -inches((radius * radius - half_width * half_width).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_floor_gives_the_published_cabin_height() {
        // Published stand-up height is 6 ft 1 in under the trim; the bare
        // shell must be a little taller than that, and not by much.
        let height_in = (cabin_radius() - floor_y()) / METRES_PER_INCH;
        assert!(height_in > 73.0 && height_in < 77.0, "{height_in}");
    }

    #[test]
    fn the_flight_compartment_is_about_two_metres_long() {
        let length = station_z(FS_FLIGHT_COMPARTMENT_BULKHEAD) - station_z(FS_FORWARD_PRESSURE_BULKHEAD);
        assert!((length - 1.962).abs() < 0.001, "{length}");
    }
}
