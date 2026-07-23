# Redline Developer Onboarding Design

## Goal

Make the current source checkout directly usable by C/C++, Python, hipGraph, and Rust developers through one accurate choose-your-path guide. State the future binary distribution contract without claiming that unpublished packages exist.

## Scope

- Rewrite `docs/INTEGRATION.md` as the canonical **Using Redline** guide.
- Add a concise integration decision table and canonical-guide link to the top-level README.
- Correct stale package copy that presents Redline primarily as a HipGraph replacement or says the C/Python paths are mock-only.
- Verify every current-source command presented as runnable.
- Document the intended PyPI and GitHub Release artifacts separately from current installation instructions.

## Non-goals

- Publishing to PyPI in this change.
- Creating a GitHub Release or uploading C SDK binaries in this change.
- Adding apt/deb packaging.
- Publishing the Rust crates to crates.io before path-only dependencies have versioned package metadata.
- Expanding API or runtime behavior.

## Information Architecture

`docs/INTEGRATION.md` begins with a decision table for four supported source integrations:

1. **C/C++ engine API** — explicit stable ABI for applications that own graph construction and module loading.
2. **Python** — locally built abi3 wheel exposing the Python graph and GPU APIs.
3. **hipGraph preload** — compatibility acceleration for supported existing `hipGraph*` applications, with unsupported operations falling through to HIP.
4. **Rust** — direct source dependency and the native graph API.

The guide then provides:

- shared ROCm, architecture, and toolchain prerequisites;
- the common HSACO, kernel-symbol, kernarg-layout, resource-access, and retained-lifecycle contract;
- a complete build, minimal usage, smoke-test, expected-result, and limitation section for each integration;
- error and fail-closed behavior;
- current distribution status and the intended release contract.

Crate READMEs remain detailed references. The top-level README remains the product landing page and links to the canonical guide rather than duplicating full walkthroughs.

## Distribution Contract

### Current

- C developers build `redline-capi` from source and receive `libredline_dispatch.so`, `libredline_dispatch.a`, and `redline_dispatch.h`.
- Python developers build and install an abi3 Python 3.9+ wheel with maturin.
- hipGraph users build the preload interposer from source.
- Rust developers use the workspace/source repository.

The guide must not show `pip install redline-dispatch` or a release download URL as currently available.

### Intended release

- **Python:** publish the `redline-dispatch` abi3 wheel to PyPI through the existing GitHub Release workflow using PyPI trusted publishing or a scoped token.
- **C/C++:** attach a versioned SDK archive to each GitHub Release containing the public header, shared library, static library, LICENSE/NOTICE material, and SHA-256 checksums.
- **Deferred:** apt/deb and crates.io packaging until demand and dependency metadata justify them.

## Verification

Before completion:

- Build `redline-capi` in release mode.
- Compile and run the C ABI smoke example against the emitted shared library.
- Build the Python wheel with maturin, install it into an isolated environment, and import the public `Graph` and `Gpu` APIs.
- Run the Rust `hipgraph_migration` example.
- Build the hipGraph preload crate and verify the documented artifact path.
- Check that all documented repository-relative paths exist.
- Run GitNexus change detection before committing documentation changes.

## Acceptance Criteria

- A developer can choose an integration without understanding Redline internals.
- Every current installation and smoke command in the guide has been executed successfully in this checkout.
- The guide explains the shared kernel and replay contract instead of presenting only build commands.
- Current and future distribution states are unambiguous.
- No stale claim remains that the C or Python path is mock-only.
- Redline is positioned as lightning-fast ROCm dispatch; hipGraph is one compatibility surface, not the product identity.
