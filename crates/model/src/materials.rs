//! Which finish each part of the model has.

use crate::texture::Finish;

pub fn finish_for(part: &str) -> Finish {
    match part {
        "wall" | "bulkheads" | "door" => Finish::Lining,
        "floor" => Finish::Carpet,
        "glazing" | "switchlight caps" | "instrument dials" => Finish::Glass,
        "guard windows" => Finish::Clear,
        "display graphics" => Finish::Lit,
        "displays" | "tablet screens" => Finish::Screen,
        "seat cushions" | "seat backs" | "headrests" => Finish::Sheepskin,
        "glareshield" | "glareshield lip" | "seat shells" | "armrests" | "seat belts" | "column boots" | "hand rests" | "control wheels" | "yoke caps" | "glareshield trim" => Finish::Leather,
        "window frames" | "window seals" | "storage bin lids" | "control column boots" | "switch boots" => Finish::Rubber,
        "lever handles" | "landing gear knob" => Finish::Ivory,
        "panel faces a" | "panel faces b" | "breaker panels" | "placards" | "lever slots" | "storage bins" => Finish::PanelFace,
        "instrument panel" | "side panels" | "pedestal" | "side consoles" | "overhead panel" | "thrust quadrant" | "flap quadrant" | "handle wells" | "footwell housings" | "breaker tiers" => Finish::Paint,
        "lever barrels" | "reverser levers" | "rudder pedals" | "pedal treads" | "yoke rings" | "toggle levers" | "switch nuts" | "fasteners" | "seat tracks" | "seat frames" | "lever arms" | "buckles" | "knob skirts" | "switchlight rims" | "breaker collars" | "air outlets" | "map lights"
        | "cup holders" | "scuff plates" | "chart clips" | "tablet arms" | "door handle" | "crash axe head" | "rudder pedal arms" | "hand holds" => Finish::Metal,
        _ => Finish::Plastic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_obvious_parts_get_the_obvious_finishes() {
        assert_eq!(finish_for("seat cushions"), Finish::Sheepskin);
        assert_eq!(finish_for("glazing"), Finish::Glass);
        assert_eq!(finish_for("lever handles"), Finish::Ivory);
        assert_eq!(finish_for("something nobody named"), Finish::Plastic);
    }
}
