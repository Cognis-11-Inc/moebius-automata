//! Explicit VII research, reachable only through collision-fixture tools.
//! These results have no admission path into ShipRuntime.
use super::*;
use serde_json::{Value, json};
use std::path::Path;

pub fn variants() -> Vec<(String, AnatomySpec)> {
    // The runtime supplies finite food storage to the showcase. Preserve that
    // canonical augmentation rather than replacing it with an unregistered body.
    let original = ShipRuntime::habitat_simulation(AnatomySpec::showcase(77), 77, 256)
        .expect("reference habitat")
        .state()
        .config
        .definitions[&1]
        .clone();
    [
        "original",
        "none",
        "one-short",
        "two-short",
        "long",
        "thin",
        "phase",
    ]
    .into_iter()
    .map(|name| {
        let mut spec = original.clone();
        let mut count = 0;
        spec.components.retain_mut(|c| {
            let crate::anatomy::Shape::Flagellum {
                length,
                radius,
                phase,
                ..
            } = &mut c.shape
            else {
                return true;
            };
            count += 1;
            match name {
                "none" => return false,
                "one-short" | "two-short" => {
                    *length = 0.30;
                    return count <= if name == "one-short" { 1 } else { 2 };
                }
                "long" => *length = (*length * 1.5).min(3.),
                "thin" => *radius = (*radius * 0.5).max(0.008),
                "phase" => *phase += 0.7,
                _ => {}
            }
            true
        });
        spec.validate().expect("supported experimental anatomy");
        (name.into(), spec)
    })
    .collect()
}

fn write(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn pose(x: f32, z: f32, yaw: f32) -> RigidPose {
    RigidPose {
        translation: [
            (x * SCALE as f32).round() as i64,
            0,
            (z * SCALE as f32).round() as i64,
        ],
        yaw_sin: (yaw.sin() * SCALE as f32).round() as i64,
        yaw_cos: (yaw.cos() * SCALE as f32).round() as i64,
        axial_roll: 0,
    }
}
fn set_pose(world: &mut WorldSnapshot, id: u64, p: RigidPose) -> Result<(), MorphError> {
    world.instances.get_mut(&id).unwrap().position = Position::from_absolute(p.translation)?;
    world.instances.get_mut(&id).unwrap().exposure =
        crate::runtime::exposure(&world.config, Position::from_absolute(p.translation)?)?;
    if world
        .config
        .construction
        .as_ref()
        .is_some_and(|r| r.organism == id)
        && let Some(steering) = &mut world.steering
    {
        steering.rigid = p;
    }
    if let Some(agent) = world.habitat.as_mut().unwrap().agents.get_mut(&id) {
        agent.rigid = p;
        agent.heading = [p.yaw_sin, 0, p.yaw_cos];
        agent.travel = None;
    }
    Ok(())
}
fn diagnostics(world: &WorldSnapshot, cache: &mut CollisionCache) -> Result<Value, MorphError> {
    let ids = world
        .habitat
        .as_ref()
        .unwrap()
        .agents
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let mut contacts = Vec::new();
    let mut workspace = ContactWorkspace::default();
    for (i, &a) in ids.iter().enumerate() {
        for &b in &ids[i + 1..] {
            let ga = cache.body(world, a)?;
            let gb = cache.body(world, b)?;
            let origin = Position::from_absolute(world.body_pose(a)?.translation)?;
            let pa = motion(world.body_pose(a)?, world.body_pose(a)?, origin)?.start;
            let pb = motion(world.body_pose(b)?, world.body_pose(b)?, origin)?.start;
            for (aid, part_a) in &ga.parts {
                for (bid, part_b) in &gb.parts {
                    if let Some(c) =
                        cached_world_contact(&mut workspace, &pa, part_a, &pb, part_b, 0.001)
                            .map_err(|_| MorphError("diagnostic contact unsupported".into()))?
                    {
                        contacts.push(json!({"a":a,"b":b,"component_a":aid,"component_b":bid,"distance":c.dist,"point1":c.point1.to_array(),"normal1":c.normal1.to_array(),"origin":origin}));
                    }
                }
            }
        }
    }
    Ok(json!(contacts))
}
/// Independent five-pose contact-distance observations along a supplied route.
/// A certificate is scoped to all other bodies and currently built pipe chunks;
/// no physical impossibility or complete configuration-space claim follows.
fn escape_samples(
    world: &WorldSnapshot,
    proposals: &BTreeMap<u64, RigidPose>,
    cache: &mut CollisionCache,
) -> Result<Value, MorphError> {
    let mut rows = Vec::new();
    for (&id, &to) in proposals {
        let a = cache.body(world, id)?;
        let from = world.body_pose(id)?;
        let origin = Position::from_absolute(from.translation)?;
        let am = motion(from, to, origin)?;
        let mut targets = Vec::new();
        for &other in world.instances.keys().filter(|other| **other != id) {
            let b = cache.body(world, other)?;
            let bp = world.body_pose(other)?;
            targets.push((format!("body:{other}"), b, bp));
        }
        if let Some(assembly) = &world.assembly {
            for f in assembly.filaments.values() {
                for chunk in 0..f.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES) {
                    let (at, b) = cache.pipe(world, f.id, chunk)?;
                    targets.push((
                        format!("pipe:{}:{chunk}", f.id),
                        Arc::new(b),
                        RigidPose {
                            translation: at.absolute()?,
                            ..RigidPose::identity()
                        },
                    ));
                }
            }
        }
        for (target, b, bp) in targets {
            let bm = motion(bp, bp, origin)?;
            let mut workspace = ContactWorkspace::default();
            let mut depths = Vec::new();
            for t in [0., 0.25, 0.5, 0.75, 1.] {
                let c = cached_world_contact(
                    &mut workspace,
                    &am.position_at_time(t),
                    &a,
                    &bm.start,
                    &b,
                    0.001,
                )
                .map_err(|_| MorphError("escape probe unsupported".into()))?;
                depths.push(c.map(|c| c.dist));
            }
            let mut previous = depths[0].unwrap_or(0.);
            let mut nondeepening = true;
            for depth in &depths[1..] {
                let depth = depth.unwrap_or(0.);
                if depth + 0.00001 < previous {
                    nondeepening = false;
                }
                previous = depth;
            }
            rows.push(json!({"body":id,"target":target,"depths":depths,"sampled_nondeepening":nondeepening,"endpoint_clear":depths[4].is_none()}));
        }
    }
    Ok(json!(rows))
}

fn rss() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|s| {
            s.strip_prefix("VmRSS:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}
fn scene(world: &WorldSnapshot, cache: &mut CollisionCache) -> Result<Value, MorphError> {
    let mut parts = Vec::new();
    for &id in world.habitat.as_ref().unwrap().agents.keys() {
        let g = cache.body(world, id)?;
        let p = world.body_pose(id)?;
        for (component, part) in &g.parts {
            parts.push(json!({"body":id,"component":component,"rigid":p,"vertices":part.vertices.iter().map(|v|v.to_array()).collect::<Vec<_>>()}));
        }
    }
    Ok(json!(parts))
}
/// Same initial world/proposals for every anatomy. Custom proposals are experimental inputs.
/// The matched family invokes only canonical resolution, not a second material runtime.
pub fn behaviour(
    out: &Path,
    ticks: usize,
    memory: fn() -> [usize; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    require((16..=128).contains(&ticks), "research tick budget")?;
    std::fs::create_dir(out)?;
    let base =
        ShipRuntime::habitat_colony(AnatomySpec::showcase(77), 77, 256, 2, Some(4000))?.snapshot();
    let ids = base
        .habitat
        .as_ref()
        .unwrap()
        .agents
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let mut paid = ShipRuntime::restore(base.clone())?;
    paid.advance(16)?;
    paid.validate()?;
    paid.snapshot()
        .save(&out.join("paid-structure-source.json"))?;
    let pi = std::f32::consts::PI;
    let scenarios = [
        ("isolated", pose(0., 0., 0.), pose(12., 0., 0.)),
        ("head-on", pose(0., -1.35, 0.), pose(0., 1.35, pi)),
        ("oblique", pose(-0.9, -1.1, 0.6), pose(0.9, 1.1, pi + 0.6)),
        ("passing", pose(-0.7, -0.8, 0.), pose(0.7, 0.8, pi)),
        ("tail", pose(0., 0., 0.), pose(0.4, -2.2, 0.)),
        ("appendages", pose(-0.6, 0., 0.), pose(0.6, 0., pi)),
        ("secreting-near", pose(-0.7, 0., 0.), pose(0.7, 0., 0.)),
        ("existing-pipe", pose(0., 0., 0.), pose(12., 0., 0.)),
        ("pipe-opening", pose(0., 0., 0.), pose(12., 0., 0.)),
        ("wet-crossing", pose(0., 0., pi * 0.5), pose(12., 0., 0.)),
        ("escape", pose(-0.5, 0., 0.), pose(0.5, 0., 0.)),
        ("congestion", pose(-0.6, 0., 0.), pose(0.6, 0., pi)),
    ];
    for (name, spec) in variants() {
        write(&out.join(format!("anatomy-{name}.json")), &spec)?;
        for (scenario, pa, pb) in scenarios {
            for family in ["natural", "matched"] {
                let dir = out.join(format!("{name}-{scenario}-{family}"));
                std::fs::create_dir(&dir)?;
                let structural = matches!(
                    scenario,
                    "existing-pipe" | "pipe-opening" | "wet-crossing" | "congestion"
                );
                let mut initial = if structural {
                    paid.snapshot()
                } else {
                    base.clone()
                };
                initial.config.definitions.insert(1, spec.clone());
                initial.habitat.as_mut().unwrap().rules.reproduction = false;
                // A controlled closed encounter: retain paid food bodies remotely
                // and prevent additional germination changing encounter inputs.
                let food_count = initial
                    .instances
                    .values()
                    .filter(|i| i.definition == BLOOD_SAC)
                    .count();
                initial.habitat.as_mut().unwrap().rules.max_sacs = food_count.max(1);
                initial.habitat.as_mut().unwrap().sacs.clear();
                initial.solids.clear();
                let others = initial
                    .instances
                    .keys()
                    .filter(|id| !ids.contains(id))
                    .copied()
                    .collect::<Vec<_>>();
                for (n, id) in others.into_iter().enumerate() {
                    set_pose(&mut initial, id, pose(100. + n as f32 * 4., 100., 0.))?;
                }
                let (mut pa, mut pb) = (pa, pb);
                if structural && scenario != "congestion" {
                    let anchor = initial
                        .assembly
                        .as_ref()
                        .unwrap()
                        .filaments
                        .values()
                        .next()
                        .and_then(|f| f.nodes.first())
                        .map(|n| n.position.absolute())
                        .transpose()?;
                    if let Some(at) = anchor {
                        pa.translation = at;
                        pa.translation[0] += if scenario == "pipe-opening" {
                            0
                        } else {
                            700_000
                        };
                        pb.translation = at;
                        pb.translation[0] += 12 * SCALE;
                    }
                }
                set_pose(&mut initial, ids[0], pa)?;
                set_pose(&mut initial, ids[1], pb)?;
                initial.save(&dir.join("initial.json"))?;
                let mut canonical_rows = None;
                for repeat in 0..2 {
                    let mut w = ShipRuntime::restore(initial.clone())?;
                    let mut state = initial.clone();
                    let mut cache = CollisionCache::default();
                    let mut rows = Vec::new();
                    let initial_scene = scene(&state, &mut cache)?;
                    if repeat == 0 {
                        write(&dir.join("scene.json"), &initial_scene)?;
                    }
                    for tick in 0..ticks {
                        let before = if family == "natural" {
                            w.snapshot()
                        } else {
                            state.clone()
                        };
                        let events_before = before.events.len();
                        let mut proposed = BTreeMap::new();
                        let mut escape = json!(null);
                        let allocation_before = memory();
                        let started = Instant::now();
                        let profile;
                        if family == "natural" {
                            w.tick()?;
                            state = w.snapshot();
                            profile = w.collision_profile();
                        } else {
                            let mut next = before.clone();
                            let mut plans = BTreeMap::new();
                            for (n, &id) in ids.iter().enumerate() {
                                let mut p = before.body_pose(id)?;
                                let mut delta = [p.yaw_sin / 25, 0, p.yaw_cos / 25];
                                if scenario == "isolated" && n == 1 {
                                    delta = [0; 3];
                                }
                                if scenario == "escape" && tick >= ticks / 2 {
                                    delta = [if n == 0 { -40_000 } else { 40_000 }, 0, 0];
                                }
                                p.translation =
                                    std::array::from_fn(|k| p.translation[k] + delta[k]);
                                proposed.insert(id, p);
                                plans.insert(
                                    id,
                                    ConstructorState {
                                        rigid: p,
                                        seeking_food: false,
                                    },
                                );
                            }
                            if scenario == "escape" && tick >= ticks / 2 && repeat == 0 {
                                escape = escape_samples(&before, &proposed, &mut cache)?;
                            }
                            resolve(&before, &mut next, &mut plans, &mut cache)?;
                            for &id in &ids {
                                let p = next.habitat.as_ref().unwrap().agents[&id].rigid;
                                set_pose(&mut next, id, p)?;
                            }
                            // Matched solver steps hold material time/obstacle age fixed.
                            state = next;
                            profile = cache.profile();
                        }
                        let elapsed = started.elapsed().as_nanos();
                        let after_mem = memory();
                        let components = diagnostics(&state, &mut cache)?;
                        let positions = ids
                            .iter()
                            .map(|id| Ok((*id, state.body_pose(*id)?)))
                            .collect::<Result<BTreeMap<_, _>, MorphError>>()?;
                        let row = json!({"sample_index":tick,"tick":state.tick,"escape_samples":escape,"proposed":proposed,"positions":positions,"contacts":state.habitat.as_ref().unwrap().contacts,"components":components,"events":&state.events[events_before..],"material_gap":state.condensate_gap(),"wall_ns":elapsed,"collision_profile":profile,"requested_bytes":after_mem[0]-allocation_before[0],"live_bytes":after_mem[1],"peak_live_bytes":after_mem[2],"rss_kib":rss()});
                        rows.push(row);
                    }
                    if family == "natural" {
                        w.validate()?;
                    }
                    if family == "natural" {
                        state.save(&dir.join(format!("final-{repeat}.json")))?;
                    } else {
                        write(&dir.join(format!("resolver-state-{repeat}.json")), &state)?;
                    }
                    let physical = rows
                        .iter()
                        .map(|v| {
                            let mut v = v.clone();
                            let obj = v.as_object_mut().unwrap();
                            for key in [
                                "wall_ns",
                                "collision_profile",
                                "requested_bytes",
                                "live_bytes",
                                "peak_live_bytes",
                                "rss_kib",
                                "escape_samples",
                            ] {
                                obj.remove(key);
                            }
                            v
                        })
                        .collect::<Vec<_>>();
                    if let Some(reference) = &canonical_rows {
                        require(reference == &physical, "research replay differs")?;
                    } else {
                        canonical_rows = Some(physical);
                    }
                    write(&dir.join(format!("rows-{repeat}.json")), &rows)?;
                }
            }
        }
    }
    Ok(())
}

/// Intentionally falsifiable component decomposition, not a lawful replacement.
pub fn components(
    world: &WorldSnapshot,
    f: &PipeCastFixture,
    workers: usize,
    repeats: usize,
) -> Result<Value, MorphError> {
    let authenticated = probe(world, f, false)?;
    let mut cache = CollisionCache::default();
    let a = cache.body(world, f.body)?;
    let (_, b) = cache.pipe(world, f.filament, f.chunk)?;
    require(
        a.parts.len() <= 64 && (1..=32).contains(&workers) && (1..=100).contains(&repeats),
        "research bounds",
    )?;
    let mut pool = if workers > 1 {
        Some(execution::Executor::new(ExecutionPolicy {
            workers,
            minimum_frontier: 2,
            ..Default::default()
        })?)
    } else {
        None
    };
    let mut rows = Vec::new();
    for _ in 0..repeats {
        let poses = [f.from, f.to, f.pipe_from, f.pipe_to];
        let context = execution::Context {
            tick: f.tick,
            body: f.body,
            ..Default::default()
        };
        let tasks = a
            .parts
            .iter()
            .enumerate()
            .map(|(i, (id, g))| execution::Task::cast(context.clone(), i, *id, g, &b, poses))
            .collect::<Vec<_>>();
        let keys = tasks.iter().map(execution::Task::key).collect::<Vec<_>>();
        let started = Instant::now();
        let mut frontier = pool.as_mut().and_then(|pool| pool.run(tasks));
        let mut selected: Hit = None;
        let mut parts = Vec::new();
        for (i, (id, g)) in a.parts.iter().enumerate() {
            let value = match (&mut pool, &mut frontier) {
                (Some(pool), Some(frontier)) => pool.consume(frontier, &keys[i]),
                _ => None,
            };
            let result = match value {
                Some(Ok(execution::Value::Cast(hit))) => hit,
                _ => cast(
                    g,
                    f.from,
                    f.to,
                    &b,
                    f.pipe_from,
                    f.pipe_to,
                    (
                        &mut CollisionProfile::default(),
                        &mut ContactWorkspace::default(),
                    ),
                )?,
            };
            if let Some(hit) = result
                && selected.is_none_or(|best| hit.0 < best.0)
            {
                selected = Some(hit);
            }
            parts.push(json!({"component":id,"hit":exact(result)}));
        }
        if let (Some(pool), Some(frontier)) = (&mut pool, frontier.take()) {
            pool.finish(frontier);
        }
        rows.push(json!({"query_ns":started.elapsed().as_nanos(),"hit":exact(selected),"exact":exact(selected)==f.reference_hit,"parts":parts}));
    }
    Ok(
        json!({"authenticated_reference":authenticated,"workers":workers,"rows":rows,"executor":pool.as_ref().map(|p|&p.profile),"authority":"isolated rejected-on-mismatch decomposition, no runtime consumer"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn anatomy_variants_preserve_nonflagellar_authority_and_stationary_collider() {
        let variants = variants();
        let reference =
            ShipRuntime::habitat_simulation(AnatomySpec::showcase(77), 77, 256).unwrap();
        let original = &reference.state().config.definitions[&1];
        for (_, spec) in &variants {
            spec.validate().unwrap();
            assert_eq!(spec.body, original.body);
            assert_eq!(
                spec.components
                    .iter()
                    .filter(|c| !matches!(c.shape, crate::anatomy::Shape::Flagellum { .. }))
                    .collect::<Vec<_>>(),
                original
                    .components
                    .iter()
                    .filter(|c| !matches!(c.shape, crate::anatomy::Shape::Flagellum { .. }))
                    .collect::<Vec<_>>()
            );
        }
        let mut world = ShipRuntime::habitat_simulation(original.clone(), 77, 32)
            .unwrap()
            .snapshot();
        let mut cache = CollisionCache::default();
        let first = cache.body(&world, 1).unwrap();
        world.tick += 19;
        let next = cache.body(&world, 1).unwrap();
        assert_eq!(geometry_hash(&first), geometry_hash(&next));
        let compiled = CompiledAnatomy::new(original.clone()).unwrap();
        assert_ne!(
            compiled
                .pose(habitat_presentation_pose(original, 0, false))
                .unwrap()
                .geometry
                .surfaces[1]
                .positions,
            compiled
                .pose(habitat_presentation_pose(original, 19, false))
                .unwrap()
                .geometry
                .surfaces[1]
                .positions
        );
    }
}

/// Piecewise circular annular cylinders: authored alternative, intentionally
/// omits Hermite curvature/ellipse/ridges/wobble. Never consumed by collision.
struct ProceduralPipe {
    segments: Vec<(Vector, Vector, f32, f32)>,
    sealed: [bool; 2],
}
impl ProceduralPipe {
    fn new(f: &crate::filament::Filament, tick: u64) -> Result<Self, MorphError> {
        let section = f
            .section
            .as_ref()
            .ok_or_else(|| MorphError("hollow section required".into()))?;
        let origin = f.nodes[0].position;
        let mut segments = Vec::new();
        for pair in f.nodes.windows(2) {
            let a = Vector::from_array(
                pair[0]
                    .position
                    .relative(origin)?
                    .map(|v| v as f32 / SCALE as f32),
            );
            let b = Vector::from_array(
                pair[1]
                    .position
                    .relative(origin)?
                    .map(|v| v as f32 / SCALE as f32),
            );
            let born = pair[0].deposited_tick.max(f.created_tick);
            let (outer, inner) = section.radii_at(tick.saturating_sub(born));
            segments.push((a, b, outer, inner));
        }
        Ok(Self {
            segments,
            sealed: [section.start_sealed, section.end_sealed],
        })
    }
    fn distance(&self, p: Vector) -> f32 {
        self.segments
            .iter()
            .enumerate()
            .map(|(i, (a, b, outer, inner))| {
                let ab = *b - *a;
                let axis = ab.normalize_or_zero();
                let axial = (p - *a).dot(axis);
                let length = ab.length();
                let radial = (p - (*a + axis * axial)).length();
                let wall = if radial < *inner {
                    inner - radial
                } else if radial > *outer {
                    radial - outer
                } else {
                    -(radial - inner).min(outer - radial)
                };
                let beyond = (-axial).max(axial - length).max(0.);
                let cap = (i == 0 && self.sealed[0] && axial <= outer - inner)
                    || (i + 1 == self.segments.len()
                        && self.sealed[1]
                        && axial >= length - (outer - inner));
                if cap && radial <= *outer {
                    beyond.max(radial - outer)
                } else if beyond > 0. {
                    (wall.max(0.).powi(2) + beyond.powi(2)).sqrt()
                } else {
                    wall
                }
            })
            .fold(f32::INFINITY, f32::min)
    }
}

pub fn procedural(
    world: &WorldSnapshot,
    out: &Path,
    memory: fn() -> [usize; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::filament::{Filament, Node, PipeSection};
    std::fs::create_dir(out)?;
    let source = world
        .assembly
        .as_ref()
        .and_then(|a| {
            a.filaments
                .values()
                .find(|f| f.section.is_some() && f.nodes.len() > 1)
        })
        .ok_or("no paid hollow pipe in input")?;
    write(&out.join("paid-source.json"), source)?;
    let mut controls = Vec::<(String, Filament, u64)>::new();
    for shape in [
        "straight-open",
        "curved-open",
        "sealed",
        "age-gradient",
        "junction-neighbour",
    ] {
        let mut f = source.clone();
        f.nodes.truncate(5);
        let origin = f.nodes[0].position;
        for (i, n) in f.nodes.iter_mut().enumerate() {
            let x = if matches!(shape, "curved-open" | "junction-neighbour") {
                (i as f32 * 1.1).sin() * 0.28
            } else {
                0.
            };
            n.position =
                origin.offset([(x * SCALE as f32).round() as i64, 0, i as i64 * 250_000])?;
            n.deposited_tick = if shape == "age-gradient" {
                world.tick.saturating_sub(i as u64 * 16)
            } else {
                world.tick
            };
        }
        // Extend short paid source recipes as explicit unpaid geometric controls.
        // Never insert these recipes into a live WorldSnapshot or material ledger.
        if f.nodes.len() == 2 {
            let last = f.nodes[1].clone();
            for i in 2..5 {
                f.nodes.push(Node {
                    id: last.id + i,
                    position: origin.offset([
                        if shape == "curved-open" {
                            i as i64 * 80_000
                        } else {
                            0
                        },
                        0,
                        i as i64 * 250_000,
                    ])?,
                    ..last.clone()
                });
            }
        }
        f.created_tick = world.tick.saturating_sub(64);
        f.section = Some(PipeSection {
            outer_radius: 180_000,
            inner_radius: 125_000,
            start_sealed: shape == "sealed",
            end_sealed: shape == "sealed",
        });
        for age in [0, 256] {
            controls.push((format!("{shape}-age{age}"), f.clone(), world.tick + age));
        }
    }
    let mut summary = Vec::new();
    for (name, f, tick) in controls {
        let dir = out.join(&name);
        std::fs::create_dir(&dir)?;
        write(
            &dir.join("recipe.json"),
            &json!({"filament":f,"tick":tick,"authority":"unpaid isolated geometry input; not a runtime world"}),
        )?;
        let before = memory();
        let started = Instant::now();
        let (_, surface) = filament_chunk_at(&f, 0, tick)?;
        let g = geometry(std::slice::from_ref(&surface), 1.)?;
        let exact_build_ns = started.elapsed().as_nanos();
        let exact_memory = memory();
        let started = Instant::now();
        let proc = ProceduralPipe::new(&f, tick)?;
        let proc_build_ns = started.elapsed().as_nanos();
        let proc_memory = memory();
        let bounds = g.shape.compute_local_aabb();
        let mut rows = Vec::new();
        let mut exact_ns = 0;
        let mut proc_ns = 0;
        let mut false_negative = 0;
        let mut false_positive = 0;
        let mut maximum_distance_error = 0f32;
        let sphere = Geometry {
            shape: Arc::new(Ball::new(0.01)),
            radius: 0.01,
            vertices: Arc::new(vec![]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::new(),
        };
        let mut workspace = ContactWorkspace::default();
        for iz in -2..=22 {
            for radius in [0., 0.08, 0.12, 0.14, 0.18, 0.21, 0.3] {
                for angle in 0..8 {
                    let angle = angle as f32 * std::f32::consts::TAU / 8.;
                    let p =
                        Vector::new(radius * angle.cos(), radius * angle.sin(), iz as f32 * 0.05);
                    let pa = Iso::from_translation(p);
                    let started = Instant::now();
                    let contact =
                        cached_world_contact(&mut workspace, &pa, &sphere, &Iso::IDENTITY, &g, 10.)
                            .map_err(|_| "unsupported canonical fidelity query")?;
                    let ns = started.elapsed().as_nanos();
                    exact_ns += ns;
                    let started = Instant::now();
                    let alternative = proc.distance(p) - 0.01;
                    proc_ns += started.elapsed().as_nanos();
                    let canonical = contact.map(|c| c.dist);
                    let hit = canonical.is_some_and(|d| d <= 0.);
                    let other = alternative <= 0.;
                    false_negative += usize::from(hit && !other);
                    false_positive += usize::from(!hit && other);
                    if let Some(d) = canonical {
                        maximum_distance_error =
                            maximum_distance_error.max((d - alternative).abs());
                    }
                    rows.push(json!({"position":p.to_array(),"canonical_distance":canonical,"procedural_distance":alternative,"hit":hit,"procedural_hit":other,"canonical_point":contact.map(|c|c.point2.to_array()),"canonical_normal":contact.map(|c|c.normal1.to_array())}));
                }
            }
        }
        write(&dir.join("queries.json"), &rows)?;
        summary.push(json!({"case":name,"queries":rows.len(),"false_negative":false_negative,"false_positive":false_positive,"max_distance_error_m":maximum_distance_error,"exact_build_ns":exact_build_ns,"procedural_build_ns":proc_build_ns,"exact_query_ns":exact_ns,"procedural_query_ns":proc_ns,"exact_build_requested_bytes":exact_memory[0]-before[0],"procedural_build_requested_bytes":proc_memory[0]-exact_memory[0],"exact_build_live_delta":exact_memory[1] as i64-before[1] as i64,"procedural_build_live_delta":proc_memory[1] as i64-exact_memory[1] as i64,"exact_mesh_vertices":surface.positions.len(),"exact_mesh_triangles":surface.indices.len()/3,"procedural_segments":proc.segments.len(),"canonical_bounds":[bounds.mins.to_array(),bounds.maxs.to_array()],"toi_deviation":"NOT RUN: static sphere probes only","normal_deviation":"NOT SUPPORTED by scalar-distance prototype","junctions":"neighbour shape recipe only; no union/load-bearing model","runtime_integration":false}));
    }
    write(&out.join("summary.json"), &summary)?;
    Ok(())
}

/// Separate construction attribution: unchanged mesh input, supported public APIs.
/// This does not reuse a geometry across a mutation or enter a runtime cache.
pub fn geometry_economy(
    world: &WorldSnapshot,
    out: &Path,
    memory: fn() -> [usize; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir(out)?;
    let mut rows = Vec::new();
    let assembly = world.assembly.as_ref().ok_or("assembly required")?;
    for f in assembly
        .filaments
        .values()
        .filter(|f| f.section.is_some())
        .take(12)
    {
        for tick in [world.tick, world.tick + 1, world.tick + 256] {
            let before = memory();
            let started = Instant::now();
            let (_, surface) = filament_chunk_at(f, 0, tick)?;
            let surface_ns = started.elapsed().as_nanos();
            let surface_mem = memory();
            let started = Instant::now();
            let g = geometry(std::slice::from_ref(&surface), 1.)?;
            let geometry_ns = started.elapsed().as_nanos();
            let geom_mem = memory();
            let started = Instant::now();
            let repeated = geometry(std::slice::from_ref(&surface), 1.)?;
            let repeated_ns = started.elapsed().as_nanos();
            let repeated_mem = memory();
            require(
                geometry_hash(&g) == geometry_hash(&repeated),
                "identical mesh reconstruction differs",
            )?;
            rows.push(json!({"filament":f.id,"tick":tick,"nodes":f.nodes.len(),"surface_vertices":surface.positions.len(),"surface_triangles":surface.indices.len()/3,"surface_generation_ns":surface_ns,"surface_requested_bytes":surface_mem[0]-before[0],"surface_retained_bytes":surface_mem[1] as i64-before[1] as i64,"collider_build_ns":geometry_ns,"collider_requested_bytes":geom_mem[0]-surface_mem[0],"collider_retained_bytes":geom_mem[1] as i64-surface_mem[1] as i64,"repeated_collider_ns":repeated_ns,"repeated_collider_requested_bytes":repeated_mem[0]-geom_mem[0],"geometry_fingerprint":geometry_hash(&g),"source_pipe_key":pipe_key(f.id,f,0,tick).map(|key|key.2)}));
        }
    }
    write(&out.join("rows.json"), &rows)?;
    Ok(())
}

/// Falsify 'frozen implies no lawful escape' using unilateral and bilateral
/// proposals from the identical frozen resolver state. No state is adopted.
pub fn escape_intervention(world: &WorldSnapshot) -> Result<Value, MorphError> {
    let ids = world
        .habitat
        .as_ref()
        .unwrap()
        .agents
        .keys()
        .copied()
        .collect::<Vec<_>>();
    require(ids.len() == 2, "two-builder escape intervention required")?;
    let mut rows = Vec::new();
    for moving in [vec![ids[0], ids[1]], vec![ids[0]], vec![ids[1]]] {
        let mut cache = CollisionCache::default();
        let a = cache.body(world, ids[0])?;
        let b = cache.body(world, ids[1])?;
        let from = [world.body_pose(ids[0])?, world.body_pose(ids[1])?];
        let mut target = from;
        for i in 0..2 {
            if moving.contains(&ids[i]) {
                target[i].translation[0] += if i == 0 { -40_000 } else { 40_000 };
            }
        }
        let origin = Position::from_absolute(from[0].translation)?;
        let am = motion(from[0], target[0], origin)?;
        let bm = motion(from[1], target[1], origin)?;
        let mut workspace = ContactWorkspace::default();
        let mut depths = Vec::new();
        for t in [0., 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.] {
            let c = cached_world_contact(
                &mut workspace,
                &am.position_at_time(t),
                &a,
                &bm.position_at_time(t),
                &b,
                0.001,
            )
            .map_err(|_| MorphError("escape contact unsupported".into()))?;
            depths.push(json!({"time":t,"distance":c.map(|c|c.dist),"normal":c.map(|c|c.normal1.to_array())}));
        }
        let recovery = nondeepening_recovery(
            &a,
            from[0],
            target[0],
            &b,
            from[1],
            target[1],
            &mut ContactWorkspace::default(),
        )?;
        let mut next = world.clone();
        let mut plans = ids
            .iter()
            .enumerate()
            .map(|(i, &id)| {
                (
                    id,
                    ConstructorState {
                        rigid: target[i],
                        seeking_food: false,
                    },
                )
            })
            .collect();
        resolve(world, &mut next, &mut plans, &mut cache)?;
        let admitted = ids
            .iter()
            .map(|id| (*id, next.habitat.as_ref().unwrap().agents[id].rigid))
            .collect::<BTreeMap<_, _>>();
        rows.push(json!({"moving_bodies":moving,"from":from,"target":target,"samples":depths,"canonical_nondeepening_recovery":recovery,"admitted":admitted,"all_proposals_fully_accepted":ids.iter().enumerate().all(|(i,id)|admitted[id]==target[i]),"contacts":next.habitat.as_ref().unwrap().contacts}));
    }
    Ok(
        json!({"authority":"isolated physical intervention; no runtime/material adoption","rows":rows}),
    )
}

#[cfg(test)]
mod model_boundary_tests {
    use super::*;
    #[test]
    fn procedural_hollow_lumen_wall_and_seal_are_explicit_alternative_law() {
        let mut model = ProceduralPipe {
            segments: vec![(Vector::ZERO, Vector::Z, 0.18, 0.125)],
            sealed: [false, false],
        };
        assert!(model.distance(Vector::new(0., 0., 0.5)) > 0.);
        assert!(model.distance(Vector::new(0.15, 0., 0.5)) < 0.);
        assert!(model.distance(Vector::new(0.3, 0., 0.5)) > 0.);
        assert!(model.distance(Vector::new(0., 0., 0.)) > 0.);
        model.sealed[0] = true;
        assert!(model.distance(Vector::ZERO) <= 0.);
    }
    #[test]
    fn relocating_an_experimental_pose_preserves_constructor_consistency() {
        let mut world =
            ShipRuntime::habitat_colony(AnatomySpec::showcase(77), 77, 32, 2, Some(4000))
                .unwrap()
                .snapshot();
        set_pose(&mut world, 1, pose(0., -1.35, 0.)).unwrap();
        ShipRuntime::restore(world).unwrap().validate().unwrap();
    }
}

/// Bounded authored trajectory trials. These are experimental interventions,
/// never organism steering/avoidance and never a runtime adoption path.
pub fn escape_routes(world: &WorldSnapshot) -> Result<Value, MorphError> {
    let ids = world
        .habitat
        .as_ref()
        .unwrap()
        .agents
        .keys()
        .copied()
        .collect::<Vec<_>>();
    require(ids.len() == 2, "two builders required")?;
    let mut trials = Vec::new();
    for axis in 0..3 {
        for sign in [-1, 1] {
            for bilateral in [false, true] {
                let mut state = world.clone();
                let mut cache = CollisionCache::default();
                let mut rows = Vec::new();
                let mut clear = false;
                for step in 0..40 {
                    let before = state.clone();
                    let mut plans = BTreeMap::new();
                    let mut proposed = BTreeMap::new();
                    for (i, &id) in ids.iter().enumerate() {
                        let mut target = before.body_pose(id)?;
                        if i == 0 || bilateral {
                            target.translation[axis] += 40_000 * sign * if i == 0 { 1 } else { -1 };
                        }
                        proposed.insert(id, target);
                        plans.insert(
                            id,
                            ConstructorState {
                                rigid: target,
                                seeking_food: false,
                            },
                        );
                    }
                    resolve(&before, &mut state, &mut plans, &mut cache)?;
                    for &id in &ids {
                        let target = state.habitat.as_ref().unwrap().agents[&id].rigid;
                        set_pose(&mut state, id, target)?;
                    }
                    let a = cache.body(&state, ids[0])?;
                    let b = cache.body(&state, ids[1])?;
                    let pa = state.body_pose(ids[0])?;
                    let pb = state.body_pose(ids[1])?;
                    let origin = Position::from_absolute(pa.translation)?;
                    let contact = cached_world_contact(
                        &mut ContactWorkspace::default(),
                        &motion(pa, pa, origin)?.start,
                        &a,
                        &motion(pb, pb, origin)?.start,
                        &b,
                        0.001,
                    )
                    .map_err(|_| MorphError("escape endpoint unsupported".into()))?;
                    clear = contact.is_none();
                    rows.push(json!({"step":step,"proposed":proposed,"admitted":ids.iter().map(|id|Ok((*id,state.body_pose(*id)?))).collect::<Result<BTreeMap<_,_>,MorphError>>()?,"contact_distance":contact.map(|c|c.dist),"clear":clear}));
                    if clear {
                        break;
                    }
                }
                trials.push(json!({"axis":axis,"sign":sign,"bilateral":bilateral,"reaches_clear":clear,"rows":rows}));
            }
        }
    }
    Ok(
        json!({"status":"bounded authored intervention, not spontaneous behaviour or configuration-space completeness","trials":trials}),
    )
}
