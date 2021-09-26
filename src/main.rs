use cgmath::{Basis2, Rad, Rotation, Rotation2, vec2};
use svg::{Document, node::element::{Circle, Group, Path, path::Data}};

mod util;

fn diamond(side_length: f64, short_angle: f64) -> Group {
    let straight = vec2(0.0, side_length);
    let angled = Basis2::from_angle(Rad(-short_angle))
        .rotate_vector(straight);
    
    let path_data = Data::new()
        .move_to((0, 0))
        .line_by((0, side_length))
        .line_by((angled.x, angled.y))
        .line_by((0, -side_length))
        .line_to((0, 0));

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", (1.0 / 96.0).to_string())
        .set("d", path_data);

    let points = [
        straight * 0.25, straight * 0.75,
        straight + angled * 0.25, straight + angled * 0.75,
        straight * 0.75 + angled, straight * 0.25 + angled,
        angled * 0.75, angled * 0.25
    ];

    let mut group = Group::new().add(path);
    for point in points {
        let circle = Circle::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("cx", point.x.to_string())
            .set("cy", point.y.to_string())
            .set("r", "0.1");
        group = group.add(circle);
    }

    group
}

fn main() {
    let connections = util::connections(8, util::equivalent_rotation_180);

    let doc = Document::new()
        .set("width", "10in")
        .set("height", "10in")
        .set("viewBox", (0, 0, 10, 10))
        .add(diamond(2.5, std::f64::consts::TAU / 10.0));

    svg::save("ignore/diamond.svg", &doc).unwrap();
}
