use cgmath::{Basis2, ElementWise, InnerSpace, Rad, Rotation, Rotation2, vec2};
use svg::{Document, node::element::{Group, Path, path::Data}};
use std::{f64::consts::TAU as TAU, io::Write, path::PathBuf, process::{Command, Stdio}};

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
    let acute_scale = if short_angle >= TAU * 0.15 { 0.35 } else { 0.15 };
    let obtuse_scale = 0.5;
    let inner_normals = [
        turn_90.rotate_vector(-straight.normalize()) * acute_scale,
        turn_90.rotate_vector(-straight.normalize()) * obtuse_scale,
        turn_90.rotate_vector(-angled.normalize()) * obtuse_scale,
        turn_90.rotate_vector(-angled.normalize()) * acute_scale,
        turn_90.rotate_vector(straight.normalize()) * acute_scale,
        turn_90.rotate_vector(straight.normalize()) * obtuse_scale,
        turn_90.rotate_vector(angled.normalize()) * obtuse_scale,
        turn_90.rotate_vector(angled.normalize()) * acute_scale,
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

#[derive(Clone, Copy, Debug)]
struct DiamondParameters {
    short_angle: f64,
    rows: usize,
    columns: usize,
    name: &'static str,
}

const THIN: DiamondParameters = DiamondParameters {
    short_angle: TAU * 0.1,
    rows: 3,
    columns: 5,
    name: "thin"
};

const FAT: DiamondParameters = DiamondParameters {
    short_angle: TAU * 0.2,
    rows: 4,
    columns: 3,
    name: "fat"
};

fn convert_svgs_to_pdfs(svgs: &[PathBuf]) -> Vec<PathBuf> {
    let pdfs = svgs.iter().map(|path| path.with_extension("pdf")).collect::<Vec<_>>();
    let shell = svgs.iter().zip(pdfs.iter()).map(|(svg, pdf)|
        format!("file-open:{}; export-filename:{}; export-do; ", svg.to_string_lossy(), pdf.to_string_lossy()))
    .collect::<String>();

    let echo_child = Command::new("echo")
        .arg(&shell)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start echo");

    let echo_stdout = echo_child.stdout.expect("Failed to get output");
    let mut child = Command::new("inkscape")
        .arg("--shell")
        .stdin(Stdio::from(echo_stdout))
        .spawn()
        .expect("Failed to start inkscape to convert svgs to pdfs");

    child.wait().expect("Inkscape failed");
    pdfs
}

fn combine_pdfs(pdfs: &[PathBuf], output: impl AsRef<std::path::Path>) {
    let mut command = &mut Command::new("pdfunite");
    for pdf in pdfs {
        command = command.arg(&*pdf.to_string_lossy());
    }
    let child = command.arg(&*output.as_ref().to_string_lossy())
        .spawn()
        .expect("Failed to start pdfunite")
        .wait()
        .expect("pdfunite failed");
}

fn main() {
    let connections = util::connections(8, util::equivalent_rotation_180);

    let mut filenames = vec![];

    for kind in [THIN, FAT] {
        let spacing = vec2(DIAMOND_SIDE * kind.short_angle.sin(), DIAMOND_SIDE);
        let offsets = (0..kind.columns).flat_map(|x| (0..kind.rows).map(move |y| 
            vec2(WIDTH, HEIGHT) / 2.0 + spacing
                .mul_element_wise(vec2(
                    x as f64 - kind.columns as f64 / 2.0 + 0.5, 
                    y as f64 - kind.rows as f64 / 2.0 + 0.5,
                ))))
            .collect::<Vec<_>>();

        let mut diamonds = connections.iter().map(|connection| {
            diamond_tile(DIAMOND_SIDE, kind.short_angle, connection)
        }).peekable();
        
        let mut num_docs = 0;
        while diamonds.peek().is_some() {
            let page_diamonds = std::iter::repeat_with(|| diamonds.next())
                .take(offsets.len())
                .flatten()
                .enumerate()
                .map(|(i, diamond)| {
                diamond.set("transform", format!("translate({}, {})", offsets[i].x, offsets[i].y))
            });

            let mut doc = new_page();
            for diamond in page_diamonds {
                doc = doc.add(diamond);
            }

            let filename = PathBuf::from(format!("output/diamond_{}_{}.svg", kind.name, num_docs));
            svg::save(&filename, &doc).unwrap();
            filenames.push(filename);
            num_docs += 1;
        }
    }
    
    let pdfs = convert_svgs_to_pdfs(&filenames);
    combine_pdfs(&pdfs, "output/diamond.pdf");
}
