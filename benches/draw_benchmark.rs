use glam::Vec3A;
use tinyrenderer::objloader::{Mesh, load_obj};
use tinyrenderer::util::{draw_mesh, draw_mesh_glam};
use tinyrenderer::renderer::Renderer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("draw_filled", |b| {
        const WIDTH: i32 = 800;
        const HEIGHT: i32 = 800;
        let mut r = Renderer::new(WIDTH, HEIGHT);
        let mesh = load_obj("obj/african_head.obj");
        
        b.iter(|| draw_mesh(&mut r, &mesh, true))
    });
    c.bench_function("draw_filled_glam", |b| {
        const WIDTH: i32 = 800;
        const HEIGHT: i32 = 800;
        let mut r = Renderer::new(WIDTH, HEIGHT);
        let mesh: Mesh<Vec3A> = load_obj("obj/african_head.obj");
        
        b.iter(|| draw_mesh_glam(&mut r, &mesh, true))
    });
    c.bench_function("draw_wireframe", |b| {
        const WIDTH: i32 = 800;
        const HEIGHT: i32 = 800;
        let mut r = Renderer::new(WIDTH, HEIGHT);
        let mesh = load_obj("obj/african_head.obj");
        
        b.iter(|| draw_mesh(&mut r, &mesh, false))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
