use std::collections::BTreeMap;

use moebius_core::{Dims, Site, TrailPoint};

pub const SPACING: f32 = 0.16;
pub const FILAMENT_WINDOW: u64 = 24;

pub fn site_to_view(site: Site, dims: Dims) -> [f32; 3] {
    [
        (site.x as f32 - (dims.width as f32 - 1.0) * 0.5) * SPACING,
        (site.y as f32 - (dims.height as f32 - 1.0) * 0.5) * SPACING,
        (site.z as f32 - (dims.depth as f32 - 1.0) * 0.5) * SPACING,
    ]
}

/// Smallest power-of-two stride whose block count is at most 512.
/// The HUD reports the stride. This does not change the snapshot.
pub fn display_stride(dims: Dims) -> u32 {
    for stride in [1, 2, 4, 8, 16, 32, 64] {
        let count = u64::from(dims.width.div_ceil(stride))
            * u64::from(dims.height.div_ceil(stride))
            * u64::from(dims.depth.div_ceil(stride));
        if count <= 512 {
            return stride;
        }
    }
    64
}

pub fn filament_segments(
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
        assert_eq!(display_stride(big), 8);
        let small = Dims {
            width: 4,
            height: 4,
            depth: 4,
        };
        assert_eq!(display_stride(small), 1);
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
}
