// tsp.rs - vibe coded so don't blame me for bugs

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Pt<'a> {
	pub id: Option<&'a str>,
	pub lat: f64,
	pub lng: f64,
}

fn dist(a: Pt, b: Pt) -> f64 {
	let dx = a.lat - b.lat;
	let dy = a.lng - b.lng;
	(dx * dx + dy * dy).sqrt()
}

//
// ---------------------------
// Route Construction
// ---------------------------
//

fn nearest_neighbor_cycle(points: &[Pt], start: usize) -> Vec<usize> {
	let n = points.len();
	let mut visited = vec![false; n];
	let mut route = Vec::with_capacity(n + 1);

	let mut current = start;
	visited[current] = true;
	route.push(current);

	for _ in 1..n {
		let mut best = None;
		let mut best_dist = f64::INFINITY;
		for i in 0..n {
			if !visited[i] {
				let d = dist(points[current], points[i]);
				if d < best_dist {
					best_dist = d;
					best = Some(i);
				}
			}
		}
		let next = best.unwrap();
		visited[next] = true;
		route.push(next);
		current = next;
	}

	// Explicitly close the cycle by returning to start
	route.push(start);

	route
}

fn nearest_neighbor_path(points: &[Pt], start: usize, end: usize) -> Vec<usize> {
	let n = points.len();
	let mut visited = vec![false; n];
	let mut route = Vec::with_capacity(n);

	visited[start] = true;
	// Do NOT visit end early
	visited[end] = false;

	let mut current = start;
	route.push(start);

	for _ in 0..n - 2 {
		let mut best = None;
		let mut best_dist = f64::INFINITY;

		for i in 0..n {
			if i == end {
				continue; // don't pick end until last
			}
			if !visited[i] {
				let d = dist(points[current], points[i]);
				if d < best_dist {
					best_dist = d;
					best = Some(i);
				}
			}
		}

		let next = best.unwrap();
		visited[next] = true;
		route.push(next);
		current = next;
	}

	// Now put end last
	route.push(end);

	route
}

//
// ---------------------------
// 2-opt optimization
// ---------------------------
//

fn two_opt_cycle(points: &[Pt], route: &mut Vec<usize>) {
	let n = route.len();
	let mut improved = true;

	while improved {
		improved = false;
		for i in 1..n - 2 {
			for j in i + 1..n - 1 {
				let a = route[i - 1];
				let b = route[i];
				let c = route[j];
				let d = route[j + 1];

				let before = dist(points[a], points[b]) + dist(points[c], points[d]);
				let after = dist(points[a], points[c]) + dist(points[b], points[d]);

				if after < before {
					route[i..=j].reverse();
					improved = true;
				}
			}
		}
	}
}

fn two_opt_path(points: &[Pt], route: &mut Vec<usize>) {
	let n = route.len();
	let mut improved = true;

	// route[0] and route[n-1] must not move
	while improved {
		improved = false;
		for i in 1..n - 3 {
			for j in i + 1..n - 2 {
				let a = route[i - 1];
				let b = route[i];
				let c = route[j];
				let d = route[j + 1];

				let before = dist(points[a], points[b]) + dist(points[c], points[d]);
				let after = dist(points[a], points[c]) + dist(points[b], points[d]);

				if after < before {
					route[i..=j].reverse();
					improved = true;
				}
			}
		}
	}
}

//
// ---------------------------
// Public API
// ---------------------------
//

pub enum EndpointMode {
	Circle,
	Path,
}

pub fn compute_route(points: &[Pt], mode: EndpointMode) -> Vec<usize> {
	match mode {
		EndpointMode::Circle => {
			let mut route = nearest_neighbor_cycle(points, 0);
			two_opt_cycle(points, &mut route);
			route
		}
		EndpointMode::Path => {
			let mut route = nearest_neighbor_path(points, 0, points.len() - 1);
			two_opt_path(points, &mut route);
			route
		}
	}
}
