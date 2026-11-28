#!/usr/bin/env python3
"""Render the consulted donor catalog; no donor runtime code or dependencies.

Source facts were read during the 2026-11-27 research session. Null fields are
uninspected, not absent in the donor. Re-running renders metadata, not research.
"""
import json
from pathlib import Path

DOC = Path(__file__).resolve().parents[1] / 'docs/habitat_003'
ROWS = '''cart3d|Cart3D|NASA Ames / NAS|https://www.nas.nasa.gov/software/cart3d.html|Adaptive Cartesian Euler flow; output-weighted adaptation|Adjoint-output refinement versus local gradient indicators
amrex|AMReX AmrCore and FluxRegister|AMReX collaboration / LBNL|https://amrex-codes.github.io/amrex/docs_html/AmrCore.html|Block AMR; subcycling; flux-register synchronization|Conservative synchronization without importing a framework
walberla|waLBerla|waLBerla project|https://walberla.net/|Parallel block-structured multiphysics and lattice Boltzmann|Qualify refinement-interface momentum and worker scaling
palabos|Palabos: Grid-refinement in Palabos|Orestis Malaspinas / University of Geneva|https://palabos.unige.ch/application/files/6716/2401/0085/PalabosSummerSchool2021_MeshRefinement.pdf|LBM acoustic refinement; nonequilibrium transfer|Check distribution notation and stress transfer before porting
openlb|OpenLB|OpenLB project|https://www.openlb.net/|Lattice Boltzmann project|Compare unit conversion, relaxation and passive transport contracts
samrai|SAMRAI|Lawrence Livermore National Laboratory|https://computing.llnl.gov/projects/samrai|Structured patch hierarchy with ghost cells|Test native ownership versus overlapping patch state
p4est|p4est|p4est project|https://www.p4est.org/|Parallel forest of octrees|Assess deterministic leaf addressing and balancing cost
chombo|Chombo|Lawrence Berkeley National Laboratory|https://commons.lbl.gov/spaces/chombo/overview|Block AMR finite-volume and finite-difference infrastructure|Conservative interface and embedded-boundary qualification
flash|FLASH|FLASH Center / University of Rochester|https://flash.rochester.edu/site/flashcode/|Multiphysics AMR framework|Separate adaptive infrastructure from each physical closure
arepo|AREPO|AREPO project|https://arepo-code.org/|Moving Voronoi finite-volume hydrodynamics|Test moving-mesh geometric conservation before adoption
gadget4|GADGET-4|V. Springel, R. Pakmor, O. Zier, M. Reinecke|https://wwwmpa.mpa-garching.mpg.de/gadget4/|N-body and smoothed particle hydrodynamics|Particle locality versus low-noise continuum transport
athena|Athena++|Princeton University / Athena++ collaboration|https://github.com/PrincetonUniversity/athena|Astrophysical fluid solver with mesh refinement|Inspect limited prolongation and distinguish MHD from atmosphere
ramses|RAMSES|Romain Teyssier / RAMSES project|https://ramses-organisation.readthedocs.io/en/latest/|Adaptive Eulerian astrophysical multiphysics|Octree ownership and self-gravity are separate responsibilities
enzo|Enzo|Enzo project|https://enzo-project.org/|Adaptive astrophysical simulation framework|Test conservation and cost independently of cosmological claims
hypers|HYPERS-Global model 2021|Yuri A. Omelchenko / NASA CCMC model registry|https://ccmc.gsfc.nasa.gov/models/HYPERS-Global~2021/|Kinetic ions, fluid electrons, radiation-free field model; event scheduling|Separate local clocks from conservative exchange causality
swmf|SWMF / BATS-R-US model 20180525|University of Michigan / NASA CCMC model registry|https://ccmc.gsfc.nasa.gov/models/SWMF~20180525/|Block-adaptive finite-volume MHD within coupled models|Explicit feedback and interface ownership before multiphysics coupling
warpx|WarpX|WarpX collaboration|https://warpx.readthedocs.io/en/latest/|Particle-in-cell plasma framework|Particle-field exchange and charge conservation require separate proof
sparta|SPARTA Direct Simulation Monte Carlo Simulator|Sandia National Laboratories|https://sparta.github.io/|DSMC particles grouped in hierarchical collision cells|Knudsen transition and statistical uncertainty before any hybrid gas claim
lammps|LAMMPS granular models|LAMMPS collaboration|https://docs.lammps.org/Howto_granular.html|Discrete-element granular interactions|Collision, friction and timestep tests before particulate solids
nasadsmc|Comparison of CFD and DSMC Using Calibrated Transport Parameters|D. S. Liechty, A. J. Wise, S. Subramaniam, K. A. Stephani|https://ntrs.nasa.gov/api/citations/20200002567/downloads/20200002567.pdf?attachment=true|Continuum/rarefied comparison with calibrated transport|Distinguish transport matching from continuum validity
bergercolella|Local adaptive mesh refinement for shock hydrodynamics|Marsha J. Berger and Phillip Colella|https://doi.org/10.1016/0021-9991(89)90035-1|Conservative mesh refinement and synchronization|Paired integrated flux and reflux falsification controls
dupuischopard|Theory and applications of an alternative lattice Boltzmann grid refinement algorithm|Alexandre Dupuis and Bastien Chopard|https://journals.aps.org/pre/abstract/10.1103/PhysRevE.67.066707|Alternative lattice Boltzmann refinement algorithm|Obtain full text before attributing a stress-transfer implementation
clawpack|Clawpack Riemann book: Approximate solvers|Clawpack Riemann book contributors|https://www.clawpack.org/riemann_book/html/Approximate_solvers.html|Approximate Riemann flux methods|Consistency, positivity and acoustic damping measurements
hypre|hypre|Lawrence Livermore National Laboratory / hypre team|https://hypre.readthedocs.io/en/latest/|Parallel multigrid and sparse linear solvers|Residual tolerance, preconditioning and whole-transaction costs
ibamr|IBAMR|IBAMR developers|https://ibamr.github.io/|Adaptive Cartesian Eulerian fluid and Lagrangian immersed structures|Adjoint interpolation/spreading, force balance and mesh convergence
librom|libROM|Lawrence Livermore National Laboratory|https://www.librom.net/|Reduced-order modeling, POD and DMD|Held-out trajectories, conservation and certified residuals before speed claims
exafmm|ExaFMM-t|ExaFMM-t project|https://github.com/exafmm/exafmm-t|Kernel-independent fast multipole method|Direct-sum accuracy and crossover cost before long-range adoption
nairnmpm|NairnMPM / NairnFEA|John Nairn / project contributors|https://github.com/nairnj/nairn-mpm-fea|Material-point and finite-element computational mechanics|Particle-grid momentum, contact and deformation convergence
chaste|Chaste|University of Oxford computational biology / Chaste team|https://chaste.github.io/|Cardiac, cell-population and soft-tissue modeling|Mechanics and cell-source balances before morphogenesis claims
physicell|PhysiCell|PhysiCell project|https://physicell.org/|Individual-cell mechanics and microenvironment models|Mass uptake, cell birth/death and microenvironment coupling qualification
morpheus|Morpheus|Technische Universitaet Dresden / Morpheus project|https://morpheus.gitlab.io/|ODE, PDE and Cellular Potts models with dependency scheduling|Separate phenomenological shape rules from biomechanical forces
compucell|CompuCell3D|CompuCell3D project|https://compucell3d.org/|Multicellular modeling platform|Diffusion/mechanics ordering and reproducibility before biological claims
sofa|SOFA|SOFA project (source not inspected)|https://www.sofa-framework.org/|Access timed out; capabilities UNRESOLVED in this record|Reattempt primary access, then articulated-mechanics energy tests
febio|FEBio|FEBio project|https://febio.org/|Nonlinear biomechanical finite elements and coupled mixtures|Constitutive calibration, large deformation and interface work
copasi|COPASI|COPASI team|https://copasi.org/|Deterministic/stochastic biochemical simulation and SBML|Stoichiometric and elemental accounting before metabolism
moose|MOOSE|Idaho National Laboratory / MOOSE team|https://mooseframework.inl.gov/|Coupled multiphysics framework|Inspect phase-field implementation and free-energy budget before deposition
biodynamo|BioDynaMo|BioDynaMo collaboration|https://biodynamo.org/|Individual-based simulation platform|Population accounting, interaction costs and ecological null controls
neuron|NEURON|NEURON project (source not inspected)|https://neuron.yale.edu/neuron/|Access timed out; capabilities UNRESOLVED in this record|Reattempt primary docs, then compartment and synaptic timing controls
nest|NEST Simulator|NEST Initiative / developers|https://nest-simulator.readthedocs.io/en/stable/|Spiking neuronal network simulation|Clock/event equivalence and bounded physical consumer before cognition claims
damask|DAMASK|DAMASK developers|https://damask-multiphysics.org/|Crystal-plasticity multiphysics with staggered coupling|Mechanical, damage and phase energy balances before solidification
'''
FIELDS = ['publication_date','version','doi','source_revision','governing_equations','physical_assumptions','spatial_representation','temporal_integration','conservation_properties','boundary_treatment','approximations','error_estimation','stability_constraints','parallelization','gpu_architecture','memory_representation','scaling_characteristics','known_failure_modes','license','dependencies']
def records():
    out=[]
    for line in ROWS.strip().splitlines():
        key,title,authors,url,features,relevance=line.split('|')
        d=dict.fromkeys(FIELDS)
        d.update(id=key,title=title,responsible=authors,url=url,access='primary project page or documentation landing page; selected text, not full source',documented_features=features,published_claims=None,source_verified_features=[],locally_reproduced=[],proposed_adaptation=relevance,required_qualification=relevance,unresolved='Null fields were not established by the inspected material. No donor executable reproduced locally.',status={'CONSULTED':True,'IMPLEMENTED':False,'VALIDATED':False},affected_local_symbols=[],runtime_dependency_added=False,code_imported=False)
        out.append(d)
    by={d['id']:d for d in out}
    by['cart3d'].update(spatial_representation='Adaptively refined Cartesian mesh',error_estimation='Adjoint-weighted output residuals',parallelization='Shared and distributed processing documented',license='NASA catalog government-purpose access; no source obtained',governing_equations='Compressible inviscid Euler equations documented by flowCart guide')
    by['amrex'].update(version='26.00-dev documentation',source_revision='1dfb2b721f048acea54ee4693fb534d69a0aaa29',license='BSD 3-Clause; LICENSE inspected at revision',access='AmrCore subcycling/reflux documentation and selected FluxRegister.cpp source',source_verified_features=['Reflux dispatch across orientations and explicit volume/scale arguments'],spatial_representation='Nested structured grid levels',temporal_integration='Fine level subcycles then synchronizes',conservation_properties='Coarse correction from area/time integrated flux mismatch',status={'CONSULTED':True,'IMPLEMENTED':'independent flux-register principle only','VALIDATED':'local FV tests, not AMReX reproduction'},affected_local_symbols=['scientific::fv::{begin,finish,accumulate}'])
    by['palabos'].update(publication_date='2021-06-09',version='Summer School slides mention Palabos 2.3',access='Selected full-text grid-refinement slides; page15 rendered and inspected',temporal_integration='Acoustic 2:1 refinement discussed',known_failure_modes='Raw versus transformed nonequilibrium notation must be resolved; current authored interface fails conservation',governing_equations='Slide15 relates transformed and raw nonequilibrium distributions; no claim of identical notation to local BGK f',license='Slides are not a solver-code license; solver license not inspected')
    by['p4est'].update(license='GPLv2 stated on project page; exact source license not inspected',known_failure_modes='Guessed GitHub p4est/p4est API source request returned404')
    by['chombo'].update(version='3.2.9, released2024-04-26, per project page',publication_date='Project page updated2025-08-13',license='Modified BSD stated; source access registration not completed',doi='10.6084/m9.figshare.28599755.v1')
    by['flash'].update(version='4.8 stated on consulted page',license='UNRESOLVED; source access request required')
    by['athena'].update(source_revision='1395e2f543f4368d448cb1a2cf92ba1f910e3416',access='GitHub primary project and selected mesh_refinement.cpp source; Princeton homepage fetch failed',source_verified_features=['ProlongateCellCenteredValues uses coarse/fine cell geometry in3D branch'],license='BSD 3-Clause; LICENSE inspected at revision')
    by['hypers'].update(version='CCMC model2021',publication_date='Registry updated2026-11-13; not publication date',governing_equations='Kinetic macro-ions, inertialess neutralizing electron fluid and Darwin field approximation',temporal_integration='EMAPS event-driven local adaptive scheduling documented',parallelization='Mapped mesh and MPI documented',known_failure_modes='Fast ion/whistler constraints; partitioning costs; omitted inner-magnetosphere physics',published_claims='Omelchenko and Karimabadi, HYPERS: A unidimensional asynchronous framework for multiscale hybrid simulations, JCP231(4),1766-1780(2012); registry metadata only')
    by['swmf'].update(version='CCMC20180525',publication_date='Registry updated2026-11-25; not paper date',governing_equations='BATS-R-US conservative MHD model documented',parallelization='MPI block decomposition documented')
    by['sparta'].update(version='27Aug2026 per official news',license='GPL; LGPL by request according to project page, exact checkout not inspected',spatial_representation='Hierarchical collision cells and triangulated cut/split surfaces',parallelization='MPI spatial decomposition documented')
    by['bergercolella'].update(publication_date='1989-05',doi='10.1016/0021-9991(89)90035-1',access='Publisher metadata and selected LBNL-hosted21-page full text',conservation_properties='Replace coarse interface flux by accumulated fine interface flux',status={'CONSULTED':True,'IMPLEMENTED':'independently authored conservative synchronization','VALIDATED':'local tests only'},affected_local_symbols=['scientific::fv::{begin,finish}'])
    by['dupuischopard'].update(publication_date='2003-06-20',doi='10.1103/PhysRevE.67.066707',access='Publisher abstract/metadata only; direct DOI fetch failed, full paper NOT read')
    by['nasadsmc'].update(access='Selected full-text NASA NTRS report',physical_assumptions='Continuum versus rarefied gas regimes and matched transport coefficients',known_failure_modes='DSMC expensive at low Kn; continuum can fail at high local Kn')
    for key,version in [('warpx','26.11 docs'),('hypre','3.2.1 docs'),('chaste','2026.1'),('copasi','4.48 build311,2026-10-22'),('compucell','4.10.0,2026-10-10'),('enzo','2.6 news dated2019-08-02; not verified latest source')]:by[key]['version']=version
    for key,license_ in [('chaste','BSD stated on primary page'),('morpheus','BSD stated on primary page'),('copasi','Artistic2.0 stated on primary page'),('biodynamo','Apache2.0 stated on primary page')]:by[key]['license']=license_
    for key in ['sofa','neuron']:by[key].update(access='Primary URL timed out; no substantive page inspected',status={'CONSULTED':'FAILED ACCESS','IMPLEMENTED':False,'VALIDATED':False})
    return out

def render():
    DOC.mkdir(exist_ok=True)
    data={'schema':'HABITAT-003/donor-evidence-v1','date':'2026-11-27','status':'PARTIAL scientific catalog; incomplete equation-level donor qualification','notice':'Null means UNRESOLVED, not unsupported by donor. No donor executable reproduced, no code imported, no new runtime dependency. Feature pages do not establish current implementation or FORGE performance.','query_record':{'scope':'Initial batches covered AMR/CFD, astrophysical/plasma/rarefied, then long-horizon biology and materials. Exact early query strings unavailable in compacted continuity; URLs and access scope retained.','additional_executed_queries':['site.sparta.github.io DSMC adaptive grid license','site.nas.nasa.gov software cart3d Euler adaptive']},'records':records(),'additional_unresolved_candidates':[{'title':'Rohde et al. mass-conservative LBM refinement','candidate_doi':'10.1002/fld.1140','status':'primary DOI fetch failed; bibliographic details not independently confirmed'}]}
    (DOC/'SCIENTIFIC_DONOR_PROVENANCE.json').write_text(json.dumps(data,indent=2)+'\n')
    lines=['# Scientific donor atlas','',data['status']+'. '+data['notice'],'','Each record in the JSON contains every requested evidence field; uninspected fields remain null. These are research leads, not completed donor qualifications. Two selected source revisions and their file hashes are in `SCIENTIFIC_SOURCE_IDENTITIES.json`. Full downloaded materials are external raw evidence.','','| Donor | Inspected scope | Proposed independent test |','|---|---|---|']
    for d in data['records']:lines.append(f"| [{d['title']}]({d['url']}) | {d['access']} | {d['required_qualification']} |")
    (DOC/'SCIENTIFIC_DONOR_ATLAS.md').write_text('\n'.join(lines)+'\n')
    lines=['# Donor licenses and dependency boundary','','No donor code was imported and no runtime dependency was added. Mathematical-method consultation is distinct from permission to redistribute code or papers. Downloaded licenses remain with source excerpts in the external archive. Unknown licenses require separate review before any import.','','| Donor | Evidence |','|---|---|']
    for d in data['records']:lines.append(f"| {d['title']} | {d['license'] or 'UNRESOLVED: exact license not inspected'} |")
    (DOC/'SCIENTIFIC_DONOR_LICENSES.md').write_text('\n'.join(lines)+'\n')
if __name__=='__main__':render()
