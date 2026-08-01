#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
set -euo pipefail

# This repository predates CI and carries historical rustfmt debt (14 files in
# radiowave / redline-dispatch / redline-observe at the time this landed). Keep
# PR/push lint useful by enforcing rustfmt on the Rust files changed by this
# branch or push, without turning untouched legacy formatting into a permanent
# red check.
#
# Ported from engines/hipfire/scripts/ci-rustfmt-changed.sh. The one deliberate
# divergence: redline is edition 2024 (see the workspace Cargo.toml), hipfire is
# 2021. rustfmt is invoked directly rather than via `cargo fmt` so it can take
# an explicit file list; skip_children=true stops it from walking into modules
# that this change did not touch.

if [[ -z "${GITHUB_ACTIONS:-}" ]]; then
  # Local invocation: check the working tree's own uncommitted changes.
  mapfile -t files < <(git diff --name-only --diff-filter=ACMRT -- '*.rs' | sort)
elif [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]; then
  base_ref="${GITHUB_BASE_REF:?GITHUB_BASE_REF is required for pull_request events}"
  git fetch --no-tags origin "${base_ref}:refs/remotes/origin/${base_ref}"
  range="origin/${base_ref}...HEAD"
  mapfile -t files < <(git diff --name-only --diff-filter=ACMRT "${range}" -- '*.rs' | sort)
elif [[ -n "${GITHUB_EVENT_BEFORE:-}" && "${GITHUB_EVENT_BEFORE}" != "0000000000000000000000000000000000000000" ]]; then
  range="${GITHUB_EVENT_BEFORE}...HEAD"
  mapfile -t files < <(git diff --name-only --diff-filter=ACMRT "${range}" -- '*.rs' | sort)
else
  base_ref="${BASE_REF:-origin/master}"
  git fetch --no-tags origin "master:refs/remotes/origin/master"
  range="${base_ref}...HEAD"
  mapfile -t files < <(git diff --name-only --diff-filter=ACMRT "${range}" -- '*.rs' | sort)
fi

if [[ "${#files[@]}" -eq 0 ]]; then
  echo "No changed Rust files to rustfmt-check."
  exit 0
fi

# Deleted/renamed-away paths survive in the diff list but not on disk.
existing=()
for f in "${files[@]}"; do
  [[ -f "$f" ]] && existing+=("$f")
done

if [[ "${#existing[@]}" -eq 0 ]]; then
  echo "No changed Rust files present on disk to rustfmt-check."
  exit 0
fi

printf 'rustfmt-checking %d changed Rust files:\n' "${#existing[@]}"
printf '  %s\n' "${existing[@]}"
rustfmt --edition 2024 --check --config skip_children=true "${existing[@]}"
