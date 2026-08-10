#!/usr/bin/env bash
# Scan everything a commit would include for secret-shaped literals.
#
#   ./scripts/check-secrets.sh
#
# Exits 0 when clean, 1 when something needs a look — so it also works as a
# pre-commit hook:
#
#   ln -s ../../scripts/check-secrets.sh .git/hooks/pre-commit
#
# This is a safety net, not a guarantee. It catches the common shapes: a literal
# assigned to a secret-ish name, and long opaque strings that look like keys.
# It cannot catch a secret that looks like ordinary text.

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

NAMED='(token|secret|passwd|password|api[_-]?key|auth)[a-z_]*[[:space:]]*[:=][^=]*"[^"]{12,}"'
OPAQUE='"[A-Za-z0-9+/_=-]{40,}"'

# Lines that merely *reference* a secret by name are correct and expected —
# ${{ secrets.FOO }} in a workflow, std::env::var("FOO") in source.
REFERENCE='(secrets\.|env::var|getenv|process\.env|\$\{\{)'

found=0

while IFS= read -r f; do
  [ -f "$f" ] || continue
  case "$f" in
    Cargo.lock | *.lock | *.png | *.jpg | *.jpeg | *.gif | *.pdf | *.woff*) continue ;;
    scripts/check-secrets.sh) continue ;; # this file contains the patterns themselves
  esac

  if hits=$(grep -nEi "$NAMED" "$f" 2>/dev/null | grep -vE "$REFERENCE"); then
    [ -n "$hits" ] && {
      printf '%s\n' "$hits" | sed -E 's/"[^"]{12,}"/"<REDACTED>"/g' | sed "s|^|  secret-shaped assignment  $f:|"
      found=1
    }
  fi

  if n=$(grep -cE "$OPAQUE" "$f" 2>/dev/null) && [ "${n:-0}" -gt 0 ]; then
    printf '  long opaque literal  %s: %s occurrence(s) — confirm these are not keys\n' "$f" "$n"
    found=1
  fi
done < <(git ls-files --cached --others --exclude-standard)

if [ "$found" -eq 0 ]; then
  echo "✅ clean — nothing secret-shaped would be committed"
  exit 0
fi

cat <<'MSG'

❌ Review the lines above before committing.

If one is a real credential: move the value into .env (already gitignored) and
read it at runtime with std::env::var, leaving only the loading code in source.
A committed secret stays in git history even if a later commit removes the line.

If one is a false positive — a hash, a test fixture, a base64 asset — it is safe
to commit. Re-run after fixing, or `git commit --no-verify` to bypass as a hook.
MSG
exit 1
