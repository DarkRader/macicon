#!/usr/bin/env bash
set -e

usage() {
  echo "Usage: $0 [--fix|--unsafe-fix] [-f max_errors] [dirs...]"
  echo "  --fix           Automatically fix linting issues"
  echo "  --unsafe-fix    Apply unsafe fixes as well"
  echo "  -f <num>        Fail if more than <num> issues are found"
  echo "  dirs            Optional list of directories to lint (default: src tests)"
}

dirs=("src" "tests")
ruff_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fix)
      ruff_args+=("--fix")
      shift
      ;;
    --unsafe-fix|--unsafe-fixes|-uf)
      ruff_args+=("--unsafe-fixes")
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    -*)
      echo "Unknown option: $1"
      usage
      exit 1
      ;;
    *)
      dirs=("$@")
      break
      ;;
  esac
done

dirs=${dirs[@]:-"src tests"}

ruff check "${ruff_args[@]}" $dirs
echo "✅ Ruff check passed"
