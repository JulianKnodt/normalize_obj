/// OFF file format parser
use super::Vec3;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Header,
    Counts,
    Verts,
    Faces,
    Done,
}

pub fn parse(f: &str) -> (Vec<Vec3>, Vec<Vec<usize>>) {
    let f = File::open(f).expect("Failed to open file");
    let buf_reader = BufReader::new(f);
    let mut verts = vec![];
    let mut faces = vec![];
    let mut curr = State::Header;

    let mut vert_count = 0;
    let mut face_count = 0;

    for l in buf_reader.lines() {
        let l = l.expect("Failed to read line");
        match &l.split_whitespace().collect::<Vec<_>>().as_slice() {
            [v, ..] if v.starts_with("#") => continue,
            [..] if curr == State::Done => {
                panic!("Unexpected line after reading all items {:?}", l);
            }

            [off] if off.to_lowercase() == "off" => {
                assert_eq!(curr, State::Header);
                curr = State::Counts;
            }
            [v, f, e] if curr == State::Counts || curr == State::Header => {
                vert_count = v.parse::<usize>().expect("Failed to parse vert count");
                face_count = f.parse::<usize>().expect("Failed to parse face count");
                e.parse::<usize>().expect("Failed to parse edge count");
                curr = State::Verts;
            }
            [x, y, z] if curr == State::Verts => {
                let x = x.parse::<f64>().expect("Failed to parse vert x");
                let y = y.parse::<f64>().expect("Failed to parse vert y");
                let z = z.parse::<f64>().expect("Failed to parse vert z");
                verts.push([x, y, z]);
                if verts.len() == vert_count {
                    curr = State::Faces
                }
            }

            [cnt, r @ ..] if curr == State::Faces => {
                let _cnt = cnt.parse::<u32>().expect("Failed to parse count");
                let v_idxs = r
                    .iter()
                    .map(|v_i| v_i.parse::<usize>().expect("Failed to parse vert idx"))
                    .collect::<Vec<_>>();
                faces.push(v_idxs);
                if faces.len() == face_count {
                    curr = State::Done
                }
            }
            _ => panic!("Unexpected line {:?}", l),
        }
    }
    (verts, faces)
}

pub fn to_obj(verts: Vec<Vec3>, faces: Vec<Vec<usize>>) -> impl Iterator<Item = String> {
    verts
        .into_iter()
        .map(|[x, y, z]| format!("v {x} {y} {z}"))
        .chain(faces.into_iter().map(|face| {
            let trailing = face
                .iter()
                .map(|vi| vi + 1) // 0 to 1 indexed
                .map(|vi| format!("{vi}// "))
                .collect::<String>();
            format!("f {}", trailing)
        }))
}
