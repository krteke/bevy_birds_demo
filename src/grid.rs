use crate::boids::Bird;
use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Default)]
pub(super) struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<IVec2, Vec<usize>>,
    min_cell: IVec2,
    max_cell: IVec2,
}

impl SpatialGrid {
    pub(super) fn rebuild(&mut self, birds: &[Bird], radius: f32) {
        self.cell_size = radius;
        for cell in self.cells.values_mut() {
            cell.clear();
        }
        self.min_cell = IVec2::splat(i32::MAX);
        self.max_cell = IVec2::splat(i32::MIN);
        for (index, bird) in birds.iter().enumerate() {
            let cell = (bird.position / self.cell_size).floor().as_ivec2();
            self.min_cell = self.min_cell.min(cell);
            self.max_cell = self.max_cell.max(cell);
            self.cells.entry(cell).or_default().push(index);
        }
    }

    pub(super) fn nearest_distance(&self, birds: &[Bird], index: usize) -> Option<f32> {
        let position = birds[index].position;
        let cell = (position / self.cell_size).floor().as_ivec2();
        let limit = (cell - self.min_cell)
            .max(self.max_cell - cell)
            .max_element();
        let mut best = f32::INFINITY;
        for ring in 0..=limit {
            for y in -ring..=ring {
                for x in -ring..=ring {
                    if x.abs().max(y.abs()) != ring {
                        continue;
                    }
                    if let Some(indices) = self.cells.get(&(cell + IVec2::new(x, y))) {
                        for &other in indices {
                            if other != index {
                                best = best.min(position.distance_squared(birds[other].position));
                            }
                        }
                    }
                }
            }
            let lower = (cell - IVec2::splat(ring)).as_vec2() * self.cell_size;
            let upper = (cell + IVec2::splat(ring + 1)).as_vec2() * self.cell_size;
            let outside = (position - lower).min(upper - position).min_element();
            if outside * outside >= best {
                break;
            }
        }
        best.is_finite().then(|| best.sqrt())
    }

    pub(super) fn for_each_neighbor(
        &self,
        birds: &[Bird],
        index: usize,
        mut visit: impl FnMut(usize, Vec2, f32),
    ) {
        let position = birds[index].position;
        let cell = (position / self.cell_size).floor().as_ivec2();
        for y in -1..=1 {
            for x in -1..=1 {
                if let Some(cell) = self.cells.get(&(cell + IVec2::new(x, y))) {
                    for &other in cell {
                        let offset = position - birds[other].position;
                        let distance_squared = offset.length_squared();
                        if other != index && distance_squared <= self.cell_size * self.cell_size {
                            visit(other, offset, distance_squared);
                        }
                    }
                }
            }
        }
    }
}
