"""Icon searching, SVG downloading, and path extraction for macicon."""

import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

from constants import COMMON_ALIASES, KNOWN_ICONS


def extract_path_from_svg(svg_content: str) -> tuple[str, str, str]:
    """Extract combined <path d='...'>, viewBox, and fill-rule from SVG markup."""
    vb_match = re.search(r'viewBox=["\']([^"\']+)["\']', svg_content)
    if vb_match:
        viewbox = vb_match.group(1)
    else:
        w_match = re.search(r'<svg[^>]*\bwidth=["\']([0-9.]+)["\']', svg_content)
        h_match = re.search(r'<svg[^>]*\bheight=["\']([0-9.]+)["\']', svg_content)
        viewbox = (
            f"0 0 {w_match.group(1)} {h_match.group(1)}" if w_match and h_match else "0 0 24 24"
        )

    fr_match = re.search(r'\bfill-rule=["\']([^"\']+)["\']', svg_content)
    fill_rule = fr_match.group(1) if fr_match else "evenodd"

    paths = re.findall(r'<path[^>]*\bd=["\']([^"\']+)["\']', svg_content)
    if not paths:
        sys.exit('⚠️  Error: No <path d="..."> found in the SVG.')
    return " ".join(paths), viewbox, fill_rule


def _fetch_from_url(query_str: str) -> tuple[dict[str, str] | None, str]:
    """Parse direct SVG URL or query parameter, downloading if direct SVG."""
    parsed = urllib.parse.urlparse(query_str)
    qs = urllib.parse.parse_qs(parsed.query)

    if qs.get("q"):
        return None, qs["q"][0]
    if (
        parsed.path.endswith(".svg")
        or "cdn.simpleicons.org" in parsed.netloc
        or "jsdelivr.net" in parsed.netloc
    ):
        try:
            req = urllib.request.Request(query_str, headers={"User-Agent": "macicon-cli"})  # noqa: S310
            with urllib.request.urlopen(req, timeout=8) as resp:  # noqa: S310
                print(f"Fetched SVG from URL: {query_str}")
                content = resp.read().decode("utf-8")
                path_d, viewbox, fill_rule = extract_path_from_svg(content)
                stem = Path(parsed.path).stem or "custom"
                return {
                    "type": "path",
                    "path_d": path_d,
                    "viewbox": viewbox,
                    "fill_rule": fill_rule,
                    "name": stem,
                }, query_str
        except (urllib.error.URLError, TimeoutError, OSError, ValueError) as e:
            sys.exit(f"⚠️  Error fetching SVG from URL '{query_str}': {e}")

    path_parts = [p for p in parsed.path.split("/") if p and p != "icons"]
    resolved_query = path_parts[-1].replace(".svg", "") if path_parts else query_str
    return None, resolved_query


def _search_online_icons(candidate_slugs: list[str], slug: str) -> dict[str, str] | None:
    """Search online CDN repositories for matching SVG vector icon."""
    for cand in candidate_slugs:
        sources = [
            ("Simple Icons", f"https://cdn.jsdelivr.net/npm/simple-icons@latest/icons/{cand}.svg"),
            (
                "Dashboard Icons",
                f"https://raw.githubusercontent.com/homarr-labs/dashboard-icons/main/svg/{cand}.svg",
            ),
        ]
        for source_name, url in sources:
            try:
                req = urllib.request.Request(url, headers={"User-Agent": "macicon-cli"})  # noqa: S310
                with urllib.request.urlopen(req, timeout=6) as resp:  # noqa: S310
                    print(f"Found icon on {source_name}: {url}")
                    content = resp.read().decode("utf-8")
                    path_d, viewbox, fill_rule = extract_path_from_svg(content)
                    return {
                        "type": "path",
                        "path_d": path_d,
                        "viewbox": viewbox,
                        "fill_rule": fill_rule,
                        "name": slug,
                    }
            except (urllib.error.HTTPError, urllib.error.URLError):
                continue
    return None


def fetch_icon_or_create(query: str, *, fallback_letter: bool = False) -> dict[str, str]:
    """Resolve icon from direct URL, built-in vector, Simple Icons, or lettermark."""
    query_str = query.strip()

    # Direct URL or simpleicons query link
    if query_str.startswith(("http://", "https://")):
        direct_icon, query_str = _fetch_from_url(query_str)
        if direct_icon is not None:
            return direct_icon

    slug = query_str.lower().strip().replace(" ", "").replace("-", "").replace("_", "")
    slug_hyphen = query_str.lower().strip().replace(" ", "-").replace("_", "-")

    if slug in COMMON_ALIASES and COMMON_ALIASES[slug] in KNOWN_ICONS:
        slug = COMMON_ALIASES[slug]
    elif slug_hyphen in COMMON_ALIASES and COMMON_ALIASES[slug_hyphen] in KNOWN_ICONS:
        slug = COMMON_ALIASES[slug_hyphen]

    if slug in KNOWN_ICONS:
        info = dict(KNOWN_ICONS[slug])
        info["type"] = "path"
        return info

    candidate_slugs = [slug]
    if slug_hyphen not in candidate_slugs:
        candidate_slugs.append(slug_hyphen)
    if slug in COMMON_ALIASES and COMMON_ALIASES[slug] not in candidate_slugs:
        candidate_slugs.append(COMMON_ALIASES[slug])
    if slug_hyphen in COMMON_ALIASES and COMMON_ALIASES[slug_hyphen] not in candidate_slugs:
        candidate_slugs.append(COMMON_ALIASES[slug_hyphen])
    for s in [f"{slug}industries", f"{slug}editor", f"{slug}app", f"{slug_hyphen}-app"]:
        if s not in candidate_slugs:
            candidate_slugs.append(s)

    online_icon = _search_online_icons(candidate_slugs, slug)
    if online_icon is not None:
        return online_icon

    if fallback_letter:
        short_monogram_max_chars = 2
        letter = (
            query_str[:short_monogram_max_chars].upper()
            if len(query_str) <= short_monogram_max_chars
            else query_str[0].upper()
        )
        print(
            f"Icon '{query_str}' not found online. Generating Apple-style lettermark '{letter}'..."
        )
        return {"type": "letter", "letter": letter, "name": f"letter-{letter.lower()}"}

    sys.exit(
        f"⚠️  Error: Could not find icon '{query_str}' on Simple Icons or Dashboard Icons.\n\n"
        f"Recommended solutions:\n"
        f"  1. Check https://simpleicons.org for the exact slug (e.g. 'googlegemini' instead of 'gemini').\n"
        f"  2. Provide the direct simpleicons link (e.g. https://simpleicons.org/?q={query_str}).\n"
        f"  3. Use --fallback-letter to auto-create a clean Apple lettermark icon.\n"
        f"  4. Provide a local vector file with --svg <path>."
    )
