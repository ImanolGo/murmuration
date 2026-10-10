#!/usr/bin/env python3
"""Record real live ANSI frames without a browser (POSIX, Pillow, ffmpeg).

Build with `cargo build --release`, then run `python3 docs/record.py`.
VHS is also supported; this avoids browser capture stalls in headless setups.
"""
import argparse
import fcntl
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import time

from PIL import Image, ImageDraw, ImageFont


CELL = re.compile(r"\x1b\[38;2;(\d+);(\d+);(\d+)m\x1b\[48;2;(\d+);(\d+);(\d+)m(.)")


def rasterize(path, output, font):
    rows = [CELL.findall(row) for row in path.read_text().splitlines()]
    width = max(len(row) for row in rows)
    image = Image.new("RGB", (width * 9 + 32, len(rows) * 18 + 32), (20, 20, 20))
    draw = ImageDraw.Draw(image)
    for y, row in enumerate(rows):
        for x, cell in enumerate(row):
            fg, bg = tuple(map(int, cell[:3])), tuple(map(int, cell[3:6]))
            left, top = x * 9 + 16, y * 18 + 16
            draw.rectangle((left, top, left + 8, top + 17), fill=bg)
            if cell[6] == "▀":
                draw.rectangle((left, top, left + 8, top + 8), fill=fg)
            elif cell[6] != " ":
                draw.text((left, top), cell[6], font=font, fill=fg)
    image.save(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seconds", type=float, default=45.0)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--birds", type=int, default=4000)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--font", help="path to a monospace font with braille glyphs")
    args = parser.parse_args()
    if args.seconds < 2:
        parser.error("--seconds must be at least 2")
    root = Path(__file__).resolve().parent.parent
    output = (args.output or root / "docs/murmuration.gif").resolve()
    font_path = args.font
    if not font_path:
        try:
            font_path = subprocess.check_output(
                ["fc-match", "-f", "%{file}", "monospace:charset=28ff"], text=True,
            ).strip()
        except (OSError, subprocess.CalledProcessError):
            font_path = "DejaVuSansMono.ttf"
    font = ImageFont.truetype(font_path, 15)
    if bytes(font.getmask("⣿")) == bytes(font.getmask("\uffff")):
        parser.error("font has no braille glyphs; choose a compatible --font")
    temp_root = "/tmp/opencode" if Path("/tmp/opencode").is_dir() else None
    with tempfile.TemporaryDirectory(prefix="murmuration-record-", dir=temp_root) as tmp:
        directory = Path(tmp)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        process = subprocess.Popen(
            [str(root / "target/release/murmuration"), "--no-arc", "--falcon",
             "--seed", str(args.seed), "--birds", str(args.birds),
             "--colors", "truecolor", "--no-mouse"],
            stdin=slave, stdout=slave, stderr=slave, cwd=directory,
            env=dict(os.environ, TERM="xterm-256color", COLORTERM="truecolor"),
        )
        os.close(slave)
        start = time.monotonic()
        next_frame = 1.0
        try:
            while process.poll() is None and time.monotonic() - start < args.seconds:
                elapsed = time.monotonic() - start
                if elapsed >= next_frame:
                    os.write(master, b"s")
                    next_frame = elapsed + 0.1
                if select.select([master], [], [], 0.01)[0]:
                    os.read(master, 65536)
            if process.poll() is None:
                os.write(master, b"q")
                deadline = time.monotonic() + 5
                while process.poll() is None and time.monotonic() < deadline:
                    if select.select([master], [], [], 0.01)[0]:
                        try:
                            os.read(master, 65536)
                        except OSError:
                            break
                process.wait(timeout=5)
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=5)
            os.close(master)
        if process.returncode != 0:
            raise RuntimeError(f"live demo exited with {process.returncode}")
        # Use simulation timestamps, not an assumed capture rate: missing a
        # capture must not accelerate the birds or freeze an entire recording.
        captures = sorted(directory.glob("*.ansi"), key=lambda p: float(p.stem.rsplit("-", 1)[1][:-1]))
        if len(captures) < 2:
            raise RuntimeError("the live demo did not produce enough frames")
        manifest = []
        for i, path in enumerate(captures):
            image = directory / f"frame-{i:05}.png"
            rasterize(path, image, font)
            manifest.append(f"file '{image}'")
            t = float(path.stem.rsplit("-", 1)[1][:-1])
            next_t = float(captures[i + 1].stem.rsplit("-", 1)[1][:-1]) if i + 1 < len(captures) else t + 0.1
            manifest.append(f"duration {max(0.01, next_t - t):.3f}")
        concat = directory / "frames.txt"
        concat.write_text("\n".join(manifest) + "\n")
        subprocess.run(
            ["ffmpeg", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0",
             "-i", str(concat), "-filter_complex",
             "split[a][b];[a]palettegen[p];[b][p]paletteuse=dither=bayer:bayer_scale=5",
             "-fps_mode", "vfr", "-loop", "0", str(output)], check=True,
        )
        print(f"Recorded {len(captures)} live frames to {output}")


if __name__ == "__main__":
    main()
