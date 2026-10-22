"""Generate CoreScope's original CPU glyph as a Windows DIB icon."""
import struct
from pathlib import Path
n = 32
pixels = bytearray()
for y in reversed(range(n)):
    for x in range(n):
        inside = 4 <= x < 28 and 4 <= y < 28
        pin = ((x in (2,3,28,29) and y in (9,10,15,16,21,22)) or
               (y in (2,3,28,29) and x in (9,10,15,16,21,22)))
        core = any(a <= x < a+5 and b <= y < b+5 for a in (9,18) for b in (9,18))
        rgba = (231,249,239,255) if core else (36,122,84,255) if inside or pin else (0,0,0,0)
        r,g,b,a=rgba
        pixels.extend((b,g,r,a))
dib=struct.pack('<IiiHHIIiiII',40,n,n*2,1,32,0,len(pixels),0,0,0,0)+pixels+bytes(n*4)
ico=struct.pack('<HHH',0,1,1)+struct.pack('<BBBBHHII',n,n,0,0,1,32,len(dib),22)+dib
Path(__file__).resolve().parents[1].joinpath('assets/corescope.ico').write_bytes(ico)
