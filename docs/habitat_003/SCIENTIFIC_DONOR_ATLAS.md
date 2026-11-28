# Scientific donor atlas

PARTIAL scientific catalog; incomplete equation-level donor qualification. Null means UNRESOLVED, not unsupported by donor. No donor executable reproduced, no code imported, no new runtime dependency. Feature pages do not establish current implementation or FORGE performance.

Each record in the JSON contains every requested evidence field; uninspected fields remain null. These are research leads, not completed donor qualifications. Two selected source revisions and their file hashes are in `SCIENTIFIC_SOURCE_IDENTITIES.json`. Full downloaded materials are external raw evidence.

| Donor | Inspected scope | Proposed independent test |
|---|---|---|
| [Cart3D](https://www.nas.nasa.gov/software/cart3d.html) | primary project page or documentation landing page; selected text, not full source | Adjoint-output refinement versus local gradient indicators |
| [AMReX AmrCore and FluxRegister](https://amrex-codes.github.io/amrex/docs_html/AmrCore.html) | AmrCore subcycling/reflux documentation and selected FluxRegister.cpp source | Conservative synchronization without importing a framework |
| [waLBerla](https://walberla.net/) | primary project page or documentation landing page; selected text, not full source | Qualify refinement-interface momentum and worker scaling |
| [Palabos: Grid-refinement in Palabos](https://palabos.unige.ch/application/files/6716/2401/0085/PalabosSummerSchool2021_MeshRefinement.pdf) | Selected full-text grid-refinement slides; page15 rendered and inspected | Check distribution notation and stress transfer before porting |
| [OpenLB](https://www.openlb.net/) | primary project page or documentation landing page; selected text, not full source | Compare unit conversion, relaxation and passive transport contracts |
| [SAMRAI](https://computing.llnl.gov/projects/samrai) | primary project page or documentation landing page; selected text, not full source | Test native ownership versus overlapping patch state |
| [p4est](https://www.p4est.org/) | primary project page or documentation landing page; selected text, not full source | Assess deterministic leaf addressing and balancing cost |
| [Chombo](https://commons.lbl.gov/spaces/chombo/overview) | primary project page or documentation landing page; selected text, not full source | Conservative interface and embedded-boundary qualification |
| [FLASH](https://flash.rochester.edu/site/flashcode/) | primary project page or documentation landing page; selected text, not full source | Separate adaptive infrastructure from each physical closure |
| [AREPO](https://arepo-code.org/) | primary project page or documentation landing page; selected text, not full source | Test moving-mesh geometric conservation before adoption |
| [GADGET-4](https://wwwmpa.mpa-garching.mpg.de/gadget4/) | primary project page or documentation landing page; selected text, not full source | Particle locality versus low-noise continuum transport |
| [Athena++](https://github.com/PrincetonUniversity/athena) | GitHub primary project and selected mesh_refinement.cpp source; Princeton homepage fetch failed | Inspect limited prolongation and distinguish MHD from atmosphere |
| [RAMSES](https://ramses-organisation.readthedocs.io/en/latest/) | primary project page or documentation landing page; selected text, not full source | Octree ownership and self-gravity are separate responsibilities |
| [Enzo](https://enzo-project.org/) | primary project page or documentation landing page; selected text, not full source | Test conservation and cost independently of cosmological claims |
| [HYPERS-Global model 2021](https://ccmc.gsfc.nasa.gov/models/HYPERS-Global~2021/) | primary project page or documentation landing page; selected text, not full source | Separate local clocks from conservative exchange causality |
| [SWMF / BATS-R-US model 20180525](https://ccmc.gsfc.nasa.gov/models/SWMF~20180525/) | primary project page or documentation landing page; selected text, not full source | Explicit feedback and interface ownership before multiphysics coupling |
| [WarpX](https://warpx.readthedocs.io/en/latest/) | primary project page or documentation landing page; selected text, not full source | Particle-field exchange and charge conservation require separate proof |
| [SPARTA Direct Simulation Monte Carlo Simulator](https://sparta.github.io/) | primary project page or documentation landing page; selected text, not full source | Knudsen transition and statistical uncertainty before any hybrid gas claim |
| [LAMMPS granular models](https://docs.lammps.org/Howto_granular.html) | primary project page or documentation landing page; selected text, not full source | Collision, friction and timestep tests before particulate solids |
| [Comparison of CFD and DSMC Using Calibrated Transport Parameters](https://ntrs.nasa.gov/api/citations/20200002567/downloads/20200002567.pdf?attachment=true) | Selected full-text NASA NTRS report | Distinguish transport matching from continuum validity |
| [Local adaptive mesh refinement for shock hydrodynamics](https://doi.org/10.1016/0021-9991(89)90035-1) | Publisher metadata and selected LBNL-hosted21-page full text | Paired integrated flux and reflux falsification controls |
| [Theory and applications of an alternative lattice Boltzmann grid refinement algorithm](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.67.066707) | Publisher abstract/metadata only; direct DOI fetch failed, full paper NOT read | Obtain full text before attributing a stress-transfer implementation |
| [Clawpack Riemann book: Approximate solvers](https://www.clawpack.org/riemann_book/html/Approximate_solvers.html) | primary project page or documentation landing page; selected text, not full source | Consistency, positivity and acoustic damping measurements |
| [hypre](https://hypre.readthedocs.io/en/latest/) | primary project page or documentation landing page; selected text, not full source | Residual tolerance, preconditioning and whole-transaction costs |
| [IBAMR](https://ibamr.github.io/) | primary project page or documentation landing page; selected text, not full source | Adjoint interpolation/spreading, force balance and mesh convergence |
| [libROM](https://www.librom.net/) | primary project page or documentation landing page; selected text, not full source | Held-out trajectories, conservation and certified residuals before speed claims |
| [ExaFMM-t](https://github.com/exafmm/exafmm-t) | primary project page or documentation landing page; selected text, not full source | Direct-sum accuracy and crossover cost before long-range adoption |
| [NairnMPM / NairnFEA](https://github.com/nairnj/nairn-mpm-fea) | primary project page or documentation landing page; selected text, not full source | Particle-grid momentum, contact and deformation convergence |
| [Chaste](https://chaste.github.io/) | primary project page or documentation landing page; selected text, not full source | Mechanics and cell-source balances before morphogenesis claims |
| [PhysiCell](https://physicell.org/) | primary project page or documentation landing page; selected text, not full source | Mass uptake, cell birth/death and microenvironment coupling qualification |
| [Morpheus](https://morpheus.gitlab.io/) | primary project page or documentation landing page; selected text, not full source | Separate phenomenological shape rules from biomechanical forces |
| [CompuCell3D](https://compucell3d.org/) | primary project page or documentation landing page; selected text, not full source | Diffusion/mechanics ordering and reproducibility before biological claims |
| [SOFA](https://www.sofa-framework.org/) | Primary URL timed out; no substantive page inspected | Reattempt primary access, then articulated-mechanics energy tests |
| [FEBio](https://febio.org/) | primary project page or documentation landing page; selected text, not full source | Constitutive calibration, large deformation and interface work |
| [COPASI](https://copasi.org/) | primary project page or documentation landing page; selected text, not full source | Stoichiometric and elemental accounting before metabolism |
| [MOOSE](https://mooseframework.inl.gov/) | primary project page or documentation landing page; selected text, not full source | Inspect phase-field implementation and free-energy budget before deposition |
| [BioDynaMo](https://biodynamo.org/) | primary project page or documentation landing page; selected text, not full source | Population accounting, interaction costs and ecological null controls |
| [NEURON](https://neuron.yale.edu/neuron/) | Primary URL timed out; no substantive page inspected | Reattempt primary docs, then compartment and synaptic timing controls |
| [NEST Simulator](https://nest-simulator.readthedocs.io/en/stable/) | primary project page or documentation landing page; selected text, not full source | Clock/event equivalence and bounded physical consumer before cognition claims |
| [DAMASK](https://damask-multiphysics.org/) | primary project page or documentation landing page; selected text, not full source | Mechanical, damage and phase energy balances before solidification |
