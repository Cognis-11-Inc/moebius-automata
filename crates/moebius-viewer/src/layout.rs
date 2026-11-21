use std::collections::BTreeMap;

#[cfg(test)]
use moebius_core::TrailPoint;
use moebius_core::{Dims, Site};

pub const SPACING: f32 = 0.3;
pub const FILAMENT_WINDOW: u64 = 18;

pub fn site_to_view(site: Site, dims: Dims) -> [f32; 3] {
    [
        (site.x as f32 - (dims.width as f32 - 1.0) * 0.5) * SPACING,
        (site.y as f32 - (dims.height as f32 - 1.0) * 0.5) * SPACING,
        (site.z as f32 - (dims.depth as f32 - 1.0) * 0.5) * SPACING,
    ]
}

/// Vertical field of view. Wide enough that the tank looms and the face still fits.
pub const CAMERA_FOV: f32 = 50.0 * std::f32::consts::PI / 180.0;

/// Camera distance that keeps the tank face inside the vertical field.
pub fn stage_distance(dims: Dims) -> f32 {
    let span = dims
        .width
        .max(dims.height)
        .max(dims.depth)
        .saturating_sub(1) as f32
        * SPACING;
    let half = (span * 0.5).max(SPACING);
    half / (CAMERA_FOV * 0.5).tan() * 1.12
}

/// Power-of-two stride whose block count is at most 64.
/// Fewer, larger blocks keep the fluid one flowing mass.
/// The HUD reports the stride. This does not change the snapshot.
pub fn display_stride(dims: Dims) -> u32 {
    for stride in [1, 2, 4, 8, 16, 32, 64] {
        let count = u64::from(dims.width.div_ceil(stride))
            * u64::from(dims.height.div_ceil(stride))
            * u64::from(dims.depth.div_ceil(stride));
        if count <= 64 {
            return stride;
        }
    }
    64
}

/// View position of a cell at a fractional tick.
/// A periodic wrap is not stretched into a line.
pub fn position_at(points: &[(u64, Site)], tick: f32, dims: Dims) -> Option<[f32; 3]> {
    if points.is_empty() {
        return None;
    }
    if tick <= points[0].0 as f32 {
        return Some(site_to_view(points[0].1, dims));
    }
    let last = points.len() - 1;
    if tick >= points[last].0 as f32 {
        return Some(site_to_view(points[last].1, dims));
    }
    let mut lo = 0usize;
    let mut hi = last;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if (points[mid].0 as f32) <= tick {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (tick_a, site_a) = points[lo];
    let (tick_b, site_b) = points[hi];
    let from = site_to_view(site_a, dims);
    if tick_b != tick_a + 1
        || site_a.x.abs_diff(site_b.x) > 1
        || site_a.y.abs_diff(site_b.y) > 1
        || site_a.z.abs_diff(site_b.z) > 1
    {
        return Some(from);
    }
    let t = tick - tick_a as f32;
    let to = site_to_view(site_b, dims);
    Some([
        from[0] + (to[0] - from[0]) * t,
        from[1] + (to[1] - from[1]) * t,
        from[2] + (to[2] - from[2]) * t,
    ])
}

/// Movement threads for the cells whose paths are already indexed.
/// A periodic wrap is omitted. The result is capped at 4096 segments.
pub fn path_segments(
    paths: &BTreeMap<u64, Vec<(u64, Site)>>,
    displayed_tick: u64,
    window: u64,
    dims: Dims,
) -> Vec<([f32; 3], [f32; 3])> {
    let mut segments = Vec::new();
    let start = displayed_tick.saturating_sub(window);
    for points in paths.values() {
        if points.len() < 2 {
            continue;
        }
        let mut index = points.partition_point(|(tick, _)| *tick < start);
        while index + 1 < points.len() {
            let (tick_a, site_a) = points[index];
            if tick_a > displayed_tick {
                break;
            }
            let (tick_b, site_b) = points[index + 1];
            index += 1;
            if tick_b > displayed_tick {
                break;
            }
            if tick_b != tick_a + 1 || site_a == site_b {
                continue;
            }
            if site_a.x.abs_diff(site_b.x) > 1
                || site_a.y.abs_diff(site_b.y) > 1
                || site_a.z.abs_diff(site_b.z) > 1
            {
                continue;
            }
            segments.push((site_to_view(site_a, dims), site_to_view(site_b, dims)));
            if segments.len() == 4_096 {
                return segments;
            }
        }
    }
    segments
}

#[cfg(test)]
fn filament_segments(
    trails: &[TrailPoint],
    displayed_tick: u64,
    window: u64,
    dims: Dims,
) -> Vec<([f32; 3], [f32; 3])> {
    let mut by_id: BTreeMap<u64, Vec<(u64, Site)>> = BTreeMap::new();
    for point in trails {
        if point.tick <= displayed_tick && displayed_tick - point.tick <= window {
            by_id
                .entry(point.id)
                .or_default()
                .push((point.tick, point.site()));
        }
    }
    let mut segments = Vec::new();
    for points in by_id.values_mut() {
        points.sort_by_key(|(tick, _)| *tick);
        points.dedup_by_key(|(tick, site)| (*tick, *site));
        for pair in points.windows(2) {
            let (tick_a, site_a) = pair[0];
            let (tick_b, site_b) = pair[1];
            if tick_b != tick_a + 1 || site_a == site_b {
                continue;
            }
            if site_a.x.abs_diff(site_b.x) > 1
                || site_a.y.abs_diff(site_b.y) > 1
                || site_a.z.abs_diff(site_b.z) > 1
            {
                continue;
            }
            segments.push((site_to_view(site_a, dims), site_to_view(site_b, dims)));
        }
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_space_orders_axes_and_centers_an_even_grid() {
        let dims = Dims {
            width: 4,
            height: 4,
            depth: 4,
        };
        let low = site_to_view(Site { x: 0, y: 0, z: 0 }, dims);
        let high = site_to_view(Site { x: 3, y: 2, z: 1 }, dims);
        assert!(low[0] < 0.0 && high[0] > 0.0);
        assert!(low[0] < high[0]);
        assert!(low.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn stride_caps_the_drawn_blocks() {
        let big = Dims {
            width: 64,
            height: 64,
            depth: 64,
        };
        assert_eq!(display_stride(big), 16);
        let hall = Dims {
            width: 128,
            height: 128,
            depth: 128,
        };
        assert_eq!(display_stride(hall), 32);
        let small = Dims {
            width: 4,
            height: 4,
            depth: 4,
        };
        assert_eq!(display_stride(small), 1);
    }

    #[test]
    fn stage_distance_keeps_the_tank_face_in_frame() {
        let dims = Dims {
            width: 64,
            height: 64,
            depth: 64,
        };
        let distance = stage_distance(dims);
        let half = 63.0 * SPACING * 0.5;
        let visible = distance * (CAMERA_FOV * 0.5).tan();
        assert!(visible > half);
        assert!(distance.is_finite());
    }

    #[test]
    fn path_segments_follow_one_step_and_skip_a_wrap() {
        let dims = Dims {
            width: 8,
            height: 4,
            depth: 4,
        };
        let mut paths = BTreeMap::new();
        paths.insert(
            1,
            vec![
                (0, Site { x: 0, y: 0, z: 0 }),
                (1, Site { x: 1, y: 0, z: 0 }),
                (2, Site { x: 7, y: 0, z: 0 }),
            ],
        );
        let shown = path_segments(&paths, 2, 18, dims);
        assert_eq!(shown.len(), 1);
        let future = path_segments(&paths, 0, 18, dims);
        assert!(future.is_empty());
    }

    #[test]
    fn filaments_skip_wraps_and_future_ticks() {
        let dims = Dims {
            width: 8,
            height: 4,
            depth: 4,
        };
        let trails = [
            TrailPoint {
                tick: 0,
                id: 1,
                x: 0,
                y: 0,
                z: 0,
            },
            TrailPoint {
                tick: 1,
                id: 1,
                x: 1,
                y: 0,
                z: 0,
            },
            TrailPoint {
                tick: 2,
                id: 1,
                x: 0,
                y: 0,
                z: 0,
            },
            TrailPoint {
                tick: 3,
                id: 1,
                x: 2,
                y: 0,
                z: 0,
            },
        ];
        let shown = filament_segments(&trails, 1, 24, dims);
        assert_eq!(shown.len(), 1);
        let wrapped = filament_segments(
            &[
                TrailPoint {
                    tick: 0,
                    id: 2,
                    x: 0,
                    y: 1,
                    z: 1,
                },
                TrailPoint {
                    tick: 1,
                    id: 2,
                    x: 7,
                    y: 1,
                    z: 1,
                },
            ],
            1,
            24,
            dims,
        );
        assert!(wrapped.is_empty());
    }

    #[test]
    fn position_glides_one_step_and_holds_across_a_wrap() {
        let dims = Dims {
            width: 8,
            height: 4,
            depth: 4,
        };
        let points = [
            (0u64, Site { x: 1, y: 1, z: 1 }),
            (1, Site { x: 2, y: 1, z: 1 }),
        ];
        let mid = position_at(&points, 0.5, dims).unwrap();
        let a = site_to_view(points[0].1, dims);
        let b = site_to_view(points[1].1, dims);
        assert!((mid[0] - (a[0] + b[0]) * 0.5).abs() < 1.0e-4);
        let wrapped = [
            (0u64, Site { x: 0, y: 1, z: 1 }),
            (1, Site { x: 7, y: 1, z: 1 }),
        ];
        assert_eq!(
            position_at(&wrapped, 0.5, dims).unwrap(),
            site_to_view(wrapped[0].1, dims)
        );
    }
}
