#![no_main]

use quickhull::ConvexHull;
use glam::DVec3;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|array: [f64; 14]| {
    if array.iter().any(|num| num.is_nan() || num.is_subnormal() || *num < 13e-27 || *num > 7e27) {
        return;
    }

    let dev: usize = 100;
    let mut points = Vec::with_capacity(10_000);

    let unit_y = DVec3::Y;
    for step_x in 0..dev {
        let angle_x = 2.0 * std::f64::consts::PI * (step_x as f64 / dev as f64);
        let p = rot_x(unit_y, angle_x);
        for step_z in 0..dev {
            let angle_z = 2.0 * std::f64::consts::PI * (step_z as f64 / dev as f64);
            let p = rot_z(p, angle_z);
            let rand_offset: f64 = array[(step_x + step_z) % 14];
            points.push(p * rand_offset);
        }
    }

    let (_, _) = ConvexHull::try_new(&points, Some(10_000))
        .unwrap()
        .vertices_indices();

    let mut points = array.windows(3).map(|val| DVec3::new(val[0], val[1], val[2])).collect::<Vec<_>>();
    points[0].x += 1.0 * f64::EPSILON;
    points[1].y += 1.0 * f64::EPSILON;
    points[2].z += 2.0 * f64::EPSILON;
    let result = ConvexHull::try_new(&points, None);
});

#[no_mangle]
fn rot_z(point: DVec3, angle: f64) -> DVec3 {
    let e1 = angle.cos() * point[0] - angle.sin() * point[1];
    let e2 = angle.sin() * point[0] + angle.cos() * point[1];
    let e3 = point[2];
    DVec3::new(e1, e2, e3)
}

#[no_mangle]
fn rot_x(point: DVec3, angle: f64) -> DVec3 {
    let e1 = point[0];
    let e2 = angle.cos() * point[1] - angle.sin() * point[2];
    let e3 = angle.sin() * point[1] + angle.cos() * point[2];
    DVec3::new(e1, e2, e3)
}
