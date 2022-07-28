#![feature(let_chains)]
use clap::{ArgEnum, Parser};

use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

mod off;

type Vec3 = [f64; 3];

/// Utility to normalize an OBJ mesh to the center, and fit it within to the unit box, or within
/// another mesh's bounding box.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Meshes to normalize.
    #[clap(short, long, value_parser)]
    src: String,
    /// How should the mesh be normalized? Either fixing the AABB, or the average of all the
    /// vertices.
    #[clap(short, long, value_parser, arg_enum, default_value_t = NormalizeKind::AABB)]
    method: NormalizeKind,
    /// Which mesh to normalize to, if any.
    #[clap(short, long, value_parser)]
    target: Option<String>,

    /// Output file name
    #[clap(short, long, value_parser)]
    output: Option<String>,

    /// Perform operation in place (can be specified in addition to outputting a file).
    #[clap(long)]
    in_place: bool,

    /// If target and src have same number of vertices, directly replace src's vertex position
    /// with target's.
    #[clap(long)]
    replace_positions: bool,
}

fn main() {
    let args = Args::parse();

    let mut out_files = vec![];
    if args.in_place {
        let out_file = File::create(&args.src).expect("Failed to replace original file");
        out_files.push(BufWriter::new(out_file));
    }
    if let Some(output) = args.output {
        let out_file = File::create(&output).expect("Failed to create output file");
        out_files.push(BufWriter::new(out_file));
    }
    if args.src.ends_with(".off") {
        println!("Converting .off file to obj without normalizing...");
        let (v, f) = off::parse(&args.src);
        let out = off::to_obj(v, f);
        for l in out {
            for f in out_files.iter_mut() {
                writeln!(f, "{}", l).expect("Write failed");
            }
        }
    } else {
        let out = normalize(
            &args.src,
            args.target.as_ref(),
            args.method,
            args.replace_positions,
        );
        for l in out {
            for f in out_files.iter_mut() {
                writeln!(f, "{}", l).expect("Write failed");
            }
        }
    };
}

fn read_vertices(file_name: &String) -> Vec<Vec3> {
    let file = File::open(file_name).expect("Failed to open file");
    let reader = BufReader::new(file);
    let mut points: Vec<Vec3> = vec![];
    for l in reader.lines() {
        let l = l.expect("Failed to read line");
        match l.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["v", x, y, z] => {
                points.push([x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap()]);
            }
            _ => continue,
        }
    }
    points
}

/// Normalize the given file to unit box or another file.
fn normalize(
    file_name: &str,
    to_match: Option<&String>,
    kind: NormalizeKind,
    replace_positions: bool,
) -> impl Iterator<Item = String> {
    let file = File::open(file_name).expect("Failed to open file");
    let to_match = to_match.map(read_vertices);
    let reader = BufReader::new(file);
    // keep track all lines read, so that can write out the file later
    let mut read_lines: Vec<Option<String>> = vec![];
    let mut points: Vec<Vec3> = vec![];

    for l in reader.lines() {
        let l = l.expect("Failed to read line");
        match l.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["v", x, y, z] => {
                points.push([x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap()]);
                read_lines.push(None);
            }
            ["v", ..] => panic!("Unhandled syntax: {:?}", l),
            _unknown => read_lines.push(Some(l)),
        }
    }
    let (center, scale) = kind.center_scale(&points);
    let map_to = to_match.as_ref().map(|tm| kind.center_scale(tm));
    for p in points.iter_mut() {
        *p = sub(*p, center);
        *p = div(*p, scale);
        if let Some((new_center, new_scale)) = map_to {
            *p = mul(*p, new_scale);
            *p = add(*p, new_center);
        }
    }
    if let Some(tm) = to_match && replace_positions {
        points.clone_from_slice(&tm);
    }
    let iter = read_lines
        .iter_mut()
        .filter(|l| l.is_none())
        .zip(points.into_iter());
    for (l, [x, y, z]) in iter {
        *l = Some(format!("v {:?} {:?} {:?}", x, y, z));
    }
    read_lines.into_iter().map(Option::unwrap)
}

fn avg(vec: &[Vec3]) -> Vec3 {
    let mut x_acc = 0.;
    let mut y_acc = 0.;
    let mut z_acc = 0.;
    let n = vec.len() as f64;
    for [x, y, z] in vec {
        x_acc += x / n;
        y_acc += y / n;
        z_acc += z / n;
    }
    [x_acc, y_acc, z_acc]
}

fn bounding_box(vs: &[Vec3]) -> (Vec3, Vec3) {
    let mut lx = f64::INFINITY;
    let mut ly = f64::INFINITY;
    let mut lz = f64::INFINITY;
    let mut hx = f64::NEG_INFINITY;
    let mut hy = f64::NEG_INFINITY;
    let mut hz = f64::NEG_INFINITY;
    for &[x, y, z] in vs {
        lx = lx.min(x);
        ly = ly.min(y);
        lz = lz.min(z);
        hx = hx.max(x);
        hy = hy.max(y);
        hz = hz.max(z);
    }
    ([lx, ly, lz], [hx, hy, hz])
}

fn add([a, b, c]: Vec3, [x, y, z]: Vec3) -> Vec3 {
    [a + x, b + y, c + z]
}

fn sub([a, b, c]: Vec3, [x, y, z]: Vec3) -> Vec3 {
    [a - x, b - y, c - z]
}

fn div([a, b, c]: Vec3, [x, y, z]: Vec3) -> Vec3 {
    [a / x, b / y, c / z]
}

fn mul([a, b, c]: Vec3, [x, y, z]: Vec3) -> Vec3 {
    [a * x, b * y, c * z]
}

fn norm([a, b, c]: &Vec3) -> f64 {
    (a * a + b * b + c * c).sqrt()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ArgEnum)]
enum NormalizeKind {
    AABB,
    Centroid,
}

impl NormalizeKind {
    fn center_scale(self, vs: &[Vec3]) -> (Vec3, Vec3) {
        match self {
            NormalizeKind::AABB => {
                let (ll, ur) = bounding_box(vs);
                let extent = sub(ur, ll);
                let half_extent = div(extent, [2., 2., 2.]);
                let center = add(ll, half_extent);
                (center, half_extent)
            }
            NormalizeKind::Centroid => {
                let center = avg(vs);
                let max_k = vs
                    .iter()
                    .map(|v| norm(&sub(*v, center)))
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(1.);
                (center, [max_k, max_k, max_k])
            }
        }
    }
}

impl fmt::Display for NormalizeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NormalizeKind::AABB => write!(f, "AABB"),
            NormalizeKind::Centroid => write!(f, "Centroid"),
        }
    }
}
