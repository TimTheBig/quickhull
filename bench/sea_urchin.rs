use criterion::{criterion_group, criterion_main, Criterion};
use rand::{prelude::Distribution, seq::SliceRandom};
use glam::DVec3;
use quickhull::ConvexHull;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::rng();

    c.bench_function("heavy_sea_urchin", |bencher| {
        bencher.iter(|| {
            criterion::black_box({
                let dist = rand::distr::StandardUniform;

                let dev: usize = 100;
                let mut points = Vec::with_capacity(dev.pow(2));

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
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// creates a sea-urchin like point cloud
// with points distributed arbitrarily within a sphere

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
