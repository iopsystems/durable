#!/usr/bin/env bash
# Run TLC over the durable model.
#
# Usage: tla/check.sh [config...]
#
# With no arguments every *.cfg in this directory is checked. Needs a Java
# 11+ runtime; tla2tools.jar is downloaded into tla/.tools on first use.
set -euo pipefail

cd "$(dirname "$0")"

TLA_VERSION="${TLA_VERSION:-v1.7.4}"
JAR="${TLA2TOOLS_JAR:-.tools/tla2tools-$TLA_VERSION.jar}"

if [[ ! -f "$JAR" ]]; then
    mkdir -p "$(dirname "$JAR")"
    curl -fsSL -o "$JAR" \
        "https://github.com/tlaplus/tlaplus/releases/download/$TLA_VERSION/tla2tools.jar"
fi

if [[ $# -eq 0 ]]; then
    set -- *.cfg
fi

for cfg in "$@"; do
    echo "==> $cfg"
    java -XX:+UseParallelGC -cp "$JAR" tlc2.TLC \
        -workers auto -cleanup -config "$cfg" MC.tla \
        || echo "==> $cfg: TLC reported a violation or error (exit $?)"
done
