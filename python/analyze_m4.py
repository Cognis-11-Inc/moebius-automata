"""Independent M4 receipt audit and descriptive comparisons (stdlib only)."""
import gzip
import json
import math
import statistics as stats
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'docs/m4'


def mean(values):
    return stats.mean(values) if values else None


def correlation(a, b):
    if not a or len(a) != len(b):
        return None
    ma, mb = stats.mean(a), stats.mean(b)
    va, vb = sum((x-ma)**2 for x in a), sum((x-mb)**2 for x in b)
    return sum((x-ma)*(y-mb) for x, y in zip(a, b)) / math.sqrt(va*vb) if va*vb else None


def main():
    groups = []
    audits = []
    for directory in sorted((ROOT / 'artifacts').iterdir()):
        results_path = directory / 'results.json'
        if not results_path.exists():
            continue
        data = json.loads(results_path.read_text())
        for env in sorted({t['environment'] for t in data['results']}):
            for controller in sorted({t['controller'] for t in data['results'] if t['environment'] == env}):
                trials = [t for t in data['results'] if t['environment'] == env and t['controller'] == controller]
                successful = [t for t in trials if t['status'] == 'PASS']
                organisms = [o for t in successful for o in t['organisms']]
                founders = [o for t in successful for o in t['organisms']
                            if o['id'] <= (192 if directory.name.startswith('production') else 12 if env == 'ecology' else 1)]
                groups.append(dict(dataset=directory.name, environment=env, controller=controller,
                    trials=len(trials), failures=len(trials)-len(successful),
                    trials_with_food=sum(any(o['intake'] for o in t['organisms']) for t in successful),
                    final_live=sum(t['final_live'] for t in successful), births=sum(t['births'] for t in successful),
                    intake=sum(o['intake'] for o in organisms),
                    mean_founder_path=mean([o['path'] for o in founders]),
                    mean_founder_net=mean([o['net_displacement'] for o in founders]),
                    mean_founder_rotation=mean([o['rotation_radians_approx'] for o in founders]),
                    mean_death_tick=mean([o['death_tick'] for o in organisms if o['death_tick'] is not None]),
                    costs=[sum(o['costs_maintenance_neural_motor_secretion_digging'][i] for o in organisms) for i in range(5)],
                    meals_at_other_food_sites=sum(o['meals_at_other_food_sites'] for o in organisms),
                    false_motion_ticks=sum(sum(o['false_motion_ticks']) for o in organisms),
                    near_unfed=sum(o['near_food_without_eating'] for o in organisms),
                    mean_joint_active_ticks=[mean([o['actual_active'][i] for o in founders]) for i in range(26)],
                    mean_joint_amplitude=[mean([o['amplitude'][i] for o in founders]) for i in range(26)]))
        rows_seen = 0
        actual_false = 0
        max_commands = {}
        for trial in data['results']:
            if trial['status'] != 'PASS':
                continue
            prefix = f"{trial['environment']}_{trial['controller']}_{trial['seed']}"
            trace = directory / (prefix+'.jsonl.gz')
            opener = gzip.open
            if not trace.exists():
                trace = directory / (prefix+'.jsonl')
                opener = open
            total_gain = 0
            total_costs = [0]*5
            per_organism = {}
            with opener(trace, 'rt') as stream:
                for line in stream:
                    row = json.loads(line)
                    r = row['organism']
                    maximum = max(map(abs, r['commands']), default=0)
                    label = trial['controller']
                    max_commands[label] = max(max_commands.get(label, 0), maximum)
                    if label == 'fixed_wave_only':
                        assert maximum == 0, 'fixed output control contaminated'

                    assert int(row['energy_balance_residual']) == 0
                    spending = sum(r[k] for k in ('maintenance','neural','actuator','secretion','digging','birth_out','death_disposal'))
                    assert r['energy_before'] + r['gain'] == r['energy_after'] + spending
                    total_gain += r['gain']
                    for i,k in enumerate(('maintenance','neural','actuator','secretion','digging')):
                        total_costs[i] += r[k]
                    per_organism[r['id']] = per_organism.get(r['id'], 0) + r['intake']
                    actual_false += sum(a == 0 and b != 0 for a,b in zip(row['actual_rates'],r['rates']))
                    rows_seen += 1
            assert total_gain == sum(o['gain'] for o in trial['organisms'])
            assert total_costs == [sum(o['costs_maintenance_neural_motor_secretion_digging'][i] for o in trial['organisms']) for i in range(5)]
            assert per_organism == {o['id']:o['intake'] for o in trial['organisms']}
            assert int(trial['material_residual']) == int(trial['energy_residual']) == 0
        audits.append(dict(dataset=directory.name, status='PASS', individual_ticks=rows_seen,
                           trials=len(data['results']), failures=data['failures'], false_motion_ticks=actual_false,
                           maximum_raw_command_by_controller=max_commands))
    # A descriptive phase probe, not an optimized coordination score.
    coordination = []
    for dataset in ('baseline_v1','corrected_v2'):
        for controller in ('current','no_wave','wave_only','random'):
            path=ROOT/'artifacts'/dataset/f'free_{controller}_1.jsonl.gz'
            with gzip.open(path,'rt') as stream:
                rows=[json.loads(line)['actual_rates'] for line in stream]
            a=[r[0] for r in rows]
            for index in (2,4,12,22,1,24):
                b=[r[index] for r in rows]
                lags={str(lag):correlation(a[max(0,lag):len(a)+min(0,lag)], b[max(0,-lag):len(b)-max(0,lag)]) for lag in range(-12,13)}
                coordination.append(dict(dataset=dataset, controller=controller, reference_axis=0,
                                         other_axis=index, correlations_by_lag=lags))
    output=dict(groups=groups, independent_audit=audits, coordination_probe=coordination,
                limits='Descriptive matched seeds; no learning or control-optimization test. Distance is geometric, not a collision-free route. Death ticks are zero based. Raw sensors and joint arrays remain in traces.')
    (ROOT/'ANALYSIS.json').write_text(json.dumps(output,indent=2)+'\n')
    print(json.dumps(audits,indent=2))


if __name__ == '__main__':
    main()
