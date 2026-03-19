use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ocps_core::pipeline::types::{EditRecipe, RgbImage16};
use ocps_core::pipeline::ImageProcessor;

fn create_synthetic_image(width: u32, height: u32) -> RgbImage16 {
    let size = (width * height * 3) as usize;
    let data = vec![32768u16; size]; // Mid-gray
    RgbImage16::from_data(width, height, data)
}

fn bench_pipeline_4mp(c: &mut Criterion) {
    // Create 2000x2000 synthetic image (4MP)
    let img = create_synthetic_image(2000, 2000);
    let mut recipe = EditRecipe::default();
    recipe.exposure = 0.5;
    recipe.contrast = 20;
    recipe.saturation = 15;

    c.bench_function("process_4mp_default", |b| {
        b.iter(|| ImageProcessor::process(black_box(&img), black_box(&recipe)))
    });
}

fn bench_pipeline_1mp(c: &mut Criterion) {
    // Create 1000x1000 synthetic image (1MP)
    let img = create_synthetic_image(1000, 1000);
    let mut recipe = EditRecipe::default();
    recipe.exposure = 0.5;

    c.bench_function("process_1mp_exposure_only", |b| {
        b.iter(|| ImageProcessor::process(black_box(&img), black_box(&recipe)))
    });
}

fn bench_histogram(c: &mut Criterion) {
    use ocps_core::histogram::Histogram;

    let data = vec![128u8; 2000 * 2000 * 3];

    c.bench_function("histogram_4mp", |b| {
        b.iter(|| Histogram::from_rgb8(black_box(&data), 2000, 2000))
    });
}

fn bench_white_balance(c: &mut Criterion) {
    use ocps_core::pipeline::process;

    let img = create_synthetic_image(1000, 1000);
    let mut recipe = EditRecipe::default();
    recipe.temperature = 6500;
    recipe.tint = 10;

    c.bench_function("white_balance_1mp", |b| {
        b.iter(|| {
            let mut output = img.clone();
            process::apply_white_balance(black_box(&mut output), 6500, 10);
        })
    });
}

criterion_group!(benches, bench_pipeline_4mp, bench_pipeline_1mp, bench_histogram, bench_white_balance);
criterion_main!(benches);
