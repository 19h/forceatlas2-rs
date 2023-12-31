mod drawer;
mod gui;

use forceatlas2::*;
use parking_lot::RwLock;
use std::{io::BufRead, sync::Arc, thread, time::Duration};

const STANDBY_SLEEP: Duration = Duration::from_millis(50);
const COMPUTE_SLEEP: Duration = Duration::from_millis(1);
type T = f32;

fn main() {
	let file = std::fs::File::open(std::env::args().nth(1).expect("Usage: viz <csv_file>"))
		.expect("Cannot open file");

	let mut nodes = 0usize;
	let mut edges = Vec::<(usize, usize)>::new();
	let mut weights = Vec::new();
	for (i, line) in std::io::BufReader::new(file).lines().enumerate() {
		let line = line.expect("Error reading CSV");
		let mut columns = line.split(&[' ', '\t', ',', ';'][..]);
		if let (Some(n1), Some(n2)) = (columns.next(), columns.next()) {
			if let (Ok(n1), Ok(n2)) = (n1.parse(), n2.parse()) {
				if n1 > nodes {
					nodes = n1;
				}
				if n2 > nodes {
					nodes = n2;
				}
				if n1 != n2 {
					edges.push(if n1 < n2 { (n1, n2) } else { (n2, n1) });
					weights.push(columns.next().map_or(1.0, |w| {
						w.parse().unwrap_or_else(|_| {
							eprintln!("Ignored weight line {} has bad number format", i);
							1.0
						})
					}));
				} else {
					eprintln!("Ignored line {} has loop", i);
				}
			} else {
				eprintln!("Ignored line {} has bad number format", i);
			}
		} else {
			eprintln!("Ignored line {} has <2 columns", i);
		}
	}
	nodes += 1;

	println!("Nodes: {}", nodes);

	let settings = Settings {
		barnes_hut: None,
		chunk_size: Some(256),
		dimensions: 2,
		dissuade_hubs: false,
		ka: 1.0,
		kg: 1.0,
		kr: 1.0,
		lin_log: false,
		prevent_overlapping: None,
		speed: 0.01,
		strong_gravity: false,
	};

	let layout = Arc::new(RwLock::new(Layout::<T>::from_graph(
		edges,
		Nodes::Degree(nodes),
		Some(weights),
		settings.clone(),
	)));

	let compute = Arc::new(RwLock::new(false));
	let settings = Arc::new(RwLock::new(settings));
	let nb_iters = Arc::new(RwLock::new(0usize));

	thread::spawn({
		let compute = compute.clone();
		let layout = layout.clone();
		let nb_iters = nb_iters.clone();
		move || loop {
			thread::sleep(if *compute.read() {
				let mut nb_iters = nb_iters.write();
				layout.write().iteration();
				*nb_iters += 1;
				COMPUTE_SLEEP
			} else {
				STANDBY_SLEEP
			});
		}
	});

	gui::run(compute, layout, settings, nb_iters);
}
