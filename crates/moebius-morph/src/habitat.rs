//! Authored local habitat policies, invoked only inside the canonical ship transaction.
//! No physics, learned pursuit or additional material stock is implied.
use crate::{
    emission::{MaterialBook, RigidPose, SCALE},
    runtime::*,
    *,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const ROOT_RADIUS: i64 = 35_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HabitatRules {
    #[serde(default)]
    pub collisions: bool,
    pub step: i64,
    pub flee_step: i64,
    pub evasion: bool,
    pub reproduction: bool,
    pub max_agents: usize,
    pub max_sacs: usize,
    pub child_stock: u64,
    pub reserve_food: u64,
    pub birth_interval: u64,
}
impl Default for HabitatRules {
    fn default() -> Self {
        Self {
            collisions: true,
            step: 60_000,
            flee_step: 26_000,
            evasion: true,
            reproduction: true,
            max_agents: 8,
            max_sacs: 48,
            child_stock: 64,
            reserve_food: 128,
            birth_interval: 360,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub travel: Option<TravelMemory>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub refilling: bool,
    pub rigid: RigidPose,
    pub heading: [i64; 3],
    pub eating: bool,
    pub born: u64,
    pub last_birth: u64,
    pub parent: Option<u64>,
}
/// Bounded, authored exploration state. This is not a learned controller.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TravelMemory {
    pub target: Option<Position>,
    pub last_food: Option<Position>,
    pub visited: Vec<[i64; 3]>,
}
fn explore(
    world: &WorldSnapshot,
    id: u64,
    agent: &mut Agent,
    blocked: bool,
) -> Result<[i64; 3], MorphError> {
    let at = world.instances[&id].position;
    let cell = |p: Position| -> Result<[i64; 3], MorphError> {
        Ok(p.absolute()?.map(|v| v.div_euclid(2 * SCALE)))
    };
    let heading = agent.heading;
    let memory = agent.travel.get_or_insert_with(TravelMemory::default);
    let here = cell(at)?;
    if !memory.visited.contains(&here) {
        if memory.visited.len() == 256 {
            memory.visited.remove(0);
        }
        memory.visited.push(here);
    }
    if memory
        .target
        .is_none_or(|p| distance(p, at).is_ok_and(|d| d < SCALE as f64))
        || blocked && (world.tick + id).is_multiple_of(24)
    {
        let mut best = None;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if [x, y, z] == [0; 3] {
                        continue;
                    }
                    let direction = normalized([x, y, z], SCALE);
                    let mut novelty = 0i128;
                    let mut target = None;
                    for step in 1..=3 {
                        let Ok(p) = at.offset(direction.map(|v| v * 2 * step)) else {
                            continue;
                        };
                        novelty += i128::from(!memory.visited.contains(&cell(p)?));
                        target = Some(p);
                    }
                    let Some(target) = target else {
                        continue;
                    };
                    let alignment: i128 = (0..3)
                        .map(|k| i128::from(heading[k]) * i128::from(direction[k]))
                        .sum();
                    let score = novelty * 4 * i128::from(SCALE).pow(2) + alignment;
                    if best.as_ref().is_none_or(|(old, _)| score > *old) {
                        best = Some((score, target));
                    }
                }
            }
        }
        memory.target = best.map(|(_, p)| p);
    }
    memory.target.map_or(Ok(heading), |p| p.relative(at))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub filament: u64,
    pub node: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Habitat {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<crate::collision::ContactObservation>,
    pub rules: HabitatRules,
    pub agents: BTreeMap<u64, Agent>,
    pub sacs: BTreeMap<u64, Attachment>,
}
fn len(v: [i64; 3]) -> f64 {
    v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt()
}
fn normalized(v: [i64; 3], scale: i64) -> [i64; 3] {
    let length = len(v);
    if length < 1. {
        return [0, 0, scale];
    }
    v.map(|x| (x as f64 * scale as f64 / length).round() as i64)
}
/// Bounded steering on the unit sphere, including an exactly-behind target.
/// Normalized linear blending cannot turn at all when desired == -heading.
fn turn_heading(heading: [i64; 3], desired: [i64; 3]) -> [i64; 3] {
    let h = normalized(heading, SCALE).map(|v| v as f64 / SCALE as f64);
    let d = normalized(desired, SCALE).map(|v| v as f64 / SCALE as f64);
    let dot = (0..3).map(|k| h[k] * d[k]).sum::<f64>().clamp(-1., 1.);
    let mut tangent = std::array::from_fn::<_, 3, _>(|k| d[k] - h[k] * dot);
    let mut length = tangent.iter().map(|v| v * v).sum::<f64>().sqrt();
    if length < 0.000001 {
        if dot > 0. {
            return normalized(desired, SCALE);
        }
        let axis = if h[1].abs() < 0.9 {
            [0., 1., 0.]
        } else {
            [1., 0., 0.]
        };
        tangent = [
            h[1] * axis[2] - h[2] * axis[1],
            h[2] * axis[0] - h[0] * axis[2],
            h[0] * axis[1] - h[1] * axis[0],
        ];
        length = tangent.iter().map(|v| v * v).sum::<f64>().sqrt();
    }
    let angle = dot.acos().min(0.06);
    normalized(
        std::array::from_fn(|k| {
            ((h[k] * angle.cos() + tangent[k] / length * angle.sin()) * SCALE as f64).round() as i64
        }),
        SCALE,
    )
}
fn distance(a: Position, b: Position) -> Result<f64, MorphError> {
    Ok(len(a.relative(b)?))
}
pub fn anchor(world: &WorldSnapshot, attachment: &Attachment) -> Option<Position> {
    world
        .assembly
        .as_ref()?
        .filaments
        .get(&attachment.filament)?
        .nodes
        .iter()
        .find(|n| n.id == attachment.node)
        .map(|n| n.position)
}
fn nearest_anchor(
    world: &WorldSnapshot,
    p: Position,
    radius: i64,
) -> Result<Option<Attachment>, MorphError> {
    let mut best = radius as f64;
    let mut choice = None;
    if let Some(a) = &world.assembly {
        for f in a.filaments.values() {
            for node in &f.nodes {
                let d = distance(node.position, p)?;
                if d < best {
                    best = d;
                    choice = Some(Attachment {
                        filament: f.id,
                        node: node.id,
                    });
                }
            }
        }
    }
    Ok(choice)
}
fn toward(from: Position, to: Position, step: i64) -> Result<Position, MorphError> {
    let delta = to.relative(from)?;
    let length = len(delta);
    let step = (step as f64).min(length);
    if length < 1. {
        return Ok(from);
    }
    from.offset(delta.map(|x| (x as f64 * step / length).round() as i64))
}

/// Plan food evasion and smooth motion from local food/pipe observations only.
/// Candidate state is private to ShipRuntime::tick until its transaction commits.
pub(crate) fn prepare(
    world: &mut WorldSnapshot,
) -> Result<BTreeMap<u64, ConstructorState>, MorphError> {
    let mut habitat = world
        .habitat
        .clone()
        .ok_or_else(|| MorphError("habitat missing".into()))?;
    let mouths = habitat
        .agents
        .keys()
        .filter_map(|id| {
            world
                .mouth_position(*id)
                .transpose()
                .map(|r| r.map(|p| (*id, p)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let food_ids = world
        .instances
        .values()
        .filter(|i| i.definition == BLOOD_SAC)
        .map(|i| i.id)
        .collect::<Vec<_>>();
    for id in food_ids {
        let position = world.instances[&id].position;
        if let std::collections::btree_map::Entry::Vacant(entry) = habitat.sacs.entry(id)
            && let Some(a) = nearest_anchor(world, position, 300_000)?
        {
            entry.insert(a);
        }

        let threat = mouths
            .iter()
            .map(|(_, p)| Ok((distance(*p, position)?, *p)))
            .collect::<Result<Vec<_>, MorphError>>()?
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let scared = habitat.rules.evasion && threat.is_some_and(|(d, _)| d < 3. * SCALE as f64);
        let mut next = position;
        if let Some(a) = habitat.sacs.get(&id).cloned() {
            let f = &world.assembly.as_ref().unwrap().filaments[&a.filament];
            let index = f.nodes.iter().position(|n| n.id == a.node).unwrap();
            let mut choice = index;
            if scared {
                let predator = threat.unwrap().1;
                for k in [index.saturating_sub(1), (index + 1).min(f.nodes.len() - 1)] {
                    if distance(f.nodes[k].position, predator)?
                        > distance(f.nodes[choice].position, predator)?
                    {
                        choice = k;
                    }
                }
            }
            let target = f.nodes[choice].position.offset([
                0,
                if habitat.rules.collisions {
                    240_000
                } else {
                    120_000
                },
                0,
            ])?;
            // Sacs crawl along real substrate; rooted growth never teleports.
            next = toward(
                position,
                target,
                if scared {
                    habitat.rules.flee_step
                } else {
                    8_000
                },
            )?;
            if scared && choice == index && (world.tick + id) % 24 < 18 {
                next = position.offset(normalized(
                    position.relative(threat.unwrap().1)?,
                    habitat.rules.flee_step,
                ))?;
                if distance(next, f.nodes[index].position)? > 700_000. {
                    habitat.sacs.remove(&id);
                }
            }
            if distance(next, target)? < 35_000. {
                habitat.sacs.insert(
                    id,
                    Attachment {
                        filament: f.id,
                        node: f.nodes[choice].id,
                    },
                );
            }
        } else if scared {
            // A detached founding sac makes short, intermittent escape strokes.
            if (world.tick + id) % 24 < 10 {
                let away = normalized(
                    position.relative(threat.unwrap().1)?,
                    habitat.rules.flee_step,
                );
                next = position.offset(away)?;
            }
        }
        world.instances.get_mut(&id).unwrap().position = next;
        world.instances.get_mut(&id).unwrap().exposure =
            crate::runtime::exposure(&world.config, next)?;
    }
    let mut plans = BTreeMap::new();
    for (&id, agent) in &mut habitat.agents {
        let body = &world.instances[&id];
        let capacity = world.config.definitions[&body.definition].food_capacity();
        let mouth = world.mouth_position(id)?.unwrap_or(body.position);
        let mut nearby = world
            .instances
            .values()
            .filter(|i| i.definition == BLOOD_SAC && i.book.reservoir > 1)
            .map(|food| Ok((distance(food.position, mouth)?, food.id)))
            .collect::<Result<Vec<_>, MorphError>>()?;
        nearby.retain(|(d, _)| *d <= 16. * SCALE as f64);
        nearby.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let contact = nearby.first().is_some_and(|(d, _)| *d <= 420_000.);
        // Refill as a coherent meal, then release the feeding goal and travel.
        // Reproduction may request another meal when its unchanged interval is due.
        let reserve = habitat.rules.reserve_food.min(capacity / 4);
        let birth_need = habitat.rules.child_stock + 96;
        let birth_due = habitat.rules.reproduction
            && world.tick.saturating_sub(agent.last_birth) >= habitat.rules.birth_interval;
        if body.book.reservoir < reserve
            || birth_due && body.book.reservoir < birth_need
            || capacity < birth_need
        {
            agent.refilling = true;
        }
        let refill_target = (capacity / 3).max(birth_need).min(capacity);
        if body.book.reservoir >= refill_target {
            agent.refilling = false;
        }
        let appetite = agent.refilling;
        agent.eating = contact && appetite && body.book.reservoir < capacity;
        let obstructed = habitat
            .contacts
            .iter()
            .any(|c| c.organism == id && c.response == crate::collision::ContactResponse::Blocked);
        let exploration = explore(world, id, agent, obstructed)?;
        if let Some((_, food)) = nearby.first() {
            agent.travel.as_mut().unwrap().last_food = Some(world.instances[food].position);
        }
        let mut food_target = nearby
            .first()
            .map(|(_, food)| world.instances[food].position)
            .or(agent.travel.as_ref().unwrap().last_food);
        if nearby.is_empty()
            && food_target.is_some_and(|p| distance(p, mouth).is_ok_and(|d| d < 500_000.))
        {
            agent.travel.as_mut().unwrap().last_food = None;
            food_target = None;
        }
        let mut desired = if obstructed {
            // Keep the existing small clearance-search curvature while blocked;
            // steering straight at the same refused normal can deadlock a meal.
            let phase = (world.tick as f64 * 0.003 + id as f64 * 1.7).sin();
            let mut heading = agent.heading;
            heading[0] = heading[0].saturating_add((phase * 9000.) as i64);
            heading[1] = heading[1]
                .saturating_add(((world.tick as f64 * 0.005 + id as f64).sin() * 3500.) as i64);
            heading
        } else if appetite {
            food_target.map_or(Ok(exploration), |p| p.relative(mouth))?
        } else {
            exploration
        };
        desired = normalized(desired, SCALE);
        // Local route reinforcement: favour nearby forward-facing paid substrate.
        if appetite && let Some(a) = nearest_anchor(world, body.position, SCALE)? {
            let f = &world.assembly.as_ref().unwrap().filaments[&a.filament];
            let k = f.nodes.iter().position(|n| n.id == a.node).unwrap();
            if k + 1 < f.nodes.len() {
                let tangent = normalized(
                    f.nodes[k + 1].position.relative(f.nodes[k].position)?,
                    SCALE,
                );
                let dot: i128 = (0..3)
                    .map(|k| i128::from(tangent[k]) * i128::from(desired[k]))
                    .sum();
                if dot > i128::from(SCALE).pow(2) / 2 {
                    desired =
                        normalized(std::array::from_fn(|k| desired[k] * 4 + tangent[k]), SCALE);
                }
            }
        }
        agent.heading = turn_heading(agent.heading, desired);
        let step = if agent.eating {
            (habitat.rules.step / 5).max(1)
        } else {
            habitat.rules.step
        };
        agent.rigid.translation = body
            .position
            .offset(normalized(agent.heading, step))?
            .absolute()?;
        let aim = if let Some(target) = food_target.filter(|_| appetite && !obstructed) {
            target.relative(body.position)?
        } else {
            agent.heading
        };
        // Turn the body toward food from its centre, not toward the mouth's
        // error vector (which reverses when a meal lies behind the lip).
        let target_yaw = (aim[0] as f64).atan2(aim[2] as f64);
        let yaw = (agent.rigid.yaw_sin as f64).atan2(agent.rigid.yaw_cos as f64);
        let turn = (target_yaw - yaw + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        let yaw = yaw + turn.clamp(-0.08, 0.08);
        if !obstructed {
            agent.rigid.yaw_sin = (yaw.sin() * SCALE as f64).round() as i64;
            agent.rigid.yaw_cos = (yaw.cos() * SCALE as f64).round() as i64;
            agent.rigid.axial_roll = (agent.rigid.axial_roll + 28_000).rem_euclid(6_283_185);
        }
        plans.insert(
            id,
            ConstructorState {
                rigid: agent.rigid,
                seeking_food: agent.eating,
            },
        );
    }
    world.habitat = Some(habitat);
    Ok(plans)
}

pub(crate) fn finish(world: &mut WorldSnapshot) -> Result<(), MorphError> {
    let mut h = world.habitat.clone().unwrap();
    // Reproduction transfers paid reserve; newborns inherit anatomy and active policy.
    let parents = h.agents.keys().copied().collect::<Vec<_>>();
    for parent in parents {
        if !h.rules.reproduction || h.agents.len() >= h.rules.max_agents {
            break;
        }
        let a = &h.agents[&parent];
        let body = &world.instances[&parent];
        if body.book.external_input == 0
            || body.book.reservoir < h.rules.child_stock + 96
            || world.tick.saturating_sub(a.last_birth) < h.rules.birth_interval
        {
            continue;
        }
        let id = world.instances.keys().max().copied().unwrap() + 1;
        let definition = body.definition;
        let mut child_pose = a.rigid;
        child_pose.translation = body.position.absolute()?;
        let position = if h.rules.collisions {
            let Some(pose) = crate::collision::spawn_pose(world, definition, child_pose, None)?
            else {
                continue;
            };
            Position::from_absolute(pose.translation)?
        } else {
            body.position.offset([250_000, 180_000, 0])?
        };
        let mut book = MaterialBook::new(
            id,
            world.config.seed,
            h.rules.child_stock,
            world.config.environment,
            4096,
        )?;
        book.sync_nozzles(&world.config.definitions[&definition]);
        book.tick = world.tick + 1;
        book.emission_enabled = true;
        book.launch_speed = 0;
        world
            .instances
            .get_mut(&parent)
            .unwrap()
            .book
            .endow(h.rules.child_stock)?;
        world.instances.insert(
            id,
            Instance {
                id,
                definition,
                position,
                velocity: [0; 3],
                exposure: crate::runtime::exposure(&world.config, position)?,
                book,
            },
        );
        let mut child = h.agents[&parent].clone();
        child.rigid.translation = position.absolute()?;
        child.born = world.tick + 1;
        child.last_birth = world.tick + 1;
        child.parent = Some(parent);
        child.travel = None;
        child.refilling = true;
        child.eating = false;
        child.heading = normalized(
            [child.heading[2], child.heading[1], -child.heading[0]],
            SCALE,
        );
        h.agents.insert(id, child);
        h.agents.get_mut(&parent).unwrap().last_birth = world.tick + 1;
        world.events.push(Event::Reproduced {
            at: world.tick,
            parent,
            child: id,
            quantity: h.rules.child_stock,
        });
    }
    // Finite condensate germinates on paid substrate, never in empty space.
    if world.tick.is_multiple_of(60)
        && world
            .instances
            .values()
            .filter(|i| i.definition == BLOOD_SAC)
            .count()
            < h.rules.max_sacs
        && world.condensate.as_ref().is_some_and(|c| c.biomass >= 16)
        && let Some(f) = world.assembly.as_ref().and_then(|a| {
            a.filaments
                .values()
                .max_by_key(|f| f.nodes.last().map_or(0, |n| n.deposited_tick))
        })
        && let Some(n) = f.nodes.last()
    {
        let attachment = Attachment {
            filament: f.id,
            node: n.id,
        };
        let position =
            n.position
                .offset([0, if h.rules.collisions { 240_000 } else { 120_000 }, 0])?;
        let position = if h.rules.collisions {
            let near = RigidPose {
                translation: position.absolute()?,
                ..RigidPose::identity()
            };
            let Some(pose) = crate::collision::spawn_pose(world, BLOOD_SAC, near, None)? else {
                world.habitat = Some(h);
                return Ok(());
            };
            Position::from_absolute(pose.translation)?
        } else {
            position
        };
        let id = world.instances.keys().max().copied().unwrap() + 1;
        world.condensate.as_mut().unwrap().biomass -= 16;
        crate::runtime::insert_sac(world, position, 16, world.tick + 1)?;
        h.sacs.insert(id, attachment);
    }
    // Sacs reproduce from their own stock, placing offspring on actual pipe nodes.
    let sacs = h.sacs.keys().copied().collect::<Vec<_>>();
    for parent in sacs {
        let count = world
            .instances
            .values()
            .filter(|i| i.definition == BLOOD_SAC)
            .count();
        if count >= h.rules.max_sacs {
            break;
        }
        if !(world.tick + parent).is_multiple_of(90) || world.instances[&parent].book.reservoir < 32
        {
            continue;
        }
        let attachment = h.sacs[&parent].clone();
        let position = anchor(world, &attachment).unwrap().offset([
            0,
            if h.rules.collisions { 260_000 } else { 160_000 },
            0,
        ])?;
        let position = if h.rules.collisions {
            let near = RigidPose {
                translation: position.absolute()?,
                ..RigidPose::identity()
            };
            let Some(pose) = crate::collision::spawn_pose(world, BLOOD_SAC, near, None)? else {
                continue;
            };
            Position::from_absolute(pose.translation)?
        } else {
            position
        };
        world.instances.get_mut(&parent).unwrap().book.endow(12)?;
        let id = world.instances.keys().max().copied().unwrap() + 1;
        crate::runtime::insert_sac(world, position, 12, world.tick + 1)?;
        h.sacs.insert(id, attachment);
    }
    world.habitat = Some(h);
    Ok(())
}

pub(crate) fn validate(world: &WorldSnapshot) -> Result<(), MorphError> {
    let Some(h) = &world.habitat else {
        return Ok(());
    };
    if h.rules.collisions {
        require(
            world.instances.values().all(|i| {
                i.velocity == [0; 3]
                    && i.book.launch_speed == 0
                    && i.book.parcels.iter().all(|p| p.velocity == [0; 3])
            }),
            "collision mode has no unconstrained drift or ballistic droplets",
        )?;
        require(
            world.config.environment == crate::emission::Environment::suspended(),
            "collision mode currently requires suspended, force-free medium",
        )?;
    }

    require(
        (1..=16).contains(&h.rules.max_agents)
            && (1..=96).contains(&h.rules.max_sacs)
            && (1..=100_000).contains(&h.rules.step)
            && (0..=h.rules.step).contains(&h.rules.flee_step)
            && (16..=256).contains(&h.rules.child_stock)
            && h.rules.reserve_food <= 8192
            && h.rules.birth_interval >= 60,
        "invalid habitat rules",
    )?;
    require(
        world.assembly.is_some() && world.config.construction.is_some(),
        "habitat infrastructure missing",
    )?;
    require(
        world
            .assembly
            .as_ref()
            .unwrap()
            .filaments
            .values()
            .all(|f| h.agents.contains_key(&f.organism)),
        "orphan habitat filament",
    )?;
    require(
        world
            .instances
            .values()
            .filter(|i| i.definition != BLOOD_SAC)
            .count()
            == h.agents.len()
            && world
                .instances
                .values()
                .filter(|i| i.definition == BLOOD_SAC)
                .count()
                <= h.rules.max_sacs,
        "unregistered habitat population",
    )?;
    require(
        !h.agents.is_empty() && h.agents.len() <= h.rules.max_agents,
        "habitat population budget",
    )?;
    for (&id, a) in &h.agents {
        let i = world
            .instances
            .get(&id)
            .ok_or_else(|| MorphError("habitat body missing".into()))?;
        let spec = &world.config.definitions[&i.definition];
        require(
            i.definition != BLOOD_SAC
                && !i.book.registered.is_empty()
                && i.position.absolute()? == a.rigid.translation
                && spec.food_capacity() > 0
                && i.book.reservoir <= spec.food_capacity()
                && a.born <= world.tick
                && a.last_birth <= world.tick,
            "habitat anatomy/storage/timeline mismatch",
        )?;
        if let Some(memory) = &a.travel {
            require(
                memory.visited.len() <= 256
                    && memory.visited.iter().all(|cell| {
                        cell.iter()
                            .all(|v| v.unsigned_abs() <= (MAX_POSITION / (2 * SCALE) + 1) as u64)
                    }),
                "invalid travel memory",
            )?;
            if let Some(p) = memory.target {
                p.absolute()?;
            }
            if let Some(p) = memory.last_food {
                p.absolute()?;
            }
        }
        crate::emission::place_point([0.; 3], a.rigid)?;
        require(
            (len(a.heading) - SCALE as f64).abs() < 4.,
            "invalid directional memory",
        )?;
        if let Some(parent) = a.parent {
            require(
                world.events.iter().any(
                    |e| matches!(e,Event::Reproduced{parent:p,child,..}if *p==parent&&*child==id),
                ),
                "offspring lacks paid provenance",
            )?;
        }
    }
    for (&id, a) in &h.sacs {
        require(
            world
                .instances
                .get(&id)
                .is_some_and(|i| i.definition == BLOOD_SAC)
                && anchor(world, a).is_some(),
            "dangling sac substrate",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod travel_tests {
    use super::*;
    #[test]
    fn an_exactly_behind_goal_turns_without_instant_reversal() {
        let mut heading = [SCALE, 0, 0];
        let first = turn_heading(heading, [-SCALE, 0, 0]);
        assert!(
            first[0] < SCALE && first[0] > 980_000 && (first[1] != 0 || first[2] != 0),
            "head-on blend locked or reversed"
        );
        for _ in 0..60 {
            heading = turn_heading(heading, [-SCALE, 0, 0]);
        }
        assert!(
            heading[0] < -900_000,
            "goal behind the body remained unreachable"
        );
    }
}
