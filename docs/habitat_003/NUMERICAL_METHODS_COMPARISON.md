# Numerical methods and comparison categories

| Candidate | State / time | Numerical scope | Current qualification |
|---|---|---|---|
|001/002 D3Q19|Dense authoritative populations, fixed lattice clock|Isothermal low-Mach BGK, scalar transport, prescribed body closure|Historical qualified controls retained; fresh002 exact baseline reproduced|
|003 adaptive D3Q19|Native leaf populations; acoustic2:1|Independently authored restriction/prolongation, temporal ghosts|Uniform test passes; nonuniform interface pilot rejected; CategoryIV|
|003 FV|Native integrated mass/momentum/scalars|Isothermal Rusanov mass/normal momentum; transported transverse momentum; explicit viscosity|Focused contracts pass; final physical measurements pending|
|003 subcycling FV|Committed leaves plus provisional half-cycle|Coarse predictor and conservative integrated interface correction|Focused conservation and pending continuation pass; speedup unproven|
|GPU environmental solver|PROPOSED device transaction|All physical stages, reduction and admission resident|NOT IMPLEMENTED/NOT RUN|

CategoryI means identical complete logical state in one backend/source/precision.
CategoryII allows measured discretization error within the registered budgets.
CategoryIII compares different model forms and must compare physics/units, not
population bits. CategoryIV includes the rejected adaptive-LBM candidate. Neither
sparse storage nor conservation alone establishes physical accuracy or real time.
FV free-slip walls differ from legacy LBM bounce-back no-slip walls. There is no
thermal-energy state or calibrated dense-gas equation of state in003.
