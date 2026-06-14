#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PATH_ARG="${1:-optics/fodo.mad}"
TURNS="${UFO_BENCH_TURNS:-100}"
PARTICLES="${UFO_BENCH_PARTICLES:-1024}"
RUNS="${UFO_BENCH_RUNS:-5}"
WARMUPS="${UFO_BENCH_WARMUPS:-1}"
PY_REF="${UFO_PY_REF:-}"
PYTHON="${UFO_PYTHON:-python3}"

cd "$ROOT"

if [[ -z "$PY_REF" ]]; then
  for candidate in HEAD~1 HEAD origin/interpreter; do
    if git cat-file -e "${candidate}:ufo/__init__.py" 2>/dev/null; then
      PY_REF="$candidate"
      break
    fi
  done
fi

if [[ -z "$PY_REF" ]]; then
  echo "error: could not find a git ref containing the legacy Python package" >&2
  echo "set UFO_PY_REF=<ref> to choose one explicitly" >&2
  exit 2
fi

TMP="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

git archive "$PY_REF" ufo optics | tar -x -C "$TMP"

echo "# legacy_python_ref=$PY_REF"
echo "# path=$PATH_ARG turns=$TURNS particles=$PARTICLES runs=$RUNS warmups=$WARMUPS"

cargo build --release --example bench_track >/dev/null
if ! "$ROOT/target/release/examples/bench_track" \
  --path "$PATH_ARG" \
  --turns "$TURNS" \
  --particles "$PARTICLES" \
  --runs "$RUNS" \
  --warmups "$WARMUPS" \
  --flag linear \
  --flag achromatic \
  --collapse-linear; then
  echo "rust,error,0,0,$PARTICLES,$TURNS,0"
  echo "# rust_error=bench_track failed; check OpenCL build/runtime output above" >&2
fi

cat >"$TMP/bench_legacy_python.py" <<'PY'
import os
import sys
import time

root = os.environ["UFO_PY_ROOT"]
path = os.environ["UFO_BENCH_PATH"]
turns = int(os.environ["UFO_BENCH_TURNS"])
particles = int(os.environ["UFO_BENCH_PARTICLES"])
runs = int(os.environ["UFO_BENCH_RUNS"])
warmups = int(os.environ["UFO_BENCH_WARMUPS"])

sys.path.insert(0, root)

try:
    import numpy as np
    import pyopencl  # noqa: F401
    import ufo
except Exception as error:
    print(f"python,dependency_error,0,0,{particles},{turns},0")
    print(f"# python_error={type(error).__name__}: {error}", file=sys.stderr)
    raise SystemExit(0)

print("implementation,phase,run,ms,particles,turns,samples")

def once(kind, run):
    start = time.perf_counter()
    lattice = ufo.Lattice(path=path)
    line = getattr(lattice, "RING", next(iter(lattice.__dict__.values())))
    track = ufo.Track(
        line,
        turns=turns,
        particles=particles,
        where=[-1],
        flags=ufo.LINEAR | ufo.ACHROMATIC,
        parameters=["x"],
    )
    track.parameters[:, 0] = np.float32(0.001)
    build_ms = (time.perf_counter() - start) * 1.0e3
    print(f"python,build_{kind},{run},{build_ms:.6f},{particles},{turns},{track.count}")

    start = time.perf_counter()
    track.run(threads=min(particles, 256))
    run_ms = (time.perf_counter() - start) * 1.0e3
    samples = track.tracks.shape[0] * track.tracks.shape[1]
    print(f"python,track_{kind},{run},{run_ms:.6f},{particles},{turns},{samples}")

for kind, count in (("warmup", warmups), ("measure", runs)):
    for run in range(count):
        try:
            once(kind, run)
        except Exception as error:
            print(f"python,error_{kind},{run},0,{particles},{turns},0")
            print(f"# python_error={type(error).__name__}: {error}", file=sys.stderr)
            break
PY

UFO_PY_ROOT="$TMP" \
UFO_BENCH_PATH="$ROOT/$PATH_ARG" \
UFO_BENCH_TURNS="$TURNS" \
UFO_BENCH_PARTICLES="$PARTICLES" \
UFO_BENCH_RUNS="$RUNS" \
UFO_BENCH_WARMUPS="$WARMUPS" \
"$PYTHON" "$TMP/bench_legacy_python.py"
