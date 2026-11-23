pub type V = [f32; 3];
pub fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: V, s: f32) -> V {
    a.map(|v| v * s)
}
pub fn dot(a: V, b: V) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
pub fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn length(a: V) -> f32 {
    dot(a, a).sqrt()
}
pub fn unit(a: V) -> V {
    let n = length(a);
    if n > 1e-12 {
        mul(a, 1. / n)
    } else {
        [0., 0., 1.]
    }
}
pub fn mix(a: V, b: V, t: f32) -> V {
    add(mul(a, 1. - t), mul(b, t))
}
/// Stable orthonormal frame; use the preceding side to avoid spinning around bends.
pub fn frame(tangent: V, previous_side: V) -> (V, V) {
    let t = unit(tangent);
    let projected = sub(previous_side, mul(t, dot(previous_side, t)));
    let side = if length(projected) > 0.01 {
        unit(projected)
    } else {
        unit(cross([0., 1., 0.], t))
    };
    (side, unit(cross(t, side)))
}
