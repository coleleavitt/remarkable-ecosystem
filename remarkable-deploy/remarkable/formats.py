"""
reMarkable .rm file format parsing.

Supports v3, v5, and v6 formats with automatic version detection.
Provides stroke/point extraction and SVG export.
"""

from __future__ import annotations

import io
import logging
import struct
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import BinaryIO

logger = logging.getLogger(__name__)


class RmVersion(IntEnum):
    """reMarkable file format versions."""
    V3 = 3
    V5 = 5
    V6 = 6


class PenType(IntEnum):
    """Pen tool types."""
    BALLPOINT_1 = 2
    BALLPOINT_2 = 15
    FINELINER_1 = 4
    FINELINER_2 = 17
    MARKER_1 = 3
    MARKER_2 = 16
    PENCIL_1 = 1
    PENCIL_2 = 14
    MECHANICAL_PENCIL_1 = 7
    MECHANICAL_PENCIL_2 = 13
    PAINTBRUSH_1 = 5
    PAINTBRUSH_2 = 12
    HIGHLIGHTER_1 = 8
    HIGHLIGHTER_2 = 18
    ERASER = 6
    ERASE_AREA = 9
    CALLIGRAPHY = 21
    SELECTION = 10
    UNKNOWN = 0


class PenColor(IntEnum):
    """Pen colors."""
    BLACK = 0
    GRAY = 1
    WHITE = 2
    YELLOW = 3
    GREEN = 4
    PINK = 5
    BLUE = 6
    RED = 7
    GRAY_OVERLAP = 8
    GREEN_2 = 9
    CYAN = 10
    MAGENTA = 11
    YELLOW_2 = 12


# Color hex values for SVG export
COLOR_MAP = {
    PenColor.BLACK: "#000000",
    PenColor.GRAY: "#9d9d9d",
    PenColor.WHITE: "#ffffff",
    PenColor.YELLOW: "#fbf785",
    PenColor.GREEN: "#00ff00",
    PenColor.PINK: "#ff8080",
    PenColor.BLUE: "#0066cc",
    PenColor.RED: "#ff0000",
    PenColor.GRAY_OVERLAP: "#9d9d9d",
    PenColor.GREEN_2: "#00a000",
    PenColor.CYAN: "#00ffff",
    PenColor.MAGENTA: "#ff00ff",
    PenColor.YELLOW_2: "#ffff00",
}


@dataclass
class Point:
    """A single point in a stroke."""
    x: float
    y: float
    pressure: float = 0.0
    tilt_x: float = 0.0
    tilt_y: float = 0.0
    speed: float = 0.0


@dataclass
class Stroke:
    """A single stroke (pen line)."""
    pen_type: PenType
    color: PenColor
    points: list[Point] = field(default_factory=list)
    base_width: float = 2.0
    
    @property
    def point_count(self) -> int:
        return len(self.points)
    
    @property
    def is_eraser(self) -> bool:
        return self.pen_type in (PenType.ERASER, PenType.ERASE_AREA)
    
    @property
    def is_highlighter(self) -> bool:
        return self.pen_type in (PenType.HIGHLIGHTER_1, PenType.HIGHLIGHTER_2)


@dataclass
class Layer:
    """A layer containing strokes."""
    strokes: list[Stroke] = field(default_factory=list)
    name: str = ""
    visible: bool = True
    
    @property
    def stroke_count(self) -> int:
        return len(self.strokes)
    
    @property
    def point_count(self) -> int:
        return sum(s.point_count for s in self.strokes)


@dataclass
class RmDocument:
    """Parsed .rm document."""
    version: RmVersion
    layers: list[Layer] = field(default_factory=list)
    
    @property
    def layer_count(self) -> int:
        return len(self.layers)
    
    @property
    def stroke_count(self) -> int:
        return sum(l.stroke_count for l in self.layers)
    
    @property
    def point_count(self) -> int:
        return sum(l.point_count for l in self.layers)
    
    def to_svg(
        self,
        width: int = 1404,
        height: int = 1872,
        *,
        include_template: bool = False,
        stroke_scale: float = 1.0,
    ) -> str:
        """
        Convert to SVG string.
        
        Args:
            width: SVG width in pixels
            height: SVG height in pixels
            include_template: Include template background
            stroke_scale: Scale factor for stroke widths
            
        Returns:
            SVG document as string
        """
        lines = [
            f'<?xml version="1.0" encoding="UTF-8"?>',
            f'<svg xmlns="http://www.w3.org/2000/svg" '
            f'width="{width}" height="{height}" '
            f'viewBox="0 0 {width} {height}">',
            f'<rect width="100%" height="100%" fill="white"/>',
        ]
        
        for layer_idx, layer in enumerate(self.layers):
            if not layer.visible:
                continue
            
            lines.append(f'<g id="layer-{layer_idx}">')
            
            for stroke in layer.strokes:
                if stroke.is_eraser:
                    continue  # Skip eraser strokes in SVG
                
                if not stroke.points:
                    continue
                
                color = COLOR_MAP.get(stroke.color, "#000000")
                width = stroke.base_width * stroke_scale
                opacity = 0.5 if stroke.is_highlighter else 1.0
                
                # Build path
                path_data = []
                for i, point in enumerate(stroke.points):
                    cmd = "M" if i == 0 else "L"
                    path_data.append(f"{cmd}{point.x:.2f},{point.y:.2f}")
                
                lines.append(
                    f'<path d="{" ".join(path_data)}" '
                    f'fill="none" stroke="{color}" '
                    f'stroke-width="{width:.2f}" '
                    f'stroke-linecap="round" '
                    f'stroke-linejoin="round" '
                    f'opacity="{opacity}"/>'
                )
            
            lines.append('</g>')
        
        lines.append('</svg>')
        return "\n".join(lines)


class RmParser:
    """
    Parser for reMarkable .rm files.
    
    Supports v3, v5, and v6 formats with automatic detection.
    """
    
    # Format headers
    HEADER_V3 = b"reMarkable .lines file, version=3          "
    HEADER_V5 = b"reMarkable .lines file, version=5          "
    HEADER_V6 = b"reMarkable .lines file, version=6          "
    
    def parse_file(self, path: Path | str) -> RmDocument:
        """
        Parse a .rm file from disk.
        
        Args:
            path: Path to .rm file
            
        Returns:
            Parsed RmDocument
        """
        path = Path(path)
        with open(path, "rb") as f:
            return self.parse(f)
    
    def parse_bytes(self, data: bytes) -> RmDocument:
        """
        Parse .rm data from bytes.
        
        Args:
            data: Raw .rm file bytes
            
        Returns:
            Parsed RmDocument
        """
        return self.parse(io.BytesIO(data))
    
    def parse(self, stream: BinaryIO) -> RmDocument:
        """
        Parse .rm data from a stream.
        
        Auto-detects version from header.
        
        Args:
            stream: Binary stream with .rm data
            
        Returns:
            Parsed RmDocument
        """
        header = stream.read(43)
        
        if header.startswith(b"reMarkable .lines file, version=6"):
            return self._parse_v6(stream)
        elif header.startswith(b"reMarkable .lines file, version=5"):
            return self._parse_v5(stream)
        elif header.startswith(b"reMarkable .lines file, version=3"):
            return self._parse_v3(stream)
        else:
            raise ValueError(f"Unknown .rm format: {header[:40]}")
    
    def _parse_v3(self, stream: BinaryIO) -> RmDocument:
        """Parse v3 format."""
        layers = []
        
        # Read layer count
        layer_count = struct.unpack("<I", stream.read(4))[0]
        
        for _ in range(layer_count):
            layer = self._parse_layer_v3(stream)
            layers.append(layer)
        
        return RmDocument(version=RmVersion.V3, layers=layers)
    
    def _parse_v5(self, stream: BinaryIO) -> RmDocument:
        """Parse v5 format (same structure as v3)."""
        layers = []
        
        layer_count = struct.unpack("<I", stream.read(4))[0]
        
        for _ in range(layer_count):
            layer = self._parse_layer_v5(stream)
            layers.append(layer)
        
        return RmDocument(version=RmVersion.V5, layers=layers)
    
    def _parse_v6(self, stream: BinaryIO) -> RmDocument:
        """
        Parse v6 format with CRDT blocks.
        
        v6 uses a tagged block structure for CRDT operations.
        """
        layers: list[Layer] = []
        current_layer = Layer()
        
        # Skip to content (past any additional header bytes)
        # v6 has tagged blocks with type prefixes
        
        try:
            while True:
                # Read block header
                block_type_data = stream.read(4)
                if len(block_type_data) < 4:
                    break
                
                block_type = struct.unpack("<I", block_type_data)[0]
                
                # Read block length
                block_len_data = stream.read(4)
                if len(block_len_data) < 4:
                    break
                
                block_len = struct.unpack("<I", block_len_data)[0]
                
                # Read block content
                block_data = stream.read(block_len)
                
                # Parse based on block type
                if block_type == 5:  # SceneLineItemBlock
                    stroke = self._parse_stroke_block(block_data)
                    if stroke:
                        current_layer.strokes.append(stroke)
                elif block_type == 1:  # Layer marker
                    if current_layer.strokes:
                        layers.append(current_layer)
                        current_layer = Layer()
                        
        except Exception as e:
            logger.debug(f"Stopped parsing v6: {e}")
        
        # Add final layer
        if current_layer.strokes:
            layers.append(current_layer)
        
        # If no layers found, create empty one
        if not layers:
            layers = [Layer()]
        
        return RmDocument(version=RmVersion.V6, layers=layers)
    
    def _parse_layer_v3(self, stream: BinaryIO) -> Layer:
        """Parse a v3 layer."""
        strokes = []
        
        stroke_count = struct.unpack("<I", stream.read(4))[0]
        
        for _ in range(stroke_count):
            stroke = self._parse_stroke_v3(stream)
            strokes.append(stroke)
        
        return Layer(strokes=strokes)
    
    def _parse_layer_v5(self, stream: BinaryIO) -> Layer:
        """Parse a v5 layer."""
        strokes = []
        
        stroke_count = struct.unpack("<I", stream.read(4))[0]
        
        for _ in range(stroke_count):
            stroke = self._parse_stroke_v5(stream)
            strokes.append(stroke)
        
        return Layer(strokes=strokes)
    
    def _parse_stroke_v3(self, stream: BinaryIO) -> Stroke:
        """Parse a v3 stroke."""
        # Pen type and color
        pen_type_raw = struct.unpack("<I", stream.read(4))[0]
        color_raw = struct.unpack("<I", stream.read(4))[0]
        
        # Base width
        _unknown = struct.unpack("<I", stream.read(4))[0]
        base_width = struct.unpack("<f", stream.read(4))[0]
        
        # Point count
        point_count = struct.unpack("<I", stream.read(4))[0]
        
        points = []
        for _ in range(point_count):
            x = struct.unpack("<f", stream.read(4))[0]
            y = struct.unpack("<f", stream.read(4))[0]
            pressure = struct.unpack("<f", stream.read(4))[0]
            tilt_x = struct.unpack("<f", stream.read(4))[0]
            tilt_y = struct.unpack("<f", stream.read(4))[0]
            
            points.append(Point(
                x=x, y=y,
                pressure=pressure,
                tilt_x=tilt_x,
                tilt_y=tilt_y,
            ))
        
        try:
            pen_type = PenType(pen_type_raw)
        except ValueError:
            pen_type = PenType.UNKNOWN
        
        try:
            color = PenColor(color_raw)
        except ValueError:
            color = PenColor.BLACK
        
        return Stroke(
            pen_type=pen_type,
            color=color,
            points=points,
            base_width=base_width,
        )
    
    def _parse_stroke_v5(self, stream: BinaryIO) -> Stroke:
        """Parse a v5 stroke (adds speed to points)."""
        # Pen type and color
        pen_type_raw = struct.unpack("<I", stream.read(4))[0]
        color_raw = struct.unpack("<I", stream.read(4))[0]
        
        # Base width
        _unknown = struct.unpack("<I", stream.read(4))[0]
        base_width = struct.unpack("<f", stream.read(4))[0]
        
        # Unknown v5 field
        _unknown2 = struct.unpack("<I", stream.read(4))[0]
        
        # Point count
        point_count = struct.unpack("<I", stream.read(4))[0]
        
        points = []
        for _ in range(point_count):
            x = struct.unpack("<f", stream.read(4))[0]
            y = struct.unpack("<f", stream.read(4))[0]
            speed = struct.unpack("<f", stream.read(4))[0]
            tilt_x = struct.unpack("<f", stream.read(4))[0]
            pressure = struct.unpack("<f", stream.read(4))[0]
            tilt_y = struct.unpack("<f", stream.read(4))[0]
            
            points.append(Point(
                x=x, y=y,
                pressure=pressure,
                tilt_x=tilt_x,
                tilt_y=tilt_y,
                speed=speed,
            ))
        
        try:
            pen_type = PenType(pen_type_raw)
        except ValueError:
            pen_type = PenType.UNKNOWN
        
        try:
            color = PenColor(color_raw)
        except ValueError:
            color = PenColor.BLACK
        
        return Stroke(
            pen_type=pen_type,
            color=color,
            points=points,
            base_width=base_width,
        )
    
    def _parse_stroke_block(self, data: bytes) -> Stroke | None:
        """Parse a v6 stroke block."""
        if len(data) < 24:
            return None
        
        stream = io.BytesIO(data)
        
        try:
            # Skip CRDT header
            _parent_id = struct.unpack("<I", stream.read(4))[0]
            _item_id = struct.unpack("<I", stream.read(4))[0]
            _left_id = struct.unpack("<I", stream.read(4))[0]
            _right_id = struct.unpack("<I", stream.read(4))[0]
            _deleted = struct.unpack("<I", stream.read(4))[0]
            
            # Stroke data
            pen_type_raw = struct.unpack("<I", stream.read(4))[0]
            color_raw = struct.unpack("<I", stream.read(4))[0]
            base_width = struct.unpack("<f", stream.read(4))[0]
            
            # Point count
            point_count = struct.unpack("<I", stream.read(4))[0]
            
            points = []
            for _ in range(point_count):
                x = struct.unpack("<f", stream.read(4))[0]
                y = struct.unpack("<f", stream.read(4))[0]
                speed = struct.unpack("<f", stream.read(4))[0]
                tilt_x = struct.unpack("<f", stream.read(4))[0]
                pressure = struct.unpack("<f", stream.read(4))[0]
                tilt_y = struct.unpack("<f", stream.read(4))[0]
                
                points.append(Point(
                    x=x, y=y,
                    pressure=pressure,
                    tilt_x=tilt_x,
                    tilt_y=tilt_y,
                    speed=speed,
                ))
            
            try:
                pen_type = PenType(pen_type_raw)
            except ValueError:
                pen_type = PenType.UNKNOWN
            
            try:
                color = PenColor(color_raw)
            except ValueError:
                color = PenColor.BLACK
            
            return Stroke(
                pen_type=pen_type,
                color=color,
                points=points,
                base_width=base_width,
            )
            
        except Exception as e:
            logger.debug(f"Failed to parse stroke block: {e}")
            return None


# Convenience functions
def parse_rm(data: bytes) -> RmDocument:
    """Parse .rm data from bytes."""
    return RmParser().parse_bytes(data)


def parse_rm_file(path: Path | str) -> RmDocument:
    """Parse .rm file from disk."""
    return RmParser().parse_file(path)


def rm_to_svg(data: bytes, **kwargs) -> str:
    """Convert .rm data to SVG string."""
    doc = parse_rm(data)
    return doc.to_svg(**kwargs)
