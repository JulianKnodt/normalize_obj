use std::{
  fs::File,
  io::{BufRead, BufReader, BufWriter, Write},
};

fn main() {
  let iter = std::env::args().skip(1).filter(|it| it.ends_with(".obj"));
  for obj_file in iter {
    let out = normalize(&obj_file).expect("Failed to normalize OBJ file");
    let out_file = File::create("normed_".to_owned() + &obj_file).expect("Failed to create file");
    let mut dst = BufWriter::new(out_file);
    for l in out {
      write!(dst, "{}\n", l).expect("Write failed");
    }
  }
}

fn normalize(file_name: &str) -> Result<impl Iterator<Item = String>, &'static str> {
  let file = File::open(file_name).map_err(|_err| "Failed to open file")?;
  let reader = BufReader::new(file);
  let mut read_lines: Vec<Option<String>> = vec![];
  let mut points: Vec<(f64, f64, f64)> = vec![];

  for l in reader.lines() {
    let l = l.map_err(|_err| "Failed to read line")?;
    if Some("v") != l.trim_start().split_whitespace().next() {
      read_lines.push(Some(l));
      continue;
    }
    read_lines.push(None);
    match l.trim().split_whitespace().collect::<Vec<_>>().as_slice() {
      ["v", x, y, z] => {
        points.push((x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap()));
      },
      v => panic!("Unexpected matched sequence {:?}", v),
    }
  }
  let average = avg(&points);
  for p in points.iter_mut() {
    *p = sub(*p, average);
  }
  let max_norm = points
    .iter()
    .map(|p| norm(p))
    .max_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap();
  println!("Shifted by {:?}, scaled by {:?}", average, max_norm);
  assert!(max_norm.is_finite());
  for p in points.iter_mut() {
    *p = kdiv(*p, max_norm);
  }
  let iter = read_lines
    .iter_mut()
    .filter(|l| l.is_none())
    .zip(points.drain(..));
  for (l, (x, y, z)) in iter {
    *l = Some(String::from(format!("v {:?} {:?} {:?}", x, y, z)));
  }
  Ok(read_lines.into_iter().map(Option::unwrap))
}

fn avg(vec: &[(f64, f64, f64)]) -> (f64, f64, f64) {
  let mut x_acc = 0.;
  let mut y_acc = 0.;
  let mut z_acc = 0.;
  let n = vec.len() as f64;
  for (x, y, z) in vec {
    x_acc += x / n;
    y_acc += y / n;
    z_acc += z / n;
  }
  (x_acc, y_acc, z_acc)
}

fn sub((a, b, c): (f64, f64, f64), (x, y, z): (f64, f64, f64)) -> (f64, f64, f64) {
  (a - x, b - y, c - z)
}

fn norm((a, b, c): &(f64, f64, f64)) -> f64 { (a * a + b * b + c * c).sqrt() }

fn kdiv((a, b, c): (f64, f64, f64), k: f64) -> (f64, f64, f64) { (a / k, b / k, c / k) }
