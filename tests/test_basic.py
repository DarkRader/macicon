"""Basic unit tests for macicon."""

from macicon.fetcher import extract_path_from_svg
from macicon.renderer import build_letter_svg
from macicon.themes import THEME_PRESETS, compute_styling, is_color_dark


def test_theme_presets():
    assert "light" in THEME_PRESETS
    assert "dark" in THEME_PRESETS
    assert "nord" in THEME_PRESETS
    assert "apple" in THEME_PRESETS

def test_color_dark():
    assert is_color_dark("#000000") is True
    assert is_color_dark("#FFFFFF") is False
    assert is_color_dark("#161618") is True
    assert is_color_dark("#EBECEF") is False

def test_compute_styling():
    bg_top, bg_bottom, border, symbol = compute_styling("light", bg="white", color="black")
    assert bg_top == "#FFFFFF"
    assert bg_bottom == "#EBECEF"
    assert border == "#D8D9DC"
    assert symbol == "#202022"

def test_extract_path():
    svg = '<svg viewBox="0 0 48 48"><path d="M10 10 L20 20"/></svg>'
    path_d, vb, fr = extract_path_from_svg(svg)
    assert path_d == "M10 10 L20 20"
    assert vb == "0 0 48 48"
    assert fr == "evenodd"

def test_build_letter_svg():
    svg = build_letter_svg("A", "#FFF", "#EEE", "#DDD", "#000", 1.25)
    assert "<text" in svg
    assert "A</text>" in svg
