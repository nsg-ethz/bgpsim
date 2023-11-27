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
    fs::{create_dir_all, write},
    path::PathBuf,
};

use bgpsim::topology_zoo::TopologyZoo;
use geojson::{Feature, GeoJson, Value};
use geoutils::Location;
use itertools::Itertools;
use mapproj::{cylindrical::mer::Mer, LonLat, Projection};

const ACCURACY: f64 = 300.0;

fn main() {
    println!("cargo:rerun-if-changed=maps/countries.geojson");

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut root = PathBuf::from(out_dir);
    root.push("maps");
    root.push("geodata");
    create_dir_all(&root).unwrap();

    let countries_geojson = include_str!("maps/countries.geojson");
    let lines = Line::from_geojson(countries_geojson.parse::<GeoJson>().unwrap());

    println!("{} lines", lines.len());
    for line in &lines {
        println!("{line:?}");
    }

    // iterate over each topology
    for topo in TopologyZoo::topologies_increasing_nodes() {
        let data = build_topo(topo, &lines);

        let s = serde_json::to_string(&data).unwrap();
        root.push(format!("{topo}.json"));
        write(&root, s).unwrap();
        root.pop();
    }
}

fn build_topo(topo: &TopologyZoo, lines: &[Line]) -> Vec<Vec<[f64; 2]>> {
    let Some(bbox) = topo_bounding_box(topo) else {
        return Vec::new();
    };

    // get the lines that overlap
    let overlapping_lines = lines.iter().filter(|x| x.bbox.overlap(&bbox));

    // project all points down to a 2d plane.
    let proj = Mer::new();
    let projected_lines = overlapping_lines.map(|l| project(&l.points, &proj));

    // compress all lines
    let min_dist = bbox.projected_size(&proj) / ACCURACY;
    let compressed_lines = projected_lines.filter_map(|l| compress(l, min_dist));

    // return the results
    compressed_lines.collect()
}

fn topo_bounding_box(topo: &TopologyZoo) -> Option<Bbox> {
    let loc = topo
        .geo_location()
        .into_values()
        .filter(|x| x.latitude() != 0.0 || x.longitude() != 0.0)
        .collect::<Vec<_>>();

    if loc.len() < 2 {
        return None;
    }

    let mut loc = loc.into_iter();
    let mut bbox = Bbox::from(loc.next().unwrap());
    for coord in loc {
        bbox &= coord;
    }

    Some(bbox)
}

fn project<P: Projection>(points: &[Location], proj: &P) -> Vec<[f64; 2]> {
    points
        .iter()
        .map(|p| {
            let xy = proj.proj_lonlat(&rad(*p)).unwrap();
            [xy.x(), -xy.y()]
        })
        .collect()
}

fn compress(points: Vec<[f64; 2]>, min_dist: f64) -> Option<Vec<[f64; 2]>> {
    let min_dist_sq = min_dist * min_dist;
    let num_p = points.len();
    let mut x = *points.first()?;

    let points = points
        .into_iter()
        .enumerate()
        .filter(|(i, p)| {
            if *i == 0 || i + 1 == num_p {
                x = *p;
                true
            } else {
                let dx = p[0] - x[0];
                let dy = p[1] - x[1];
                let dist = dx * dx + dy * dy;
                if dist > min_dist_sq {
                    x = *p;
                    true
                } else {
                    false
                }
            }
        })
        .map(|(_, p)| p)
        .collect_vec();

    if points.len() < 2 {
        None
    } else {
        Some(points)
    }
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
struct Bbox {
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
}

impl std::fmt::Debug for Bbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bbox")
            .field("lat", &format!("{:.3}..{:.3}", self.min_lat, self.max_lat))
            .field("lon", &format!("{:.3}..{:.3}", self.min_lon, self.max_lon))
            .finish()
    }
}

impl From<Location> for Bbox {
    fn from(value: Location) -> Self {
        Self {
            min_lat: value.latitude(),
            max_lat: value.latitude(),
            min_lon: value.longitude(),
            max_lon: value.longitude(),
        }
    }
}

impl Bbox {
    fn overlap(&self, other: &Self) -> bool {
        self.max_lat > other.min_lat && self.max_lon > other.min_lon
    }

    fn projected_size<P: Projection>(&self, proj: &P) -> f64 {
        let min = rad(Location::new(self.min_lat, self.min_lon));
        let max = rad(Location::new(self.max_lat, self.max_lon));
        let p_min = proj.proj_lonlat(&min).unwrap();
        let p_max = proj.proj_lonlat(&max).unwrap();

        let dx = (p_min.x() - p_max.x()).abs();
        let dy = (p_min.y() - p_max.y()).abs();

        f64::max(dx, dy)
    }
}

impl std::ops::BitAndAssign<Location> for Bbox {
    fn bitand_assign(&mut self, rhs: Location) {
        self.min_lat = self.min_lat.min(rhs.latitude());
        self.max_lat = self.max_lat.max(rhs.latitude());
        self.min_lon = self.min_lon.min(rhs.longitude());
        self.max_lon = self.max_lon.max(rhs.longitude());
    }
}

impl std::ops::BitAndAssign<Bbox> for Bbox {
    fn bitand_assign(&mut self, rhs: Bbox) {
        self.min_lat = self.min_lat.min(rhs.min_lat);
        self.max_lat = self.max_lat.max(rhs.max_lat);
        self.min_lon = self.min_lon.min(rhs.min_lon);
        self.max_lon = self.max_lon.max(rhs.max_lon);
    }
}

fn loc(pos: impl AsRef<[f64]>) -> Location {
    let pos = pos.as_ref();
    Location::new(pos[1], pos[0].clamp(-179.99999999, 179.99999999))
}

struct Line {
    bbox: Bbox,
    points: Vec<Location>,
}

impl std::fmt::Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Line")
            .field("bbox", &self.bbox)
            .field("points", &self.points.len())
            .finish()
    }
}

impl FromIterator<Location> for Line {
    fn from_iter<T: IntoIterator<Item = Location>>(iter: T) -> Self {
        Self::new(Vec::from_iter(iter))
    }
}

impl Line {
    pub fn new(points: Vec<Location>) -> Self {
        let mut bbox = Bbox::from(points[0]);
        for p in &points[1..] {
            bbox &= *p;
        }
        Self { bbox, points }
    }

    pub fn from_geojson(d: GeoJson) -> Vec<Line> {
        let GeoJson::FeatureCollection(countries) = d else {
            panic!("Invalid country.geojson")
        };
        countries
            .features
            .into_iter()
            .flat_map(Self::from_feature)
            .collect()
    }

    pub fn from_feature(f: Feature) -> Vec<Line> {
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
            .map(|x| x.iter().map(loc).collect::<Line>())
            .collect()
    }
}
