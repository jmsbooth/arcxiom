"""Standards-neutral vector PDF schematic prototype, not technical CAD compliance."""

from __future__ import annotations

from pathlib import Path
import math
from reportlab.lib.pagesizes import A3, landscape
from reportlab.pdfgen.canvas import Canvas

from .model import SourceError

MAX_DRAWING_NODES = 20


def _text(value: str) -> str:
    # The draft renderer intentionally refuses fonts it cannot reliably represent.
    if not value.isascii() or any(ord(ch) < 32 for ch in value):
        raise SourceError("AXP003", "prototype PDF drawing supports ASCII-only labels")
    return value


def render_pdf(ir: dict, path: str | Path, view: str = "overview") -> Path:
    if view != "overview":
        raise SourceError("AXP001", "only the overview viewpoint is implemented")
    nodes = ir["entities"]
    if len(nodes) > MAX_DRAWING_NODES:
        raise SourceError("AXP002", f"schematic supports at most {MAX_DRAWING_NODES} nodes")
    for node in nodes:
        _text(node["id"])
        _text(node["name"])
        _text(node["type"])
    destination = Path(path)
    if destination.is_symlink():
        raise SourceError("AXP004", "cannot write a symlink destination")
    if not destination.parent.is_dir():
        raise SourceError("AXP005", "output directory does not exist")
    w, h = landscape(A3)
    canvas = Canvas(str(destination), pagesize=(w, h), pageCompression=0, invariant=1)
    canvas.setTitle(f"Arcxiom | {ir['namespace']} | schematic")
    canvas.setAuthor("Arcxiom reference prototype")
    canvas.setSubject("Standards-neutral architecture schematic; not to scale")
    canvas.setLineWidth(0.8)
    canvas.rect(32, 32, w - 64, h - 64)
    canvas.setFont("Helvetica-Bold", 19)
    canvas.drawString(56, h - 72, f"ARCXIOM  /  {ir['namespace'][:56].upper()}")
    canvas.setFont("Helvetica", 10)
    canvas.drawString(56, h - 94, "DECLARATIVE ARCHITECTURE VIEW  |  OVERVIEW")
    canvas.drawRightString(w - 56, h - 70, "SCHEMATIC  /  NOT TO SCALE")
    canvas.line(40, h - 109, w - 40, h - 109)

    left, top = 62, h - 240
    col_step, row_step = 214, 142
    width, height = 180, 73
    positions = {}
    for idx, node in enumerate(nodes):
        col, row = idx % 5, idx // 5
        x, y = left + col * col_step, top - row * row_step
        positions[node["id"]] = (x, y)
    canvas.setStrokeColorRGB(0.48, 0.5, 0.52)
    for edge in ir["relationships"]:
        a, b = positions[edge["from"]], positions[edge["to"]]
        ax, ay = a[0] + width / 2, a[1] + height / 2
        bx, by = b[0] + width / 2, b[1] + height / 2
        dx, dy = bx - ax, by - ay
        distance = math.hypot(dx, dy)
        if distance < 1:
            continue
        unit_x, unit_y = dx / distance, dy / distance
        # Trim the directed edge to the actual node-box boundaries.
        candidates = [((width / 2) + 3) / abs(unit_x) if unit_x else float("inf"),
                      ((height / 2) + 3) / abs(unit_y) if unit_y else float("inf")]
        trim = min(candidates)
        sx, sy = ax + unit_x * trim, ay + unit_y * trim
        ex, ey = bx - unit_x * trim, by - unit_y * trim
        canvas.line(sx, sy, ex, ey)
        # Arrowhead communicates direction independently of label placement.
        head = 7
        wing = 3.4
        path = canvas.beginPath()
        path.moveTo(ex, ey)
        path.lineTo(ex - unit_x * head - unit_y * wing,
                    ey - unit_y * head + unit_x * wing)
        path.lineTo(ex - unit_x * head + unit_y * wing,
                    ey - unit_y * head - unit_x * wing)
        path.close()
        canvas.setFillColorRGB(0.38, 0.40, 0.42)
        canvas.drawPath(path, stroke=0, fill=1)
        label = _text(edge["relation"])
        canvas.setFont("Helvetica", 6.6)
        label_width = canvas.stringWidth(label, "Helvetica", 6.6)
        offset_x, offset_y = -unit_y * 10, unit_x * 10
        cx = (sx + ex) / 2 + offset_x
        cy = (sy + ey) / 2 + offset_y
        canvas.setFillColorRGB(1, 1, 1)
        canvas.roundRect(cx - label_width / 2 - 3, cy - 3,
                         label_width + 6, 12, 2, stroke=0, fill=1)
        canvas.setFillColorRGB(0.21, 0.23, 0.26)
        canvas.drawCentredString(cx, cy, label)
    canvas.setStrokeColorRGB(0.14, 0.16, 0.18)
    for node in nodes:
        x, y = positions[node["id"]]
        canvas.setFillColorRGB(0.97, 0.98, 0.99)
        canvas.roundRect(x, y, width, height, 6, stroke=1, fill=1)
        canvas.setFillColorRGB(0.07, 0.09, 0.12)
        canvas.setFont("Helvetica-Bold", 9)
        canvas.drawString(x + 9, y + 52, _text(node["type"][:25]).upper())
        canvas.setFont("Helvetica", 10)
        canvas.drawString(x + 9, y + 35, _text(node["name"][:29]))
        canvas.setFont("Helvetica", 7.5)
        canvas.drawString(x + 9, y + 16, _text(node["id"][:34]))
    canvas.setFillColorRGB(0, 0, 0)
    canvas.line(40, 120, w - 40, 120)
    canvas.setFont("Helvetica-Bold", 11)
    canvas.drawString(56, 97, "ARCXIOM  |  DRAWING REGISTER")
    canvas.setFont("Helvetica", 9)
    canvas.drawString(56, 77, f"NAMESPACE: {ir['namespace']}  |  ENTITIES: {len(nodes)}  |  RELATIONSHIPS: {len(ir['relationships'])}")
    canvas.drawString(56, 57, "ISSUE: PROTOTYPE  |  PROFILE: GENERIC  |  NO ISO / NCS COMPLIANCE CLAIM")
    canvas.drawRightString(w - 56, 57, "SHEET 1 / 1")
    canvas.save()
    return destination
