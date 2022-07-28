# fractal.rs

A multithreaded fractal renderer in Rust

[![Website shields.io](https://img.shields.io/website-up-down-green-red/http/shields.io.svg?style=flat-square)](https://bertrandbev.github.io/fractal/)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg?style=flat-square)](https://raw.githubusercontent.com/BertrandBev/fractal/master/LICENSE)
[![Made with emscripten](https://img.shields.io/badge/Made%20width-druid-purple.svg?style=flat-square)](https://github.com/linebender/druid)

<img src="https://raw.githubusercontent.com/BertrandBev/fractal/master/doc/images/demo.gif" width="100%">

## Online release

The live wasm-compiled release is accessible [here](https://bertrandbev.github.io/fractal/). Due to some rust wasm compiler limitations, that web version is single threaded and therefore slower than native desktop 

## How to run

Navigate to the cloned folder and execute

```bash
cargo run --release
```

## the Mandelbrot set

The [Mandelbrot set](https://en.wikipedia.org/wiki/Mandelbrot_set) is the set of complex numbers $c$ for which the iterated sequence $z_{n + 1} = z_{n^2} + c; z_0 = 0$ does not diverge in modulus to infinity, i.e. remains bounded.

Computationally, for every pixel of the rendering area, the above equation is iterated a fixed number of times, and a pixel color is associated with the rate of divergence of the sequence. Typically the black color is associated with a converging point (whose norm stays within some fixed threshold after the iteration count), and a coloring scheme is chosen to clarify the renderings

### Optimisations

Multiple optimisations can be implemented to speed up the sequence interation. Choosing $z = x + i y$ and $c = x_0 + i y_0$, we can expand

$$z^2 = x^2 - y^2 + 2 i x y$$
$$x = Re(x^2 + c) = x^2 - y^2 + x_0$$
$$y = Im(x^2 + c) = 2 x y + y_0$$

Further, we can pre-compute $z^2$ since it will be used in both the iterations and the exit condition checks. A pixel iteration can then be written that way

```rust
let mut z = Complex::zero();
let mut z_sqr = Complex::zero();
let mut iter = 0;

loop {
    z.i = 2. * z.r * z.i + c.i;
    z.r = z_sqr.r - z_sqr.i + c.r;
    z_sqr.r = z.r * z.r;
    z_sqr.i = z.i * z.i;
    iter += 1;
    if iter >= max_iter || z_sqr.r + z_sqr.i > escape_radius_sqr {
      break;
    }
}
```

### Coloring


## Mutlistage multithreaded renderer

For deep zoom levels, the number of iterations has to be increased to maintain an appropriate level of details, which makes for slower rendering time and reduces the exploration smoothness. Multistage rendering works by splitting up a rendering task in $n$ stages, each of which doubles the rendering area up to the screen size. The time required to render the $n^{th}$ stage is $\frac{1}{2^n}$ the time it takes to render the full screen size. Since:

$$\sum_{k=1}^{\inf} \frac{1}{2^k} = 1$$

Regardless of the number of stages, the rendering time is bounded to double the rendering time of the final stage.

The fractal rendering is trivially parallelizable, since every pixel color can be computed independently of each other. The renderer maintains a thread pool sharing the work at every stage by rendering a fraction of the stage's pixels. Every animation frame, the canvas pauses the threads execution and interleaves their pixel buffers onto the canvas buffer to smoothly display progress.

<img src="https://raw.githubusercontent.com/BertrandBev/fractal/master/doc/images/renderer.gif" width="60%">

## Limitations

The iterator uses `f64` to compute the complex series, which leads to numerical precision issues for deep zoom levels. Typically after a zoom multiplier of $10^{15}$, numerical limits start to degrade the renders

<img src="https://raw.githubusercontent.com/BertrandBev/fractal/master/doc/images/f64.png" width="60%">

## Web assembly compilation

The renderer can be compiled to [wasm](https://www.rust-lang.org/what/wasm) by installing and running [wasm-pack](https://github.com/rustwasm/wasm-pack)

```bash
wasm-pack build --target web --release
```

The `wasm` compiler doesn't support yet the `std::thread` library, therefore a single thread is used on web. For other targets, the thread count is set automatically to the cpu count

