//! # Quickhull
//!
//! A Rust-implementation of the Quickhull algorithm for computing convex hulls for point sets.
//!
//! This is a simplified and cleaned up version of [chull](https://github.com/u65xhd/chull),
//! focusing on making the algorithm robust and efficient for the 3D cases.
//!
//! ## References
//!
//! - C. Bradford Barber et al. 1996. [The Quickhull Algorithm for Convex Hulls](https://www.cise.ufl.edu/~ungor/courses/fall06/papers/QuickHull.pdf) (the original paper)
//! - Dirk Gregorius. GDC 2014. [Physics for Game Programmers: Implementing Quickhull](https://archive.org/details/GDC2014Gregorius)

#![warn(missing_docs, clippy::all)]

use glam::{DMat4, DVec3};

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt;

#[cfg(test)]
mod tests;

/// A polygonal face belonging to a [`ConvexHull`].
#[derive(Debug, Clone)]
pub struct Face {
    /// The indices of the face's points.
    pub indices: Vec<usize>,
    /// The indices of points in front of the face plane, or the points that can "see" the face,
    /// and the distance to each of those points along the normal.
    pub outside_points: Vec<(usize, f64)>,
    /// The indices of neighboring faces.
    pub neighbor_faces: Vec<usize>,
    /// The normal of the face.
    pub normal: DVec3,
    /// How far away from the origin this face is along its normal.
    pub distance_from_origin: f64,
}

impl Face {
    /// Creates a [`Face`] using the `points` with the given `indices`.
    #[must_use]
    pub fn from_triangle(points: &[DVec3], indices: [usize; 3]) -> Self {
        let points_of_face = indices.map(|i| points[i]);
        let normal = triangle_normal(points_of_face);
        let origin = normal.dot(points_of_face[0]);

        Self {
            indices: indices.to_vec(),
            outside_points: Vec::new(),
            neighbor_faces: Vec::new(),
            normal,
            distance_from_origin: origin,
        }
    }
}

/// The type of error returned during [`ConvexHull`] construction.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// The given point set is empty, so no convex hull could be computed.
    Empty,
    /// The convex hull generation algorithm encountered degeneracies.
    Degenerated,
    /// The given point set cannot produce a valid convex hull.
    DegenerateInput(DegenerateInput),
    /// A round-off error.
    RoundOffError(&'static str),
}

/// The type of degeneracy for when attempting to compute a convex hull for a point set.
#[derive(Debug, Clone, PartialEq)]
pub enum DegenerateInput {
    /// The input points are approximately equal.
    Coincident,
    /// The input points are approximately on the same line.
    Collinear,
    /// The input points are approximately on the same plane.
    Coplanar,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ErrorKind::Empty => write!(f, "empty"),
            ErrorKind::Degenerated => write!(f, "degenerated"),
            ErrorKind::DegenerateInput(kind) => write!(f, "degenerate input: {:?}", kind),
            ErrorKind::RoundOffError(msg) => {
                write!(f, "erroneous results by roundoff error: {msg}")
            }
        }
    }
}
impl Error for ErrorKind {}

/// A 3D convex hull representing the smallest convex set containing
/// all input points in a given point set.
///
/// This can be thought of as a shrink wrapping of a 3D object.
#[derive(Clone, Debug)]
pub struct ConvexHull {
    /// The points of the convex hull.
    pub points: Vec<DVec3>,
    /// The faces of the convex hull.
    faces: BTreeMap<usize, Face>,
}

impl ConvexHull {
    /// Attempts to compute a [`ConvexHull`] for the given set of points.
    ///
    /// ## Errors
    /// If their are 0 or <= 3 points.
    pub fn try_new(points: &[DVec3], max_iter: Option<usize>) -> Result<Self, ErrorKind> {
        let num_points = points.len();

        if num_points == 0 {
            return Err(ErrorKind::Empty);
        }

        if num_points <= 3 {
            return Err(ErrorKind::Degenerated);
        }

        // Create the initial simplex, a tetrahedron in 3D.
        let mut c_hull = Self::init_tetrahedron(points)?;

        // Run the main quick hull algorithm.
        c_hull.update(max_iter)?;

        // Shrink the hull, removing unused points.
        c_hull.remove_unused_points();

        if c_hull.points.len() <= 3 {
            return Err(ErrorKind::Degenerated);
        }

        Ok(c_hull)
    }

    /// Computes the minimum and maximum extents for the given point set, along with
    /// the indices of the minimum and maximum vertices along each coordinate axis.
    fn compute_extremes(points: &[DVec3]) -> ([usize; 3], [usize; 3]) {
        let mut min = points[0];
        let mut max = points[0];

        let mut min_vertices = [0; 3];
        let mut max_vertices = [0; 3];

        for (i, vtx) in points.iter().enumerate().skip(1) {
            if vtx.x < min.x {
                min.x = vtx.x;
                min_vertices[0] = i;
            } else if vtx.x > max.x {
                max.x = vtx.x;
                max_vertices[0] = i;
            }

            if vtx.y < min.y {
                min.y = vtx.y;
                min_vertices[1] = i;
            } else if vtx.y > max.y {
                max.y = vtx.y;
                max_vertices[1] = i;
            }

            if vtx.z < min.z {
                min.z = vtx.z;
                min_vertices[2] = i;
            } else if vtx.z > max.z {
                max.z = vtx.z;
                max_vertices[2] = i;
            }
        }

        (min_vertices, max_vertices)
    }

    fn init_tetrahedron(points: &[DVec3]) -> Result<Self, ErrorKind> {
        let (min_indices, max_indices) = Self::compute_extremes(points);
        // Get the indices of the vertices used for the initial tetrahedron.
        let indices_set = Self::init_tetrahedron_indices(points, min_indices, max_indices)?;

        let mut faces = BTreeMap::new();

        #[allow(clippy::explicit_counter_loop)]
        for i_face in 0..4 {
            let mut face_indices = [0, 0, 0];
            let mut face_indices_pos = 0;
            // create face
            for (j, index) in indices_set.iter().enumerate() {
                if j != i_face {
                    face_indices[face_indices_pos] = *index;
                    face_indices_pos += 1;
                }
            }
            let mut face = Face::from_triangle(points, face_indices);

            // Check the order of the face's vertices.
            let rem_point = indices_set[i_face];
            let pos = position_from_face(points, &face, rem_point);
            if pos > 0.0 {
                face.indices.swap(0, 1);
                face.normal = -face.normal;
                face.distance_from_origin = -face.distance_from_origin;
            }
            if face.indices.len() != 3 {
                return Err(ErrorKind::RoundOffError(
                    "number of face's vertices should be 3",
                ));
            }
            faces.insert(i_face, face);
        }

        // Link neighbors.
        let simplex_face_key: Vec<_> = faces.keys().copied().collect();
        for (key, face) in &mut faces.iter_mut() {
            for neighbors_key in simplex_face_key
                .iter()
                .filter(|neighbor_key| *neighbor_key != key)
            {
                face.neighbor_faces.push(*neighbors_key);
            }
        }

        let simplex = Self {
            points: points.to_vec(),
            faces,
        };

        Ok(simplex)
    }

    /// Computes the indices for the initial tetrahdron built from the given
    /// `points` and the indices of the extreme points along each axis.
    fn init_tetrahedron_indices(
        points: &[DVec3],
        min_indices: [usize; 3],
        max_indices: [usize; 3],
    ) -> Result<[usize; 4], ErrorKind> {
        let mut indices = [0; 4];
        #[cfg(not(test))]
        debug_assert!(
            points.len() > 3,
            "This should be checked before this function"
        );

        // The maximum one-dimensional extent of the point-cloud, and the index
        // corresponding to that dimension (x = 0, y = 1, z = 2).
        let mut max_extent = 0.0;
        let mut max_dimension_index = 0;

        for i in 0..3 {
            let extent = points[max_indices[i]][i] - points[min_indices[i]][i];
            if extent > max_extent {
                max_extent = extent;
                max_dimension_index = i;
            }
        }

        if max_extent == 0.0 {
            // The point cloud seems to consist of a single point.
            return Err(ErrorKind::DegenerateInput(DegenerateInput::Coincident));
        }

        // The first two vertices are the ones farthest apart in the maximum dimension.
        indices[0] = max_indices[max_dimension_index];
        indices[1] = min_indices[max_dimension_index];

        // The third vertex should be the one farthest from the line segment
        // between the first two vertices.
        let unit_01 = (points[indices[1]] - points[indices[0]]).normalize();
        let mut normal = DVec3::ZERO;

        let mut max_squared_distance = 0.0;

        for i in 0..points.len() {
            let diff = points[i] - points[indices[0]];
            let cross = unit_01.cross(diff);
            let distance_squared = cross.length_squared();

            if distance_squared > max_squared_distance
                && points[i] != points[indices[0]]
                && points[i] != points[indices[1]]
            {
                max_squared_distance = distance_squared;
                indices[2] = i;
                normal = cross;
            }
        }

        if max_squared_distance == 0.0 {
            return Err(ErrorKind::DegenerateInput(DegenerateInput::Collinear));
        }

        normal = normal.normalize();

        // Recompute the normal to make sure it is perpendicular to unit_10.
        normal = (normal - normal.dot(unit_01) * unit_01).normalize();

        // We now have a base triangle. The fourth vertex should be the one farthest
        // from the triangle along the normal.
        let mut max_distance = 0.0;
        let d0 = points[indices[2]].dot(normal);

        for i in 0..points.len() {
            let distance = (points[i].dot(normal) - d0).abs();

            if distance > max_distance
                && points[i] != points[indices[0]]
                && points[i] != points[indices[1]]
                && points[i] != points[indices[2]]
            {
                max_distance = distance;
                indices[3] = i;
            }
        }

        if max_distance.abs() == 0.0 {
            return Err(ErrorKind::DegenerateInput(DegenerateInput::Coplanar));
        }

        Ok(indices)
    }

    fn update(&mut self, max_iter: Option<usize>) -> Result<(), ErrorKind> {
        let mut face_add_count = *self.faces.keys().last().unwrap() + 1;
        let mut num_iter = 0;
        let mut assigned_point_indices: HashSet<usize> = HashSet::new();

        // Mark the points of the faces as assigned.
        for face in self.faces.values() {
            for index in &face.indices {
                assigned_point_indices.insert(*index);
            }
        }

        // Initialize the outside points, sometimes called "conflict lists".
        // They are outside the current hull, but can "see" some faces and therefore could be on the final hull.
        for (_key, face) in &mut self.faces.iter_mut() {
            for (i, _point) in self.points.iter().enumerate() {
                if assigned_point_indices.contains(&i) {
                    continue;
                }

                let pos = position_from_face(&self.points, face, i);

                // If the point can "see" the face, add it to the face's list of outside points.
                if pos > 0.0 {
                    face.outside_points.push((i, pos));
                }
            }
        }

        let (max_iter, truncate) = if let Some(iter) = max_iter {
            (iter, true)
        } else {
            (0, false)
        };

        // The main algorithm of quick hull.
        //
        // For each face that has outside points:
        //
        // 1. Find the outside point that is farthest from the face, the "eye point".
        // 2. Find the "horizon", the vertices that form the boundary between the visible
        //    and non-visible parts of the current hull from the viewpoint of the eye point.
        // 3. Create faces connecting the horizon vertices to the eye point.
        // 4. Assign the orphaned vertices to the new faces, and remove the old faces.
        // 5. Repeat.
        while let Some((key, face)) = self
            .faces
            .iter()
            .find(|(_, face)| !face.outside_points.is_empty())
            .map(|(a, b)| (*a, b))
        {
            if truncate && num_iter >= max_iter {
                break;
            }

            num_iter += 1;

            // Select the furthest point.
            let (furthest_point_index, _) = *face.outside_points.last().unwrap();

            // Initialize the visible set.
            let visible_set =
                initialize_visible_set(&self.points, furthest_point_index, &self.faces, key, face);

            // Get the horizon.
            let horizon = compute_horizon(&visible_set, &self.faces)?;

            // Create new faces connecting the horizon vertices to the furthest point.
            let mut new_keys = Vec::with_capacity(horizon.len());
            for (ridge, unvisible) in horizon {
                let mut new_face = vec![furthest_point_index];

                assigned_point_indices.insert(furthest_point_index);

                for point in ridge {
                    new_face.push(point);
                    assigned_point_indices.insert(point);
                }

                if new_face.len() != 3 {
                    return Err(ErrorKind::RoundOffError(
                        "number of new face's vertices should be 3",
                    ));
                }

                let mut new_face = Face::from_triangle(&self.points, new_face.try_into().unwrap());
                new_face.neighbor_faces.push(unvisible);

                let new_key = face_add_count;
                face_add_count += 1;

                self.faces.insert(new_key, new_face);
                let unvisible_faset = self.faces.get_mut(&unvisible).unwrap();
                unvisible_faset.neighbor_faces.push(new_key);
                new_keys.push(new_key);
            }

            if new_keys.len() < 3 {
                return Err(ErrorKind::RoundOffError(
                    "number of new faces should be grater than 3",
                ));
            }

            // Link the faces to their neighbors.
            for (i, key_a) in new_keys.iter().enumerate() {
                let points_of_new_face_a: HashSet<_> = self
                    .faces.get(key_a)
                    .unwrap().indices
                    .iter().copied()
                    .collect();

                for key_b in new_keys.iter().skip(i + 1) {
                    let points_of_new_face_b: HashSet<_> = self
                        .faces
                        .get(key_b).unwrap()
                        .indices.iter()
                        .copied().collect();

                    let num_intersection_points = points_of_new_face_a
                        .intersection(&points_of_new_face_b)
                        .collect::<Vec<_>>()
                        .len();

                    if num_intersection_points == 2 {
                        let face_a = self.faces.get_mut(key_a).unwrap();
                        face_a.neighbor_faces.push(*key_b);

                        let face_b = self.faces.get_mut(key_b).unwrap();
                        face_b.neighbor_faces.push(*key_a);
                    }
                }

                let face_a = self.faces.get(key_a).unwrap();
                if face_a.neighbor_faces.len() != 3 {
                    return Err(ErrorKind::RoundOffError(
                        "number of neighbors should be 3",
                    ));
                }
            }

            // Check the order of the new face's vertices.
            for new_key in &new_keys {
                let new_face = self.faces.get(new_key).unwrap();
                let mut degenerate = true;

                for assigned_point_index in &assigned_point_indices {
                    let position = position_from_face(&self.points, new_face, *assigned_point_index);

                    if position == 0.0 {
                        continue;
                    } else if position > 0.0 {
                        let new_face = self.faces.get_mut(new_key).unwrap();
                        new_face.indices.swap(0, 1);
                        new_face.normal = -new_face.normal;
                        new_face.distance_from_origin = -new_face.distance_from_origin;
                        degenerate = false;
                        break;
                    }

                    degenerate = false;
                    break;
                }

                if degenerate {
                    return Err(ErrorKind::Degenerated);
                }
            }

            // Assign the orphaned vertices to the new faces.
            let mut visible_faces = Vec::with_capacity(visible_set.len());
            for visible in &visible_set {
                visible_faces.push(self.faces.get(visible).unwrap().clone());
            }

            for new_key in &new_keys {
                let new_face = self.faces.get_mut(new_key).unwrap();
                let mut checked_point_set = HashSet::new();

                for visible_face in &visible_faces {
                    for (outside_point_index, _) in &visible_face.outside_points {
                        if assigned_point_indices.contains(outside_point_index)
                            || checked_point_set.contains(outside_point_index)
                        {
                            continue;
                        }

                        checked_point_set.insert(outside_point_index);

                        let pos = position_from_face(&self.points, new_face, *outside_point_index);
                        if pos > 0.0 {
                            new_face.outside_points.push((*outside_point_index, pos));
                        }
                    }
                }

                new_face
                    .outside_points
                    .sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
            }

            // Delete the old visible faces.
            for visible in visible_set {
                let visible_face = self.faces.get(&visible).unwrap().clone();
                for neighbor_key in visible_face.neighbor_faces {
                    let neighbor = self.faces.get_mut(&neighbor_key).unwrap();
                    let index = neighbor
                        .neighbor_faces
                        .iter()
                        .enumerate()
                        .find(|(_, k)| **k == visible)
                        .map(|(i, _)| i)
                        .unwrap();
                    neighbor.neighbor_faces.swap_remove(index);
                }
                self.faces.remove(&visible);
            }
        }

        if !self.is_convex() {
            return Err(ErrorKind::RoundOffError("concave"));
        }

        Ok(())
    }

    /// Adds the given points to the point set, attempting to update the convex hull.
    ///
    /// ## Errors
    /// If updating the points fails or results in less then three points.
    pub fn add_points(&mut self, points: &mut Vec<DVec3>) -> Result<(), ErrorKind> {
        self.points.append(points);
        self.update(None)?;
        self.remove_unused_points();

        if self.points.len() <= 3 {
            return Err(ErrorKind::Degenerated);
        }

        Ok(())
    }

    /// Adds the given iterator of points to the point set, attempting to update the convex hull.
    ///
    /// ## Errors
    /// If updating the points fails or results in less then three points.
    pub fn add_iter_points(&mut self, points: impl Iterator<Item = DVec3>) -> Result<(), ErrorKind> {
        self.points.extend(points);
        self.update(None)?;
        self.remove_unused_points();

        if self.points.len() <= 3 {
            return Err(ErrorKind::Degenerated);
        }

        Ok(())
    }

    /// Returns the vertices and indices of the convex hull.
    #[must_use]
    pub fn vertices_indices(self) -> (Vec<DVec3>, Vec<usize>) {
        let mut indices = Vec::new();
        for face in self.faces.values() {
            for i in &face.indices {
                indices.push(*i);
            }
        }
        (self.points, indices)
    }

    pub(crate) fn remove_unused_points(&mut self) {
        let mut indices_list = BTreeSet::new();

        for face in self.faces.values() {
            for i in &face.indices {
                indices_list.insert(*i);
            }
        }

        let indices_list: BTreeMap<usize, usize> = indices_list
            .into_iter()
            .enumerate()
            .map(|(i, index)| (index, i))
            .collect();

        for face in self.faces.values_mut() {
            let mut new_face_indices = Vec::with_capacity(face.indices.len());
            for i in &face.indices {
                new_face_indices.push(*indices_list.get(i).unwrap());
            }
            std::mem::swap(&mut face.indices, &mut new_face_indices);
        }

        let mut vertices = Vec::with_capacity(indices_list.len());

        for (index, _) in indices_list {
            vertices.push(self.points[index]);
        }

        self.points = vertices;
    }

    /// Computes the volume of the convex hull.
    /// Sums up volumes of tetrahedrons from an arbitrary point to all other points
    ///
    /// Returns non-negative value, for extremely small objects might return 0.0
    #[must_use]
    pub fn volume(self) -> f64 {
        let (hull_vertices, hull_indices) = self.vertices_indices();
        let reference_point = hull_vertices[hull_indices[0]].extend(1.0);
        let mut volume = 0.0;
        for i in (3..hull_indices.len()).step_by(3) {
            let mut mat = DMat4::ZERO;
            for j in 0..3 {
                let row = hull_vertices[hull_indices[i + j]].extend(1.0);
                *mat.col_mut(j) = row;
            }
            *mat.col_mut(3) = reference_point;
            volume += mat.determinant().max(0.0);
        }
        volume / 6.0
    }

    /// Checks if the convex hull is convex with the given tolerance.
    fn is_convex(&self) -> bool {
        self.faces.values().any(|face| {
            position_from_face(&self.points, face, 0) <= 0.0
        })
    }

    /// Computes the point on the convex hull that is furthest in the given direction.
    #[must_use]
    pub fn support_point(&self, direction: DVec3) -> DVec3 {
        let mut max = self.points[0].dot(direction);
        let mut index = 0;

        for (i, point) in self.points.iter().enumerate().skip(1) {
            let dot_product = point.dot(direction);
            if dot_product > max {
                max = dot_product;
                index = i;
            }
        }

        self.points[index]
    }
}

// Computes the indices of the faces that are visible from the point farthest from the given `face`.
fn initialize_visible_set(
    points: &[DVec3],
    furthest_point_index: usize,
    faces: &BTreeMap<usize, Face>,
    face_key: usize,
    face: &Face,
) -> HashSet<usize> {
    let mut visible_set = HashSet::new();
    visible_set.insert(face_key);
    let mut neighbor_stack: Vec<_> = face.neighbor_faces.clone();
    let mut visited_neighbor = HashSet::new();
    while let Some(neighbor_key) = neighbor_stack.pop() {
        if visited_neighbor.contains(&neighbor_key) {
            continue;
        }

        visited_neighbor.insert(neighbor_key);

        let neighbor = faces.get(&neighbor_key).unwrap();
        let pos = position_from_face(points, neighbor, furthest_point_index);
        if pos > 0.0 {
            visible_set.insert(neighbor_key);
            neighbor_stack.append(&mut neighbor.neighbor_faces.clone());
        }
    }
    visible_set
}

/// Tries to computes the horizon represented as a vector of ridges and the keys of their neighbors.
fn compute_horizon(
    visible_set: &HashSet<usize>,
    faces: &BTreeMap<usize, Face>,
) -> Result<Vec<(Vec<usize>, usize)>, ErrorKind> {
    let mut horizon = Vec::new();
    for visible_key in visible_set {
        let visible_face = faces.get(visible_key).unwrap();
        let points_of_visible_face: HashSet<_> = visible_face.indices.iter().copied().collect();
        if points_of_visible_face.len() != 3 {
            return Err(ErrorKind::RoundOffError(
                "number of visible face's vertices should be 3",
            ));
        }

        for neighbor_key in &visible_face.neighbor_faces {
            // if neighbor is unvisible
            if !visible_set.contains(neighbor_key) {
                let unvisible_neighbor = faces.get(neighbor_key).unwrap();
                let points_of_unvisible_neighbor: HashSet<usize> =
                    unvisible_neighbor.indices.iter().copied().collect();
                if points_of_unvisible_neighbor.len() != 3 {
                    return Err(ErrorKind::RoundOffError(
                        "number of unvisible face's vertices should be 3",
                    ));
                }

                let horizon_ridge: Vec<_> = points_of_unvisible_neighbor
                    .intersection(&points_of_visible_face)
                    .copied()
                    .collect();
                if horizon_ridge.len() != 2 {
                    return Err(ErrorKind::RoundOffError(
                        "number of ridge's vertices should be 2",
                    ));
                }
                horizon.push((horizon_ridge, *neighbor_key));
            }
        }
    }

    if horizon.len() < 3 {
        return Err(ErrorKind::RoundOffError("horizon len < 3"));
    }
    Ok(horizon)
}

trait ToRobust {
    fn to_robust(self) -> robust::Coord3D<f64>;
}

impl ToRobust for glam::DVec3 {
    fn to_robust(self) -> robust::Coord3D<f64> {
        robust::Coord3D { x: self.x, y: self.y, z: self.z }
    }
}

fn position_from_face(points: &[DVec3], face: &Face, point_index: usize) -> f64 {
    -robust::orient3d(
        points[face.indices[0]].to_robust(),
        points[face.indices[1]].to_robust(),
        points[face.indices[2]].to_robust(),
        points[point_index].to_robust(),
    )
}

/// Computes the normal of a triangle face with a counterclockwise orientation.
fn triangle_normal([a, b, c]: [DVec3; 3]) -> DVec3 {
    let ab = b - a;
    let ac = c - a;
    ab.cross(ac)
}
