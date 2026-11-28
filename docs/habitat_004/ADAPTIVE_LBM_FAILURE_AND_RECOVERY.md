# Adaptive LBM: exact failure audit, no repaired candidate

Original scientific/lbm.rs is unchanged. Independent Python reproduction matches
every population of the original rejected candidate exactly. A per-direction
volumetric streaming audit separates coarse, fine-half1 and fine-half2 incoming/
outgoing inventory. The net unpaired interlevel transfer is mass1.108082692e-7kg,
x-momentum-5.595405591e-6kg*m/s, agreeing with actual failed global residuals to
roundoff. Fine-half1 cancels half the coarse exchange; fine-half2 changes it while
the coarse tentative full update still consumes the old restricted fine snapshot.
There is no shared integrated interface register. This establishes the mechanism
for that seed, not a universal diagnosis of all LBM schemes or a stress-transfer fix.

Both original homogeneous coarse and homogeneous fine wave controls complete100
cycles without conservation rejection. The interface presence is an isolated seam,
not a failure of the whole discipline. Candidate state, refusal, digests, source
and inputs are retained. No population normalization or moment repair was added.

Rohde/Kandhai/Derksen/van den Akker DOI10.1002/fld.1140 is now verified from publisher
and author institution, resolving003's bibliographic uncertainty. Publisher abstract
uses volumetric particle propagation rather than spatial/temporal density interpolation.
Full paper/implementation not reproduced. A revised local candidate needs a documented
19-population/time-area exchange and stress law, not just forcing M/P totals. Missing
transfer/tau/time ratio/flux/duplicate ownership/premature-commit falsification remains
conditional on that new candidate. Revised conservation E58 is NOT RUN.
