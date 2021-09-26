use cgmath::{Basis2, ElementWise, InnerSpace, Rad, Rotation, Rotation2, vec2};
use svg::{Document, node::element::{Group, Path, path::Data}};
use std::f64::consts::TAU as TAU;

mod util;

fn diamond_tile(side_length: f64, short_angle: f64, connection: &[u32]) -> Group {
    let straight = vec2(0.0, side_length);
    let angled = Basis2::from_angle(Rad(-short_angle))
        .rotate_vector(straight);
    let top_left_offset = -(straight + angled) / 2.0;
    
    let path_data = Data::new()
        .move_to((top_left_offset.x, top_left_offset.y))
        .line_by((0, side_length))
        .line_by((angled.x, angled.y))
        .line_by((0, -side_length))
        .line_by((-angled.x, -angled.y));

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1.0 / 96.0)
        .set("d", path_data);

    let points = [
        top_left_offset + straight * 0.25,
        top_left_offset + straight * 0.75,
        top_left_offset + straight + angled * 0.25,
        top_left_offset + straight + angled * 0.75,
        top_left_offset + straight * 0.75 + angled,
        top_left_offset + straight * 0.25 + angled,
        top_left_offset + angled * 0.75,
        top_left_offset + angled * 0.25
    ];

    let turn_90 = Basis2::from_angle(Rad(TAU / 4.0));
    const ACUTE_SCALE: f64 = 0.15;
    const OBTUSE_SCALE: f64 = 0.5;
    let inner_normals = [
        turn_90.rotate_vector(-straight.normalize()) * ACUTE_SCALE,
        turn_90.rotate_vector(-straight.normalize()) * OBTUSE_SCALE,
        turn_90.rotate_vector(-angled.normalize()) * OBTUSE_SCALE,
        turn_90.rotate_vector(-angled.normalize()) * ACUTE_SCALE,
        turn_90.rotate_vector(straight.normalize()) * ACUTE_SCALE,
        turn_90.rotate_vector(straight.normalize()) * OBTUSE_SCALE,
        turn_90.rotate_vector(angled.normalize()) * OBTUSE_SCALE,
        turn_90.rotate_vector(angled.normalize()) * ACUTE_SCALE,
    ];

    let mut group = Group::new().add(path);
    for (i0, i1) in connection.iter().map(|i| *i as usize).enumerate() {
        let p0 = points[i0];
        let p3 = points[i1];
        let p1 = p0 + inner_normals[i0];
        let p2 = p3 + inner_normals[i1];
        let path_data = Data::new()
            .move_to(Into::<(f64, f64)>::into(p0))
            .cubic_curve_to((p1.x, p1.y, p2.x, p2.y, p3.x, p3.y));

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 0.1)
            .set("d", path_data);

        group = group.add(path);
    }

    group
}

const WIDTH: f64 = 8.5;
const HEIGHT: f64 = 11.0;
const DIAMOND_SIDE: f64 = 2.5;

fn new_page() -> Document {
    Document::new()
        .set("width", format!("{}in", WIDTH))
        .set("height", format!("{}in", HEIGHT))
        .set("viewBox", (0.0, 0.0, WIDTH, HEIGHT))
}

fn main() {
    let thin_spacing = vec2(DIAMOND_SIDE * (TAU * 0.15).cos(), DIAMOND_SIDE);
    let thin_offsets = (0..5).flat_map(|x| (0..3).map(move |y| 
        vec2(WIDTH, HEIGHT) / 2.0 + thin_spacing.mul_element_wise(vec2(x as f64 - 2.0, y as f64 - 1.0))))
        .collect::<Vec<_>>();

    let connections = util::connections(8, util::equivalent_rotation_180);

    let mut diamonds = connections.iter().map(|connection| {
        diamond_tile(DIAMOND_SIDE, TAU / 10.0, connection)
    }).peekable();
    
    let mut num_docs = 0;
    while diamonds.peek().is_some() {
        let page_diamonds = std::iter::repeat_with(|| diamonds.next())
            .take(thin_offsets.len())
            .flatten()
            .enumerate()
            .map(|(i, diamond)| {
            diamond.set("transform", format!("translate({}, {})", thin_offsets[i].x, thin_offsets[i].y))
        });

        let mut doc = new_page();
        for diamond in page_diamonds {
            doc = doc.add(diamond);
        }

        svg::save(format!("ignore/diamond_{}.svg", num_docs), &doc).unwrap();
        num_docs += 1;
    }
}
