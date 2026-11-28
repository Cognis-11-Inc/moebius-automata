# Long-horizon scientific map — consulted leads, no biology implementation

The40 individual donor records include12 long-horizon entries. Field-level
provenance and unknowns live in SCIENTIFIC_DONOR_PROVENANCE.json and the permanent
Atlas library. Failed SOFA/NEURON page access remains FAILED ACCESS, not a source read.

| Future responsibility | Consulted leads | Required local proof before any claim |
|---|---|---|
|Morphogenesis/growth|Chaste, PhysiCell, Morpheus, CompuCell3D|Constitutive/source laws; birth/death and material balances; null morphology controls|
|Biomechanics/articulation|FEBio; SOFA access failed; NairnMPM|Force/work and contact balance; rig versus physical deformation distinction|
|Fluid–structure|IBAMR, FEBio|Paired interpolation/spreading, interface power, convergence and body reaction|
|Reaction–diffusion/metabolism|COPASI, Morpheus, PhysiCell|Stoichiometric/elemental conservation; reaction clocks; reproducible uptake/source budgets|
|Phase/solidification/deposition|MOOSE, DAMASK, granular LAMMPS|Free-energy and latent-energy budget; phase transport; surface material ownership|
|Ecology/populations|BioDynaMo, PhysiCell, Chaste|Founders versus births, deaths, uptake, resource exhaustion and sustained reproduction; no scripted flourishing|
|Neural computation|NEST; NEURON access failed|Validated bounded neural dynamics and clock integration; no inferred cognition or autonomy|

These are future independent qualification questions, not implementations hidden
behind a donor list. Equation/algorithm-level reviews remain incomplete for most
entries. MORPH005 stays PARTIAL and paused; MORPH006 suspended. No organism,
chemistry, topology or ship-runtime behavior was changed by HABITAT003.
