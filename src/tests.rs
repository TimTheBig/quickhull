use super::*;

#[test]
fn four_points_coincident() {
    let points = (0..4).map(|_| DVec3::splat(1.0)).collect::<Vec<_>>();

    let result = ConvexHull::try_new(&points, None);
    assert!(
        matches!(
            result,
            Err(ErrorKind::DegenerateInput(DegenerateInput::Coincident))
        ),
        "{result:?} should be 'coincident' error"
    );
}

#[test]
fn four_points_collinear() {
    let mut points = (0..4).map(|_| DVec3::splat(1.0)).collect::<Vec<_>>();
    points[0].x += f64::EPSILON;
    let result = ConvexHull::try_new(&points, None);
    assert!(
        matches!(
            result,
            Err(ErrorKind::DegenerateInput(DegenerateInput::Collinear))
        ),
        "{result:?} should be 'collinear' error"
    );
}

#[test]
fn four_points_coplanar() {
    let mut points = (0..4).map(|_| DVec3::splat(1.0)).collect::<Vec<_>>();
    points[0].x += f64::EPSILON;
    points[1].y += f64::EPSILON;
    let result = ConvexHull::try_new(&points, None);
    assert!(
        matches!(
            result,
            Err(ErrorKind::DegenerateInput(DegenerateInput::Coplanar))
        ),
        "{result:?} should be 'coplanar' error"
    );
}

#[test]
fn four_points_min_volume() {
    let mut points = (0..4).map(|_| DVec3::splat(1.0)).collect::<Vec<_>>();
    points[0].x += 3.0 * f64::EPSILON;
    points[1].y += 3.0 * f64::EPSILON;
    points[2].z += 3.0 * f64::EPSILON;
    let result = ConvexHull::try_new(&points, None);
    assert_eq!(
        4.3790577010150533e-47,
        result.expect("this should compute ok").volume()
    );
}

#[test]
fn volume_should_be_positive() {
    let mut points = (0..4).map(|_| DVec3::splat(1.0)).collect::<Vec<_>>();
    points[0].x += 1.0 * f64::EPSILON;
    points[1].y += 1.0 * f64::EPSILON;
    points[2].z += 2.0 * f64::EPSILON;
    let result = ConvexHull::try_new(&points, None);
    assert!(result.expect("this should compute ok").volume() > 0.0);
}

#[test]
fn face_normal_test() {
    let p1 = DVec3::new(-1.0, 0.0, 0.0);
    let p2 = DVec3::new(1.0, 0.0, 0.0);
    let p3 = DVec3::new(0.0, 1.0, 0.0);
    let normal_z = triangle_normal([p1, p2, p3]);
    assert_eq!(normal_z, DVec3::new(0.0, 0.0, 2.0));

    let p1 = DVec3::new(0.0, -1.0, 0.0);
    let p2 = DVec3::new(0.0, 1.0, 0.0);
    let p3 = DVec3::new(0.0, 0.0, 1.0);
    let normal_x = triangle_normal([p1, p2, p3]);
    assert_eq!(normal_x, DVec3::new(2.0, 0.0, 0.0));

    let p1 = DVec3::new(0.0, 0.0, -1.0);
    let p2 = DVec3::new(0.0, 0.0, 1.0);
    let p3 = DVec3::new(1.0, 0.0, 0.0);
    let normal_y = triangle_normal([p1, p2, p3]);
    assert_eq!(normal_y, DVec3::new(0.0, 2.0, 0.0));
}

#[test]
fn inner_outer_test() {
    let p1 = DVec3::new(1.0, 0.0, 0.0);
    let p2 = DVec3::new(0.0, 1.0, 0.0);
    let p3 = DVec3::new(0.0, 0.0, 1.0);
    let outer_point = DVec3::new(0.0, 0.0, 10.0);
    let inner_point = DVec3::new(0.0, 0.0, 0.0);
    let within_point = DVec3::new(1.0, 0.0, 0.0);

    let points = vec![p1, p2, p3, outer_point, inner_point, within_point];

    let face = Face::from_triangle(&points, [0, 1, 2]);
    let outer = position_from_face(&points, &face, 3);
    assert!(outer > 0.0);
    let inner = position_from_face(&points, &face, 4);
    assert!(inner < 0.0);
    let within = position_from_face(&points, &face, 5);
    assert!(within == 0.0);
}

#[test]
fn octahedron_test() {
    let p1 = DVec3::new(1.0, 0.0, 0.0);
    let p2 = DVec3::new(0.0, 1.0, 0.0);
    let p3 = DVec3::new(0.0, 0.0, 1.0);
    let p4 = DVec3::new(-1.0, 0.0, 0.0);
    let p5 = DVec3::new(0.0, -1.0, 0.0);
    let p6 = DVec3::new(0.0, 0.0, -1.0);

    let (_v, i) = ConvexHull::try_new(&[p1, p2, p3, p4, p5, p6], None)
        .unwrap()
        .vertices_indices();
    assert_eq!(i.len(), 8 * 3);
}

#[test]
fn octahedron_translation_test() {
    let p1 = DVec3::new(1.0, 0.0, 0.0);
    let p2 = DVec3::new(0.0, 1.0, 0.0);
    let p3 = DVec3::new(0.0, 0.0, 1.0);
    let p4 = DVec3::new(-1.0, 0.0, 0.0);
    let p5 = DVec3::new(0.0, -1.0, 0.0);
    let p6 = DVec3::new(0.0, 0.0, -1.0);

    let points: Vec<_> = [p1, p2, p3, p4, p5, p6]
        .into_iter()
        .map(|p| p + DVec3::splat(10.0))
        .collect();
    let (_v, i) = ConvexHull::try_new(&points, None)
        .unwrap()
        .vertices_indices();
    assert_eq!(i.len(), 8 * 3);
}

#[test]
fn cube_test() {
    let p1 = DVec3::new(1.0, 1.0, 1.0);
    let p2 = DVec3::new(1.0, 1.0, -1.0);
    let p3 = DVec3::new(1.0, -1.0, 1.0);
    let p4 = DVec3::new(1.0, -1.0, -1.0);
    let p5 = DVec3::new(-1.0, 1.0, 1.0);
    let p6 = DVec3::new(-1.0, 1.0, -1.0);
    let p7 = DVec3::new(-1.0, -1.0, 1.0);
    let p8 = DVec3::new(-1.0, -1.0, -1.0);

    let (_v, i) = ConvexHull::try_new(&[p1, p2, p3, p4, p5, p6, p7, p8], None)
        .unwrap()
        .vertices_indices();
    assert_eq!(i.len(), 6 * 2 * 3);
}

#[test]
fn cube_volume_test() {
    let p1 = DVec3::new(2.0, 2.0, 2.0);
    let p2 = DVec3::new(2.0, 2.0, 0.0);
    let p3 = DVec3::new(2.0, 0.0, 2.0);
    let p4 = DVec3::new(2.0, 0.0, 0.0);
    let p5 = DVec3::new(0.0, 2.0, 2.0);
    let p6 = DVec3::new(0.0, 2.0, 0.0);
    let p7 = DVec3::new(0.0, 0.0, 2.0);
    let p8 = DVec3::new(0.0, 0.0, 0.0);

    let cube = ConvexHull::try_new(&[p1, p2, p3, p4, p5, p6, p7, p8], None).unwrap();
    assert_eq!(cube.volume(), 8.0);
}

// Heavy test (~ 0.75s)
#[test]
fn sphere_volume_test() {
    let points = sphere_points(50);
    let hull = ConvexHull::try_new(&points, None).unwrap();
    let volume = hull.volume();
    let expected_volume = 4.0 / 3.0 * std::f64::consts::PI;
    assert!(
        (volume - expected_volume).abs() < 0.1,
        "Expected {expected_volume}, got {volume}"
    );
}

#[test]
fn cube_support_point_test() {
    let p1 = DVec3::new(1.0, 1.0, 1.0);
    let p2 = DVec3::new(1.0, 1.0, 0.0);
    let p3 = DVec3::new(1.0, 0.0, 1.0);
    let p4 = DVec3::new(1.0, 0.0, 0.0);
    let p5 = DVec3::new(0.0, 1.0, 1.0);
    let p6 = DVec3::new(0.0, 1.0, 0.0);
    let p7 = DVec3::new(0.0, 0.0, 1.0);
    let p8 = DVec3::new(0.0, 0.0, 0.0);

    let cube = ConvexHull::try_new(&[p1, p2, p3, p4, p5, p6, p7, p8], None).unwrap();
    assert_eq!(cube.support_point(DVec3::splat(0.5)), p1);
}

#[test]
fn flat_test() {
    let p1 = DVec3::new(1.0, 1.0, 10.0);
    let p2 = DVec3::new(1.0, 1.0, 10.0);
    let p3 = DVec3::new(1.0, -1.0, 10.0);
    let p4 = DVec3::new(1.0, -1.0, 10.0);
    let p5 = DVec3::new(-1.0, 1.0, 10.0);
    let p6 = DVec3::new(-1.0, 1.0, 10.0);
    let p7 = DVec3::new(-1.0, -1.0, 10.0);
    let p8 = DVec3::new(-1.0, -1.0, 10.0);

    assert!(ConvexHull::try_new(&[p1, p2, p3, p4, p5, p6, p7, p8], None)
        .is_err_and(|err| err == ErrorKind::DegenerateInput(DegenerateInput::Coplanar)));
}

#[test]
fn line_test() {
    let points = (0..10)
        .map(|i| DVec3::new(i as f64, 1.0, 10.0))
        .collect::<Vec<_>>();
    assert!(ConvexHull::try_new(&points, None)
        .is_err_and(|err| err == ErrorKind::DegenerateInput(DegenerateInput::Collinear)));
}

#[test]
fn simplex_may_degenerate_test() {
    let points = vec![
        DVec3::new(1.0, 0.0, 1.0),
        DVec3::new(1.0, 1.0, 1.0),
        DVec3::new(2.0, 1.0, 0.0),
        DVec3::new(2.0, 1.0, 1.0),
        DVec3::new(2.0, 0.0, 1.0),
        DVec3::new(2.0, 0.0, 0.0),
        DVec3::new(1.0, 1.0, 2.0),
        DVec3::new(0.0, 1.0, 2.0),
        DVec3::new(0.0, 0.0, 2.0),
        DVec3::new(1.0, 0.0, 2.0),
    ];

    let (_v, _i) = ConvexHull::try_new(&points, None)
        .unwrap()
        .vertices_indices();
}

#[test]
fn simplex_may_degenerate_test_2() {
    let vertices = vec![
        DVec3::new(0., 0., 0.),
        DVec3::new(1., 0., 0.),
        DVec3::new(1., 0., 1.),
        DVec3::new(0., 0., 1.),
        DVec3::new(0., 1., 0.),
        DVec3::new(1., 1., 0.),
        DVec3::new(1., 1., 1.),
        DVec3::new(0., 1., 1.),
        DVec3::new(2., 1., 0.),
        DVec3::new(2., 1., 1.),
        DVec3::new(2., 0., 1.),
        DVec3::new(2., 0., 0.),
        DVec3::new(1., 1., 2.),
        DVec3::new(0., 1., 2.),
        DVec3::new(0., 0., 2.),
        DVec3::new(1., 0., 2.),
    ];

    let indices = [4, 5, 1, 11, 1, 5, 1, 11, 10, 10, 2, 1, 5, 8, 11];
    let points = indices.iter().map(|i| vertices[*i]).collect::<Vec<_>>();
    let (_v, _i) = ConvexHull::try_new(&points, None)
        .unwrap()
        .vertices_indices();
}

#[cfg(test)]
fn sphere_points(divisions: usize) -> Vec<DVec3> {
    #[inline]
    fn rot_z(point: DVec3, angle: f64) -> DVec3 {
        let e1 = angle.cos() * point[0] - angle.sin() * point[1];
        let e2 = angle.sin() * point[0] + angle.cos() * point[1];
        let e3 = point[2];
        DVec3::new(e1, e2, e3)
    }

    #[inline]
    fn rot_x(point: DVec3, angle: f64) -> DVec3 {
        let e1 = point[0];
        let e2 = angle.cos() * point[1] - angle.sin() * point[2];
        let e3 = angle.sin() * point[1] + angle.cos() * point[2];
        DVec3::new(e1, e2, e3)
    }

    let mut points = Vec::with_capacity(divisions * divisions);
    let unit_y = DVec3::Y;
    for step_x in 0..divisions {
        let angle_x = 2.0 * std::f64::consts::PI * (step_x as f64 / divisions as f64);
        let p = rot_x(unit_y, angle_x);
        for step_z in 0..divisions {
            let angle_z = 2.0 * std::f64::consts::PI * (step_z as f64 / divisions as f64);
            let p = rot_z(p, angle_z);
            points.push(p);
        }
    }

    points
}

#[test]
fn sphere_test() {
    let points = sphere_points(10);
    let (_v, _i) = ConvexHull::try_new(&points, None)
        .unwrap()
        .vertices_indices();
}

#[test]
fn big_sphere_test() {
    let points = sphere_points(80);
    let (_v, _i) = ConvexHull::try_new(&points, None)
        .unwrap()
        .vertices_indices();
}

#[test]
fn very_big_sphere_test() {
    let points = sphere_points(400);
    let (_v, _i) = ConvexHull::try_new(&points, Some(300))
        .unwrap()
        .vertices_indices();
}

/// Useful for fuzzing and profiling
/// creates a sea-urchin like point cloud
/// with points distributed arbitrarily within a sphere
#[test]
fn heavy_sea_urchin_test() {
    use rand::prelude::{Distribution, SeedableRng, SliceRandom};

    // increase this to ~1000 to gather more samples for a sampling profiler
    let iterations = 1;

    for s in 0..iterations {
        let mut rng = rand::rngs::StdRng::seed_from_u64(s);
        let dist = rand::distr::StandardUniform;

        fn rot_z(point: DVec3, angle: f64) -> DVec3 {
            let e1 = angle.cos() * point[0] - angle.sin() * point[1];
            let e2 = angle.sin() * point[0] + angle.cos() * point[1];
            let e3 = point[2];
            DVec3::new(e1, e2, e3)
        }

        fn rot_x(point: DVec3, angle: f64) -> DVec3 {
            let e1 = point[0];
            let e2 = angle.cos() * point[1] - angle.sin() * point[2];
            let e3 = angle.sin() * point[1] + angle.cos() * point[2];
            DVec3::new(e1, e2, e3)
        }

        let mut points = Vec::new();
        let dev = 100;
        let unit_y = DVec3::Y;
        for step_x in 0..dev {
            let angle_x = 2.0 * std::f64::consts::PI * (step_x as f64 / dev as f64);
            let p = rot_x(unit_y, angle_x);
            for step_z in 0..dev {
                let angle_z = 2.0 * std::f64::consts::PI * (step_z as f64 / dev as f64);
                let p = rot_z(p, angle_z);
                let rand_offset: f64 = dist.sample(&mut rng);
                points.push(p * rand_offset);
            }
        }

        points.shuffle(&mut rng);
        let (_v, _i) = ConvexHull::try_new(&points, None)
            .unwrap()
            .vertices_indices();
    }
}

#[test]
fn test_chull_errors() {
    // Empty
    assert_eq!(
        ConvexHull::try_new(&[], None).unwrap_err(),
        ErrorKind::Empty
    );

    // Less then three points
    assert_eq!(
        ConvexHull::try_new(&[DVec3{ x: 0.0, y: 0.0, z: 0.0 }, DVec3{ x: 1.0, y: 1.0, z: 1.0 }], None).unwrap_err(),
        ErrorKind::Degenerated
    );

    // Collinear
    assert_eq!(
        ConvexHull::init_tetrahedron(&[DVec3{ x: 0.0, y: 0.0, z: 0.0 }, DVec3{ x: 10.0, y: 10.0, z: 10.0 }]).unwrap_err(),
        ErrorKind::DegenerateInput(DegenerateInput::Collinear)
    );

    // Coplanar
    assert_eq!(
        ConvexHull::init_tetrahedron(&[DVec3{ x: 0.0, y: 0.0, z: 5.0 }, DVec3{ x: 10.0, y: 13.0, z: 10.0 }, DVec3{ x: -10.1, y: 13.0, z: 10.0 }]).unwrap_err(),
        ErrorKind::DegenerateInput(DegenerateInput::Coplanar)
    );
}
