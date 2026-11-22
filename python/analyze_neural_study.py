#!/usr/bin/env python3
"""Analyze committed-state neural-study artifacts; never train or simulate.
Requires only the standard library for statistics/audits. Optional figures use
matplotlib if already present; no dependency is installed by this script.
"""
import argparse, csv, gzip, hashlib, json, math, random, shutil, statistics, struct
from pathlib import Path


def bootstrap(values, seed=1701):
    rng = random.Random(seed)
    means = sorted(statistics.mean(rng.choices(values, k=len(values))) for _ in range(2000))
    return [means[49], means[1949]]


def independent_snapshot(path):
    data = path.read_bytes()
    assert data[:8] == b"MAS1SNAP"
    version, = struct.unpack_from('<I', data, 8)
    length, = struct.unpack_from('<I', data, 76)
    assert version == 1 and len(data) == 80 + length
    offset = 80
    def take(fmt):
        nonlocal offset
        result = struct.unpack_from('<'+fmt, data, offset)
        offset += struct.calcsize('<'+fmt)
        return result
    tick, seed = take('QQ')
    dims = take('III')
    reservoir, waste, external_input, external_output, credited, maintenance, disposed, initial, next_id, cursor = take('10Q')
    count, = take('I')
    live_energy = 0
    for _ in range(count):
        cell_id, x, y, z, energy, age = take('QIIIQQ')
        assert 0 < cell_id < next_id and x < dims[0] and y < dims[1] and z < dims[2]
        live_energy += energy
        offset += 8
    dead, = take('I')
    offset += dead * 52
    volume, = take('I')
    assert volume == math.prod(dims)
    field = sum(struct.unpack_from('<'+str(volume)+'Q', data, offset))
    offset += volume * 8
    assert data[offset:offset+5] == b'FARM3'
    offset += 5
    size, = take('I')
    assert offset + size == len(data)
    state = json.loads(data[offset:])
    material = initial + external_input - field - reservoir - waste - external_output - sum(x['amount'] for x in state['flesh']) - len(state.get('jets', []))
    energy = credited - live_energy - maintenance - disposed
    momentum = list(state.get('environment_momentum', [0, 0, 0]))
    for brain in state['brains']:
        for axis, value in enumerate(brain['neural']['body']['momentum']):
            momentum[axis] += value
    for jet in state.get('jets', []):
        for axis, value in enumerate(jet['momentum']):
            momentum[axis] += value
    assert material == energy == 0 and momentum == [0, 0, 0]
    return dict(file=str(path), tick=tick, seed=seed, live=count, dead=dead, material_residual=material, energy_residual=energy, momentum_residual=momentum, hash_status='BLAKE3 not independently recomputed by this Python audit; Rust decode/replay tests verify it')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('run', type=Path)
    parser.add_argument('--dest', type=Path, required=True)
    args = parser.parse_args()
    report = json.loads((args.run / 'results.json').read_text())
    results = report['results']
    assert all(r['status'] == 'COMPLETE' for r in results), 'failed trials remain in raw results; do not discard them'
    assert all(r['material_residual'] == r['energy_residual'] == 0 for r in results)
    dest = args.dest
    dest.mkdir(parents=True, exist_ok=True)
    groups = {}
    for r in results:
        groups.setdefault((r['environment'], r['condition']), []).append(r)
    metrics = ['intake','early_intake','late_intake','final_energy','distance_sites','jets_released','excavated','dead','plastic_l1_max']
    summaries = []
    comparisons = []
    for (env, cond), rows in groups.items():
        summary = dict(environment=env, condition=cond, n=len(rows))
        for metric in metrics:
            values = [r[metric] for r in rows]
            summary[metric] = dict(mean=statistics.mean(values), sample_sd=statistics.stdev(values), values=values)
        summaries.append(summary)
        if cond == 'plastic_on':
            continue
        control = {r['seed']:r for r in groups[(env,'plastic_on')]}
        assert all(r['initial_hash'] == control[r['seed']]['initial_hash'] and r['initial_genotypes'] == control[r['seed']]['initial_genotypes'] for r in rows)
        if env == 'lineage':
            continue
        for metric in metrics:
            differences = [control[r['seed']][metric] - r[metric] for r in rows]
            comparisons.append(dict(environment=env, baseline=cond, metric=metric, sign='plastic_on minus baseline', n=len(rows), paired_differences=differences, mean_difference=statistics.mean(differences), sample_sd=statistics.stdev(differences), bootstrap95=bootstrap(differences)))
    mutation_on = groups[('lineage','plastic_on')]
    mutation_off = groups[('lineage','mutation_off')]
    assert all(r['changed_germline_descendants']==r['common_garden_different_descendants']==0 for r in mutation_off)
    lineage = dict(mutating_descendants=sum(r['descendants_measured'] for r in mutation_on), mutated_germlines=sum(r['changed_germline_descendants'] for r in mutation_on), different_fresh_state_command_probes=sum(r['common_garden_different_descendants'] for r in mutation_on), mutation_off_changed_germlines=sum(r['changed_germline_descendants'] for r in mutation_off), mutation_off_changed_command_probes=sum(r['common_garden_different_descendants'] for r in mutation_off), mutating_extinct_replicates=sum(r['live']==0 for r in mutation_on), nonmutating_extinct_replicates=sum(r['live']==0 for r in mutation_off), adaptive_evolution='NOT DEMONSTRATED')
    checks = [independent_snapshot(path) for path in sorted(args.run.glob('final_*.bin'))]
    assessment = dict(trials=len(results), matched_initial_state_and_founders='PASS', failed_trials=report['failed_trials'], summaries=summaries, paired_comparisons=comparisons, lineage=lineage, independent_final_snapshot_checks=checks, performance=report['performance'], verdicts=dict(motor_activity='IMPLEMENTED AND MEASURED; individual participation in raw results', recurrence='mechanism and controlled constant-input oscillation test PASS; spontaneous useful coordination NOT DEMONSTRATED', experience_dependent_parameter_change='MEASURED', improved_foraging_from_plasticity='NOT DEMONSTRATED', learned_locomotion='NOT DEMONSTRATED', heritable_command_divergence='MEASURED in fresh-state probes; mechanism is mutation, not adaptive success', learned_tunnelling='NOT DEMONSTRATED', evolutionary_adaptation='NOT DEMONSTRATED; see per-condition extinction counts', emergent_ecosystem='NOT DEMONSTRATED'), limitations=[f'{len(mutation_on)} seeds; exploratory multiple comparisons; no retention/generalization transfer test','mutation/common-garden probe measures commands, not all possible phenotype effects','chemistry ablation removes named external chemical channels; feedstock readiness and intake remain','recurrence knockout also changes dynamic response amplitude; not a gain-matched control','no full articulated rigid-body, capsule contact or hydrodynamic solver; angular counter-rotation approximate','effective inertia is not tracked biomass; mechanical energy is not the action-energy ledger','local material supplies secretion; a stored internal secretion reservoir is not modeled','microbenchmark on 16^3 excludes rendering/hashing; not a 256^3 scalability guarantee'])
    (dest/'analysis.json').write_text(json.dumps(assessment,indent=2)+'\n')
    for file in ['results.json','plan.json','interfaces.json']:
        shutil.copy2(args.run/file,dest/file)
    for path in sorted(args.run.glob('trace_*.jsonl')):
        (dest/(path.name+'.gz')).write_bytes(gzip.compress(path.read_bytes(),mtime=0))
    source_manifest = {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(args.run.iterdir()) if p.is_file()}
    (dest/'raw_artifact_hashes.json').write_text(json.dumps(source_manifest,indent=2)+'\n')
    lines=['# Neural foundation — measured results','',f"{len(results)} trials; {len(mutation_on)} matched seeds per condition; {report['failed_trials']} failed trials.",'','Published studies are context. These results validate only the stated project mechanisms and observations.','', '| Environment / condition | Intake mean ± SD | COM distance, sites | Excavated units | Plastic delta L1 |','|---|---:|---:|---:|---:|']
    for s in summaries:
        lines.append(f"| {s['environment']} / {s['condition']} | {s['intake']['mean']:.3f} ± {s['intake']['sample_sd']:.3f} | {s['distance_sites']['mean']:.4f} | {s['excavated']['mean']:.3f} | {s['plastic_l1_max']['mean']:.3f} |")
    endpoints_equal=all(statistics.mean(r[key] for r in groups[(env,'plastic_on')])==statistics.mean(r[key] for r in groups[(env,'plastic_off')]) for env in ['foraging','contact'] for key in ['intake','final_energy'])
    plastic_statement=('Plasticity changed synaptic state, but plastic-on and plastic-off had the same mean intake and endpoint energy in these trials.' if endpoints_equal else 'Plasticity changed synaptic state; endpoint differences and paired uncertainty are retained in analysis.json. Endpoint differences alone do not establish learned competence.')
    random_better=statistics.mean(r['intake'] for r in groups[('foraging','random')])>statistics.mean(r['intake'] for r in groups[('foraging','plastic_on')])
    baseline_statement=('The random baseline obtained more food in the foraging environment.' if random_better else 'Random-baseline comparisons are recorded without inferring purpose or competence.')
    outcome_statement=plastic_statement+' '+baseline_statement+' This does not establish learned locomotion.'
    lines += ['', outcome_statement,'',f"Mutation produced {lineage['mutated_germlines']} changed descendant genotypes among {lineage['mutating_descendants']} measured descendants, with {lineage['different_fresh_state_command_probes']} different fresh-state command probes. Mutation-off produced {lineage['mutation_off_changed_germlines']} changed genotypes and {lineage['mutation_off_changed_command_probes']} changed probes. Extinct lineage trials: {lineage['mutating_extinct_replicates']}/{len(mutation_on)} with mutation, {lineage['nonmutating_extinct_replicates']}/{len(mutation_off)} without. This is heritable divergence, not demonstrated evolutionary adaptation.",'','## Uncertainty and controls','','All seed values, sample standard deviations, paired differences and deterministic 95% percentile bootstrap intervals (2000 resamples) are in `analysis.json`. Identical initial state hashes and founder genotypes were checked across matched conditions. These small exploratory comparisons are not corrected for multiple testing; intervals describe these seeded fixtures, not ecological generality.','',f"All {len(checks)} saved final snapshots independently pass Python material, energy and linear-momentum checks. BLAKE3 is not independently recomputed by this script; Rust snapshot decode/replay tests cover hashing and continuation. All trials pass the core audits at every tick.",'','## Performance','','| Initial population | ms / World tick, 16³ | Final population |','|---|---:|---:|']
    for p in report['performance']:
        lines.append(f"| {p['initial_population']} | {p['seconds']/p['ticks']*1000:.4f} | {p['final_population']} |")
    lines += ['', 'Measurements include cloning, neural/actuator integration and integrity audits. Rendering, snapshot hashing and the dense 256³ world are excluded. No broader scalability guarantee is inferred.','', '## Limits',''] + ['- '+x for x in assessment['limitations']]
    (dest/'REPORT.md').write_text('\n'.join(lines)+'\n')
    try:
        import matplotlib
        matplotlib.use('Agg')
        import matplotlib.pyplot as plt
        trace=[json.loads(x) for x in (args.run/'trace_foraging_1.jsonl').read_text().splitlines()]
        trace=[r for r in trace if 'activations' in r]
        figure,axes=plt.subplots(3,1,figsize=(11,9),sharex=True,constrained_layout=True)
        ticks=[r['tick'] for r in trace]
        for n in range(8):axes[0].plot(ticks,[r['activations'][n]/1024 for r in trace],label=f'n{n}')
        axes[0].set_ylabel('Neural activation');axes[0].legend(ncol=8,fontsize=8)
        image=axes[1].imshow([[r['angular_rates'][axis] for r in trace] for axis in range(len(trace[0]['angular_rates']))],aspect='auto',origin='lower',extent=[ticks[0],ticks[-1],-.5,len(trace[0]['angular_rates'])-.5],cmap='RdBu_r',vmin=-64,vmax=64)
        axes[1].set_ylabel('Actuator axis');figure.colorbar(image,ax=axes[1],label='Realized rate (Q rad/tick)')
        for axis in range(3):axes[2].plot(ticks,[r['velocity'][axis]/1024 for r in trace],label='xyz'[axis])
        axes[2].set_ylabel('COM velocity (sites/tick)');axes[2].set_xlabel('Committed tick');axes[2].legend()
        figure.suptitle('Seed 1, recurrent plasticity on: motor activity without demonstrated locomotion learning')
        figure.savefig(dest/'activity.png',dpi=160);plt.close(figure)
    except ImportError:
        (dest/'PLOT_NOT_RUN.txt').write_text('matplotlib unavailable; raw traces preserved.\n')
    print(json.dumps(dict(trials=len(results),matched='PASS',independent_snapshots=len(checks),lineage=lineage,dest=str(dest))))


if __name__=='__main__':main()
