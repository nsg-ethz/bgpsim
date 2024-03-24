// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

//! Build script to generate outline data for all topology zoo topologies.

use std::{
    env,
    f64::consts::PI,
    fs::{create_dir_all, remove_file, OpenOptions},
    io::{BufWriter, Write},
    iter::repeat,
    path::PathBuf,
};

use geojson::{Feature, GeoJson, Value};
use geoutils::Location;
use mapproj::{cylindrical::mer::Mer, LonLat, Projection};

const RESOLUTION: f64 = 200.0;
const LOD: [f64; 3] = [0.5 * PI, 0.1 * PI, 0.0];
const GRID_ROWS: usize = 5;
const GRID_COLS: usize = 20;

fn main() {
    println!("cargo:rerun-if-changed=maps/countries.geojson");

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut path = PathBuf::from(out_dir);
    path.push("maps");
    path.push("mapping");
    create_dir_all(&path).unwrap();

    let countries_geojson = include_str!("maps/countries.geojson");
    let lines = read_geojson(countries_geojson.parse::<GeoJson>().unwrap());

    let p = Mer::new();
    let proj = |l: &Vec<Location>| project(l, &p);
    let lines: Vec<Line> = lines.iter().map(proj).map(Line::new).collect();

    println!("{} lines", lines.len());
    let mut global_bbox = lines.first().unwrap().bbox;
    lines.iter().for_each(|l| {
        global_bbox &= l.bbox;
    });

    let chunks: Vec<Chunk> = (0..GRID_ROWS)
        .flat_map(|row| (0..GRID_COLS).zip(repeat(row)))
        .filter_map(|(col, row)| Chunk::from_lines(&lines, global_bbox, (col, row)))
        .collect();

    println!(
        "{} chunks with {} lines",
        chunks.len(),
        chunks.iter().map(|c| c.lines.len()).sum::<usize>()
    );
    let _ = lines;

    // store each file
    for (i, chunk) in chunks.iter().enumerate() {
        for (lod, scale) in LOD.iter().enumerate() {
            let min_dist = scale / RESOLUTION;
            let chunk = chunk.compress(min_dist);
            path.push(format!("{i}:{lod}.cbor"));
            if path.exists() {
                remove_file(&path).unwrap();
            }
            let f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            ciborium::into_writer(&chunk, f).unwrap();
            path.pop();
        }
    }

    // store the index
    path.pop();
    path.push("index.rs");
    if path.exists() {
        remove_file(&path).unwrap();
    }
    let mut f = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap(),
    );
    writeln!(&mut f, "const LOD: [f64; {}] = {:?};", LOD.len(), LOD).unwrap();
    writeln!(&mut f, "const INDEX: [Bbox; {}] = [", chunks.len()).unwrap();
    for chunk in chunks {
        let min = chunk.bbox.min;
        let max = chunk.bbox.max;
        writeln!(
            &mut f,
            "    Bbox {{ min: Point {{ x: {}f64, y: {}f64 }}, max: Point {{ x: {}f64, y: {}f64 }} }},",
            min.x, min.y, max.x, max.y
        )
        .unwrap()
    }
    write!(&mut f, "];").unwrap();
}

fn project<P: Projection>(points: &[Location], proj: &P) -> Vec<Point> {
    points
        .iter()
        .map(|p| {
            let xy = proj.proj_lonlat(&rad(*p)).unwrap();
            Point {
                x: xy.x(),
                y: -xy.y(),
            }
        })
        .collect()
}

fn rad(x: Location) -> LonLat {
    let mut lon = x.longitude();
    let mut lat = x.latitude();
    if lon < 0.0 {
        lon += 360.0;
    }
    lon = lon * std::f64::consts::PI / 180.0;
    lat = lat * std::f64::consts::PI / 180.0;
    LonLat::new(lon, lat)
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Bbox {
    min: Point,
    max: Point,
}

impl std::fmt::Debug for Bbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bbox")
            .field("x", &format!("{:.3}..{:.3}", self.min.x, self.max.x))
            .field("y", &format!("{:.3}..{:.3}", self.min.y, self.max.y))
            .finish()
    }
}

impl From<Point> for Bbox {
    fn from(value: Point) -> Self {
        Self {
            min: value,
            max: value,
        }
    }
}

impl std::ops::BitAndAssign<Point> for Bbox {
    fn bitand_assign(&mut self, rhs: Point) {
        self.min.x = self.min.x.min(rhs.x);
        self.min.y = self.min.y.min(rhs.y);
        self.max.x = self.max.x.max(rhs.x);
        self.max.y = self.max.y.max(rhs.y);
    }
}

impl std::ops::BitAndAssign<Bbox> for Bbox {
    fn bitand_assign(&mut self, rhs: Bbox) {
        self.min.x = self.min.x.min(rhs.min.x);
        self.min.y = self.min.y.min(rhs.min.y);
        self.max.x = self.max.x.max(rhs.max.x);
        self.max.y = self.max.y.max(rhs.max.y);
    }
}

impl Bbox {
    fn mid(&self) -> Point {
        Point {
            x: (self.min.x + self.max.x) * 0.5,
            y: (self.min.y + self.max.y) * 0.5,
        }
    }
}

fn input_loc(pos: impl AsRef<[f64]>) -> Location {
    let pos = pos.as_ref();
    Location::new(
        pos[1].clamp(-89.999999, 89.999999),
        pos[0].clamp(-179.999999, 179.999999),
    )
}

struct Line {
    bbox: Bbox,
    points: Vec<Point>,
}

impl std::fmt::Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Line")
            .field("bbox", &self.bbox)
            .field("points", &self.points.len())
            .finish()
    }
}

impl FromIterator<Point> for Line {
    fn from_iter<T: IntoIterator<Item = Point>>(iter: T) -> Self {
        Self::new(Vec::from_iter(iter))
    }
}

impl Line {
    pub fn new(points: Vec<Point>) -> Self {
        let mut bbox = Bbox::from(points[0]);
        for p in &points[1..] {
            bbox &= *p;
        }
        Self { bbox, points }
    }
}

struct Chunk {
    bbox: Bbox,
    lines: Vec<Vec<Point>>,
}

impl Chunk {
    fn from_lines(from: &[Line], global_box: Bbox, chunk: (usize, usize)) -> Option<Self> {
        let mut bbox = None;
        let mut lines = Vec::new();

        let scale_col = global_box.max.x - global_box.min.x;
        let offset_col = global_box.min.x;
        let scale_row = global_box.max.y - global_box.min.y;
        let offset_row = global_box.min.y;

        for line in from {
            let Point { x, y } = line.bbox.mid();
            let col = (((x - offset_col) / scale_col) * GRID_COLS as f64).floor() as usize;
            let row = (((y - offset_row) / scale_row) * GRID_ROWS as f64).floor() as usize;
            assert!(col < GRID_COLS);
            assert!(row < GRID_ROWS);
            if (col, row) == chunk {
                // matches!
                if bbox.is_none() {
                    bbox = Some(line.bbox);
                }
                *bbox.as_mut().unwrap() &= line.bbox;
                lines.push(line.points.clone());
            }
        }

        Some(Self { bbox: bbox?, lines })
    }

    fn compress(&self, min_dist: f64) -> Vec<Vec<(f64, f64)>> {
        self.lines
            .iter()
            .filter_map(|l| compressed(l, min_dist))
            .collect()
    }
}

/// Compress the data
fn compressed(points: &[Point], min_dist: f64) -> Option<Vec<(f64, f64)>> {
    let min_dist_sq = min_dist * min_dist;
    let num_p = points.len();
    let mut x = *points.first()?;

    let points: Vec<(f64, f64)> = points
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            if *i == 0 || i + 1 == num_p {
                x = **p;
                true
            } else {
                let dx = p.x - x.x;
                let dy = p.y - x.y;
                let dist = dx * dx + dy * dy;
                if dist > min_dist_sq {
                    x = **p;
                    true
                } else {
                    false
                }
            }
        })
        .map(|(_, p)| (p.x, p.y))
        .collect();

    if points.len() < 2 {
        None
    } else {
        Some(points)
    }
}

pub fn read_geojson(d: GeoJson) -> Vec<Vec<Location>> {
    let GeoJson::FeatureCollection(countries) = d else {
        panic!("Invalid country.geojson")
    };
    countries
        .features
        .into_iter()
        .flat_map(read_feature)
        .collect()
}

pub fn read_feature(f: Feature) -> Vec<Vec<Location>> {
    let Feature {
        geometry: Some(geometry),
        ..
    } = f
    else {
        panic!("Invalid country.geojson")
    };

    let polygon = match geometry.value {
        Value::Polygon(polygon) => vec![polygon],
        Value::MultiPolygon(multi) => multi,
        _ => panic!("Invalid country.geojson"),
    };

    polygon
        .into_iter()
        .flatten()
        .map(|x| x.iter().map(input_loc).collect())
        .collect()
}
