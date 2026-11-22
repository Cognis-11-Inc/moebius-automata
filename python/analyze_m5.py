"""Independent accounting, geometry, competence and null-result audit for M5.

Usage: python3 python/analyze_m5.py EXPANDED_EVIDENCE REPLAY_JSONL
"""
import gzip
import json
import math
import statistics as stats
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parents[1] / 'docs/m5'


def norm(v):
    return math.sqrt(sum(x*x for x in v))/1024


def delta(a, b):
    return [(y-x+16384) % 32768-16384 for x, y in zip(a, b)]


def read_trace(root, name):
    p = root/(name+'.jsonl.gz')
    opener = gzip.open
    if not p.exists():
        p = root/(name+'.jsonl')
        opener = open
    with opener(p, 'rt') as stream:
        return [json.loads(line) for line in stream]


def dataset_name(value):
    if isinstance(value, str):
        return value
    return bytes(value['Unix']).decode()


def main(expanded, replay_path):
    replay = [json.loads(line) for line in replay_path.read_text().splitlines()]
    for row in replay:
        row['dataset'] = dataset_name(row['dataset'])
    by_dataset = {r['dataset']: r for r in replay}
    groups, audit, cases, rows_by_case = [], [], {}, {}
    all_goals = {}
    forward_gap = dict(trials=0,minimum_head_distance_squared=10**9,maximum_head_y=-1)
    for root in sorted(p for p in expanded.iterdir() if p.is_dir()):
        data = json.loads((root/'results.json').read_text())
        assert data['failures'] == 0, 'preserved execution failures require inspection'
        replayed = by_dataset[root.name]
        expected = {c['name']: c for c in replayed['cases']}
        trial_count = 0
        ticks = 0
        successes = 0
        goals = []
        kin_sum = []
        for trial in data['results']:
            if 'case' not in trial:
                assert trial['distance_squared'] == 9
                continue
            case = trial['case']
            name = case['name']
            rows = read_trace(root, name)
            forward_case = root.name in ('screen_v1','periodic_v1') and case['drag']==8 and case['energy']==128
            if forward_case:
                forward_gap['trials'] += 1
            assert len(rows) == trial['ticks']
            acct = expected[name]
            energy = case['energy']
            field = 512*bool(case['target_delta'])+bool(case['founding_bite'])
            waste = 0
            exported = 0
            intake = 0
            fees = [0]*5
            motor_debt = 0
            prev_angles = [0]*26
            prior_mouth = trial['initial_mouth']
            target_tick = None
            founding_site = None
            first_signal = None
            path = 0.
            first_stop = None
            zero_start = None
            for expected_tick, row in enumerate(rows):
                assert row['tick'] == expected_tick
                r, receipt = row['organism'], row['receipt']
                assert energy == r['energy_before']
                spend = sum(r[k] for k in ('maintenance','neural','actuator','secretion','digging','birth_out','death_disposal'))
                assert energy+r['gain'] == r['energy_after']+spend
                energy = r['energy_after']
                assert r['gain'] == 4*r['intake'] == receipt['energy_credited']
                assert r['intake'] == receipt['nutrient_uptaken']
                assert receipt['energy_maintenance'] == r['maintenance']+r['neural']+r['actuator']
                assert receipt['energy_disposed'] == r['secretion']+r['digging']+r['death_disposal']
                assert receipt['external_input'] == 0
                assert receipt['births'] == r['birth_out'] == 0
                assert all(receipt[k] == 0 for k in ('jet_released','jet_deposited','jet_wasted','secreted','excavated','trail_laid'))
                field -= r['intake']
                waste += r['intake']-receipt['external_output']
                exported += receipt['external_output']
                assert field+waste+exported == acct['initial_field']
                assert int(row['material_residual']) == int(row['energy_residual']) == 0
                for i, k in enumerate(('maintenance','neural','actuator','secretion','digging')):
                    fees[i] += r[k]
                effort = sum(abs(c*s//1024) if c*s >= 0 else abs(-((-c*s)//1024))
                             for c,s in zip(r['drive'][:26], acct['strengths'][:26]))//8
                total = motor_debt+effort
                assert total//4096 == r['actuator']
                motor_debt = total % 4096
                assert row['mouth_before'] == prior_mouth
                prior_mouth = row['mouth_after']
                actual = [a-b for a,b in zip(r['angles'],prev_angles)]
                assert actual == r['rates']
                prev_angles = r['angles']
                movement = delta(r['position_before'],r['position_after'])
                assert movement == row['com_delta']
                path += norm(movement)
                head = [(v+512)//1024 % 32 for v in row['mouth_after']]
                assert head == [r['head'][a] for a in ('x','y','z')]
                if forward_case:
                    food = [trial['target'][a] for a in ('x','y','z')]
                    d2 = sum((a-b)**2 for a,b in zip(head,food))
                    forward_gap['maximum_head_y'] = max(forward_gap['maximum_head_y'],head[1])
                    if d2 < forward_gap['minimum_head_distance_squared']:
                        forward_gap['minimum_head_distance_squared'] = d2
                        forward_gap['nearest_sample'] = dict(dataset=root.name,name=name,tick=row['tick'],head=head,energy_before=r['energy_before'])
                if not r['died']:
                    assert row['velocity_after_if_alive'] == movement
                    assert row['velocity_after_if_alive'] == [int(v/16) for v in row['momentum_after_if_alive']]
                    assert [a+b for a,b in zip(row['environment_momentum'],row['momentum_after_if_alive'])] == [0]*3
                else:
                    assert row['environment_momentum'] == [0]*3
                if r['intake']:
                    food = [r['food_site'][a] for a in ('x','y','z')]
                    distances = [((a-b+16) % 32)-16 for a,b in zip(head,food)]
                    assert sum(d*d for d in distances) <= 2
                    assert r['food_available_at_attempt'] >= r['intake']
                    if founding_site is None:
                        founding_site = r['food_site']
                    if r['food_site'] == trial['target'] and target_tick is None:
                        target_tick = row['tick']
                if any(row['sensors_before'][93:99]) and first_signal is None:
                    first_signal = row['tick']
                if not r['died'] and row['velocity_after_if_alive'] == [0]*3:
                    if zero_start is None:
                        zero_start = row['tick']
                    if row['tick']-zero_start >= 7 and first_stop is None:
                        first_stop = zero_start
                else:
                    zero_start = None
                intake += r['intake']
            assert abs(path-trial['com_path']) < 1e-10
            assert fees == trial['costs_maintenance_neural_actuator_secretion_digging']
            assert target_tick == trial['first_target_tick']
            assert first_signal == trial['first_chemical_signal_tick']
            assert field == acct['final_field'] and waste == acct['final_waste'] and exported == acct['external_output']
            assert energy == acct['live_energy'] == trial['final_energy']
            if acct['actuator_debt_final_if_alive'] is not None:
                assert motor_debt == acct['actuator_debt_final_if_alive']
            if target_tick is not None:
                successes += 1
                bite = rows[target_tick]
                if case['founding_bite']:
                    assert rows[0]['organism']['intake'] == 1
                    assert rows[0]['organism']['food_site'] != trial['target']
                event = dict(dataset=root.name,name=name,seed=case['seed'],target_delta=case['target_delta'],
                    tick=target_tick, founding_bite=case['founding_bite'],
                    first_signal=first_signal,energy=case['energy'],drag=case['drag'],mouth_to_food_at_bite=bite['mouth_to_target'],com_displacement_to_bite=norm(delta(trial['initial_com'],bite['organism']['position_after'])),
                    mouth_displacement_to_bite=norm(delta(trial['initial_mouth'],bite['mouth_after'])),
                    spending_to_bite=sum(sum(r['organism'][k] for k in ('maintenance','neural','actuator','secretion','digging')) for r in rows[:target_tick+1]))
                goals.append(event)
            cases[(root.name,name)] = dict(trial=trial,pending_actuator_energy=motor_debt/4096.,
                                           first_eight_tick_zero_start=first_stop)
            # Keep only brake / differential-control rows for causal comparisons.
            if root.name == 'braking_v1':
                rows_by_case[name] = rows
            trial_count += 1
            ticks += len(rows)
            kin_sum.append(dict(name=name,com_net=trial['com_net'],mouth_net=trial['mouth_net'],com_path=trial['com_path']))
        assert trial_count == replayed['trials'] and ticks == replayed['ticks']
        audit.append(dict(dataset=root.name,status='PASS',trials=trial_count,organism_ticks=ticks,
                          sensor_probes=replayed['sensor_probes'],target_successes=successes))
        kinds = sorted({t['controller'].get('kind','schedule') for t in data['results'] if 'case' in t})
        for kind in kinds:
            ts = [t for t in data['results'] if 'case' in t and t['controller'].get('kind','schedule') == kind]
            groups.append(dict(dataset=root.name,controller=kind,trials=len(ts),
                successes=sum(t['first_target_tick'] is not None for t in ts),
                deaths=sum(t['death_tick'] is not None for t in ts),
                mean_com_path=stats.mean(t['com_path'] for t in ts),
                mean_motor_energy=stats.mean(t['costs_maintenance_neural_actuator_secretion_digging'][2] for t in ts),
                by_target=[dict(delta=d, trials=sum(t['case']['target_delta']==d for t in ts),successes=sum(t['case']['target_delta']==d and t['first_target_tick'] is not None for t in ts))
                           for d in [list(v) for v in sorted({tuple(t['case']['target_delta']) for t in ts if t['case']['target_delta']})]]))
        all_goals[root.name] = goals
    # Paired joint-axis ablations; nonlinear effects are not additive forces.
    attr = [(key[1],v['trial']) for key,v in cases.items() if key[0]=='attribution_v1']
    base = next(t for name,t in attr if t['controller']['schedule']['ablate'] is None and t['controller']['schedule']['pattern']=='bend_twist')
    joint = []
    single_axis = []
    for _,t in attr:
        schedule = t['controller']['schedule']
        if schedule['pattern'].startswith('axis_'):
            single_axis.append(dict(axis=int(schedule['pattern'][5:]),amplitude=schedule['amplitude'],com_net=t['com_net'],mouth_net=t['mouth_net'],rotation=t['rotation'],paid_actuator=t['costs_maintenance_neural_actuator_secretion_digging'][2],pending_actuator_energy=cases[('attribution_v1',t['case']['name'])]['pending_actuator_energy']))
        axis = schedule['ablate']
        if axis is not None:
            joint.append(dict(axis=axis,com_net_without_axis=t['com_net'],com_net_full=base['com_net'],
                              change_in_com_net=t['com_net']-base['com_net'],
                              change_in_mouth_net=t['mouth_net']-base['mouth_net'],
                              change_in_rotation=t['rotation']-base['rotation'],
                              change_in_paid_actuator=t['costs_maintenance_neural_actuator_secretion_digging'][2]-base['costs_maintenance_neural_actuator_secretion_digging'][2]))
    brakes = []
    release = rows_by_case['brake_release']
    for name,rows in rows_by_case.items():
        assert [r['state_hash'] for r in rows[:16]] == [r['state_hash'] for r in release[:16]]
        suffix = rows[16:]
        brakes.append(dict(name=name,first_eight_tick_zero_start=cases[('braking_v1',name)]['first_eight_tick_zero_start'],
                           spending_after_switch=sum(r['organism']['actuator'] for r in suffix),
                           minimum_speed_after_switch=min(norm(r['velocity_after_if_alive']) for r in suffix if r['velocity_after_if_alive'] is not None),
                           final_momentum_last_living=next(r['momentum_after_if_alive'] for r in reversed(rows) if r['momentum_after_if_alive'] is not None),
                           death_tick=cases[('braking_v1',name)]['trial']['death_tick']))
    # Controlled steering comparison at a common living time, after same paid launch.
    steering=[]
    for tick in (31,47,63):
        for name in ('brake_turn','brake_reverse8'):
            va=release[tick]['velocity_after_if_alive'];vb=rows_by_case[name][tick]['velocity_after_if_alive']
            dot=sum(a*b for a,b in zip(va,vb));length=math.sqrt(sum(a*a for a in va)*sum(b*b for b in vb))
            angle=math.degrees(math.acos(max(-1,min(1,dot/length)))) if length else None
            steering.append(dict(name=name,tick=tick,release_velocity=va,controlled_velocity=vb,heading_difference_degrees=angle))
    # Descriptive screen maxima, never claims of global motor optimality.
    screen=[]
    for dataset in ('screen_v1','periodic_v1'):
        for drag in (0,8,32,128):
            for energy in (128,2048):
                ts=[v['trial'] for key,v in cases.items() if key[0]==dataset and v['trial']['case']['drag']==drag and v['trial']['case']['energy']==energy]
                if ts:
                    best=max(ts,key=lambda t:t['com_net'])
                    screen.append(dict(dataset=dataset,drag=drag,energy=energy,trials=len(ts),target_successes=sum(t['first_target_tick'] is not None for t in ts),
                                       maximum_com_net=best['com_net'],maximum_case=best['case']['name'],
                                       maximum_mouth_net=max(t['mouth_net'] for t in ts),
                                       paid_actuator_for_maximum=best['costs_maintenance_neural_actuator_secretion_digging'][2],
                                       com_path_for_maximum=best['com_path'],
                                       paid_actuator_per_path=best['actuator_energy_per_com_path'],
                                       pending_actuator_energy=cases[(dataset,best['case']['name'])]['pending_actuator_energy']))
    sensor=[]
    for dataset in ('sensors_v1','sensors_lobes_v1'):
        ts=json.loads((expanded/dataset/'results.json').read_text())['results']
        for anchor in ('com','mouth'):
            for orientation in ([0,0,0],[0,0,700],[0,0,804],[0,600,600]):
                selected=[t for t in ts if t['anchor']==anchor and t['orientation']==orientation]
                sensor.append(dict(dataset=dataset,anchor=anchor,orientation=orientation,
                                   aligned_visible=sum(any(t['chemical']) for t in selected[:6]),
                                   off_axis_visible=sum(any(t['chemical']) for t in selected[6:]),
                                   aligned_trials=6,off_axis_trials=24))
    result=dict(independent_audit=audit,groups=groups,screen=screen,joint_ablation=joint,braking=brakes,
                steering=steering,sensors=sensor,single_axis=single_axis,forward_gap=forward_gap,limits='Finite diagnostic search. Execution PASS is not a reachability PASS. Mouth capture is rounded-head lattice distance squared <=2. Model controller is authored, not learned. Paid actuator energy and pending fractional debt are distinct.')
    (OUT/'GOAL_EVENTS.json').write_text(json.dumps(all_goals,indent=2)+'\n')
    (OUT/'ANALYSIS.json').write_text(json.dumps(result,indent=2)+'\n')
    (OUT/'REPLAY_AUDIT.json').write_text(json.dumps(replay,indent=2)+'\n')
    print(json.dumps(audit,indent=2))


if __name__ == '__main__':
    main(Path(sys.argv[1]),Path(sys.argv[2]))
