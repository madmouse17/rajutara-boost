#!/usr/bin/env python3
"""Checks every line of every ```rust sig fence in guidelines/*.md against an optirust checkout.

Usage: scripts/verify-signatures.py [path/to/optirust]   (default: ../optirust)

A line passes when it occurs, whitespace-normalised, in the checkout's crates/**/*.rs;\na `pub fn` line must also end exactly where the source signature ends.
"""
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "../optirust")
here = pathlib.Path(__file__).resolve().parent.parent


def norm(text: str) -> str:
    text = re.sub(r"\s+", " ", text)
    text = re.sub(r"\( ", "(", text)
    text = re.sub(r",? \)", ")", text)
    text = re.sub(r"< ", "<", text)
    return re.sub(r",? >", ">", text).strip()


source = norm(" ".join(p.read_text() for p in (root / "crates").rglob("*.rs")))
checked = failed = 0
for md in sorted((here / "guidelines").glob("*.md")):
    in_sig = False
    for n, line in enumerate(md.read_text().splitlines(), 1):
        if line.startswith("```"):
            in_sig = line.strip() == "```rust sig" and not in_sig
            continue
        if in_sig and line.strip():
            checked += 1
            want = norm(line.strip().rstrip(";"))
            # A fn signature must end where the source's does: at the body or at a trait item's `;`.
            found = any(want + end in source for end in (" {", ";")) if want.startswith("pub fn") else want in source
            if not found:
                failed += 1
                print(f"MISSING {md.name}:{n}: {line.strip()}")
print(f"{checked} signature lines checked, {failed} missing")
sys.exit(1 if failed else 0)
