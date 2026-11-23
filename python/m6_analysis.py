"""Recompute M6 summaries and independent trace accounting; stdlib only.
Usage: python3 python/m6_analysis.py RESTORED_PHASE_DIRECTORY OUTPUT.json
Phase directories are capability-v1, optimize-v1, etc. (see m6_artifacts.py).
"""
import gzip
import json
import math
import random
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path


def summary(values):
    return dict(n=len(values), mean=st.mean(values), median=st.median(values),
                minimum=min(values), maximum=max(values))


def interval(values):
    rng = random.Random(20261122)
    means = sorted(st.mean(rng.choices(values, k=len(values))) for _ in range(10000))
    return dict(**summary(values), mean_bootstrap_95=[means[249], means[9749]],
                positive=sum(x > 0 for x in values), values=values)


def load(root, phase, file='measurements.json'):
    return json.loads((root / phase / file).read_text())


def grouped(rows):
    groups = defaultdict(list)
    for row in rows:
        groups[row.get('control', row.get('mode'))].append(row.get('measurement', row))
    return {k: dict(episodes=len(ms), sustained=sum(m['sustained'] for m in ms),
                    jets=sum(m['jets'] for m in ms),
                    **{field: summary([m[field] for m in ms]) for field in
                       ['net', 'path', 'active_gain', 'paid_motor', 'rotation', 'active_axes']},
                    energy_per_net=[m['energy_per_net'] for m in ms])
            for k, ms in groups.items()}


def trunc(a, b):
    return (abs(a) // b) * (-1 if a < 0 else 1)


def audit(path):
    """Independent arithmetic from saved states; does not call Rust core.
    Full native replay separately checks all state fields and secretion momentum.
    """
    episodes = ticks = cues = jet_ticks = exported = 0
    max_momentum = 0
    details = {}
    with gzip.open(path, 'rt') as stream:
        for line in stream:
            v = json.loads(line)
            if v['type'] == 'start':
                begin = v
                previous = v['initial_neural_state']
                genome = previous['genome']
                assert {b['axis'] for b in genome['anatomy']['axes']} == {'Bend','Twist','Jaw'}, 'unknown serialized axis schema'
                total = [0., 0., 0.]
                travel = paid = energy_used = 0
                command_min = [1024] * 26
                command_max = [-1024] * 26
                late_lunges = 0
                late_impulse_ticks = late_impulse_L1 = late_drag_L1 = 0
                cues_before = cues
                field_total = sum(v.get('initial_field', []))
            elif v['type'] == 'cue':
                cues += 1
                assert sum(v['field']) == field_total
            elif v['type'] == 'tick':
                ticks += 1
                n, o, r = v['neural'], v['organism'], v['receipt']
                assert len(n['sensors']) == 107 and len(n['commands']) == 30
                cost = sum(o[k] for k in ['maintenance', 'neural', 'actuator', 'digging', 'secretion', 'birth_out', 'death_disposal'])
                assert o['energy_after'] == o['energy_before'] + o['gain'] - cost
                assert r['external_input'] == r['nutrient_uptaken'] == 0
                exported += r['external_output']
                field_total += r['reservoir_to_field'] - r['nutrient_uptaken'] - r['secreted'] - r['jet_released']
                assert not o['died'] and r['births'] == r['deaths'] == 0
                energy_used += cost
                # Actual smoothed joint drives, original strengths, fractional debt.
                effort = sum(abs(trunc(d * s, 1024)) for d, s in zip(o['drive'][:26], genome['strengths'])) // 8
                debt = previous['actuator_debt'] + effort
                assert o['actuator'] == debt // 4096 and n['actuator_debt'] == debt % 4096
                assert o['neural'] == int(n['steps'] % 8 == 0)
                paid += o['actuator']
                d = [((b-a+8192) % 16384-8192)/1024 for a,b in zip(o['position_before'],o['position_after'])]
                if begin['mode'] != 'random':
                    assert [round(x*1024) for x in d] == n['body']['velocity'], (path, v['name'], v['tick'], d, n['body']['velocity'])
                assert n['body']['velocity'] == [trunc(p,16) for p in n['body']['momentum']]
                for i in range(3): total[i] += d[i]
                travel += math.sqrt(sum(x*x for x in d))
                jet_ticks += int(r['jet_released'] > 0)
                # This independent ledger covers no-jet ticks only. Full native replay covers jets.
                if r['jet_released'] == 0 and r['secreted'] == 0 and begin['mode'] != 'random':
                    residual = [a+b for a,b in zip(v['environment_momentum'],n['body']['momentum'])]
                    max_momentum = max(max_momentum, *map(abs,residual))
                    assert residual == [0,0,0]
                for i,c in enumerate(n['commands'][:26]):
                    command_min[i] = min(command_min[i], c)
                    command_max[i] = max(command_max[i], c)
                if v['tick'] >= begin['episode']['ticks']//2:
                    damp = [trunc(p*begin['episode']['drag'],1024) for p in previous['body']['momentum']]
                    impulse = [new-old+drag for new,old,drag in zip(n['body']['momentum'],previous['body']['momentum'],damp)]
                    late_impulse_ticks += int(any(impulse))
                    late_impulse_L1 += sum(map(abs,impulse))
                    late_drag_L1 += sum(map(abs,damp))
                signed_bend = sum(rate for rate, binding in zip(n['body']['rates'],genome['anatomy']['axes']) if binding['axis'] == 'Bend')
                if v['tick'] >= begin['episode']['ticks']//2 and abs(signed_bend)>280: late_lunges += 1
                previous = n
            elif v['type'] == 'end':
                episodes += 1
                m = v['measurement']
                assert abs(math.sqrt(sum(x*x for x in total))-m['net']) < 1e-10
                assert abs(travel-m['path']) < 1e-10 and paid == m['paid_motor']
                assert abs(m['pending_motor']-previous['actuator_debt']/4096) < 1e-12
                # Effective cost includes incoming debt only in carry-fast probes; disclose separately.
                if 'selected_frozen' in m['name'] or 'bias_only' in m['name'] or 'vacuum' in m['name'] or 'shuffled' in m['name']:
                    details[m['name']] = dict(command_ranges=[b-a for a,b in zip(command_min,command_max)],
                        late_bend_rate_threshold_ticks=late_lunges,
                        late_inferred_medium_impulse_ticks=late_impulse_ticks,
                        late_inferred_medium_impulse_L1=late_impulse_L1, late_drag_L1=late_drag_L1,
                        initial_actuator_debt=begin['initial_neural_state']['actuator_debt'],
                        energy_used=energy_used, cues=cues-cues_before)
    return dict(status='PASS', episodes=episodes, ticks=ticks, cue_events=cues,
                jet_release_ticks=jet_ticks, material_exported=exported, no_jet_linear_residual=max_momentum,
                scope='COM/path/velocity, energy balance, actual smoothed motor fees and debt, sensor ABI, no births/intake/input, recorded waste export, cue material conservation; independent no-jet ledger. Native replay separately includes all jets and full fields.',
                selected_mechanism=details)


def main(root, output):
    held=load(root,'heldout-v1')
    paired={}
    for r in held: paired.setdefault(r['measurement']['episode']['seed'],{})[r['control']]=r['measurement']['net']
    result={'date_local':'2026-11-22', 'statistics':'10,000 deterministic paired bootstrap resamples over 8 fixed starts; descriptive uncertainty conditional on one optimizer seed, not a population estimate across independent searches.',
            'capability':grouped(load(root,'capability-v1')),
            'babble':grouped(load(root,'babble-v1')),
            'heldout':grouped(held),
            'selected_minus_original':interval([p['selected_frozen']-p['original_frozen'] for p in paired.values()]),
            'falsification':grouped(load(root,'falsify-v1'))}
    result['evolution']={}
    for phase in ['optimize-v1','direction-v1','direction-v2']:
        history=load(root,phase,'history.json'); gens=defaultdict(list)
        with gzip.open(root/phase/'candidate_genomes.jsonl.gz','rt') as stream:
            candidates=[json.loads(line) for line in stream]
        for c in candidates: gens[c['generation']].append(c)
        diversity=[]
        for g,cs in gens.items():
            vectors=[[x for key in ['recurrent','sensory','output','biases','motor_biases','tau'] for x in c['genome'][key]] for c in cs]
            distances=[st.mean(abs(a-b) for a,b in zip(v,w)) for i,v in enumerate(vectors) for w in vectors[i+1:]]
            diversity.append(dict(generation=g,unique=len({c['genome_id'] for c in cs}),mean_pairwise_NN_parameter_L1=st.mean(distances),scores=summary([h['score'] for h in history if h['generation']==g])))
        result['evolution'][phase]=dict(selection=load(root,phase,'selection.json'),generations=diversity,
            selected_training=[h for h in history if h['genome_id']==load(root,phase,'selection.json')['selected']][-1])
    lives=load(root,'lifetime-v1'); life={}
    for controller in ['original','selected']:
        for arm in ['plasticity_on','plasticity_off','feedback_off','untrained']:
            rows=[r for r in lives if r['controller']==controller and r['arm']==arm and r['measurement']['episode']['seed']>=600]
            probes=defaultdict(dict)
            for r in rows:probes[r['measurement']['episode']['seed']][r['probe']]=r['measurement']
            life[controller+'_'+arm]=dict(changed_nonzero_synapses=rows[0]['changed_nonzero_synapses'],
                pre_net=summary([p['pre']['net'] for p in probes.values()]),
                frozen_minus_pre=interval([p['post_frozen']['net']-p['pre']['net'] for p in probes.values()]),
                inherited_minus_pre=interval([p['fresh_inherited']['net']-p['pre']['net'] for p in probes.values()]),
                carry_minus_pre=interval([p['post_carry']['net']-p['pre']['net'] for p in probes.values()]),
                frozen_sustained=sum(p['post_frozen']['sustained'] for p in probes.values()),
                parent=rows[0]['parent_genome'],inherited=rows[0]['inherited_genome'])
    result['lifetime']=life
    result['direction']={}
    for phase in ['direction-heldout-v1','direction-heldout-v2']:
        rows=load(root,phase)
        result['direction'][phase]=dict(status='INVALID_CUE' if phase.endswith('v1') else 'VALID_CUE_BOUNDED_NULL',controls=grouped(rows),
            phase_progress={c:dict(passes=sum(r['direction_pass'] for r in rows if r['control']==c),
                first=summary([r['measurement']['directional_phases'][0] for r in rows if r['control']==c]),
                second=summary([r['measurement']['directional_phases'][1] for r in rows if r['control']==c])) for c in {r['control'] for r in rows}})
    result['independent_audit']={}
    for phase in sorted(p for p in root.iterdir() if (p/'episodes.jsonl.gz').is_file()):
        print('audit',phase.name,flush=True)
        result['independent_audit'][phase.name]=audit(phase/'episodes.jsonl.gz')
    Path(output).write_text(json.dumps(result,indent=2)+'\n')

if __name__=='__main__':main(Path(sys.argv[1]),sys.argv[2])
