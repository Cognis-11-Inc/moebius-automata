# Conservative multiscale contract and derivation

For each leaf, U=(M,Px,Py,Pz,S0,...,S3) stores SI integrated inventories, not
samples duplicated at parent and child. Geometry gives V=h³, rho=M/V, u=P/M,
Cj=Sj/V. The authored continuum approximation is d(rho)/dt+div(rho*u)=0,
d(rho*u)/dt+div(rho*u*u+pI)=mu*Laplacian(u)+rho*g, p=cs²rho;
d(Cj)/dt+div(Cj*(u+slipj))=Dj*Laplacian(Cj). mu is constant nominal dynamic
viscosity; this is a vector-Laplacian closure, not the complete compressible stress.

Each oriented face is evaluated once. Let Q=dt*A*F. Left receives -Q, right +Q;
therefore the pair cancels without normalization. Rusanov uses lambda=max(|un|)+cs
for mass and normal momentum. Tangential momentum is transported by that same
mass flux using upstream tangential velocity, with -mu*delta(u)/distance added.
Species use upstream transport and centered Fick diffusion. No acoustic damping
is imposed on species. Sealed FV walls mirror normal velocity, retain tangential
velocity, and account for pressure/viscous reaction as external wall impulse.
Outward terminal species slip enters a six-wall reservoir; no material is deleted.

Refinement divides extensive inventories by8; coarsening sums8 children. This
conserves inventories but does not reconstruct gradients. Global accounted
momentum includes prescribed gravity/wall impulses; bodies are refused in003.
The shared001 species description is reused; the Stokes slip law remains an
assumption, not a resolved particle flow. See protocol for positivity/CFL/budgets.

Local tests exercise face cancellation, remapping, synchronization, explicit
refusal and checkpoint continuation. A test-enforced inventory guard and the
reflux correction receive honest-red mutations, with exact command/results in
HABITAT-003_VALIDATION.json. No threshold may be loosened to repair an experiment.
