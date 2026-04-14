import { deflateSync } from "node:zlib";
import { writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const w = 512, h = 512;
const rows = Buffer.alloc((w * 4 + 1) * h);
const R = 0x4F, G = 0x46, B = 0xE5, A = 0xFF;
for (let y = 0; y < h; y++) {
  const off = y * (w * 4 + 1);
  rows[off] = 0;
  for (let x = 0; x < w; x++) {
    const i = off + 1 + x * 4;
    rows[i] = R; rows[i + 1] = G; rows[i + 2] = B; rows[i + 3] = A;
  }
}
const zdata = deflateSync(rows);

const crcTable = new Uint32Array(256);
for (let n = 0; n < 256; n++) {
  let c = n;
  for (let k = 0; k < 8; k++) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
  crcTable[n] = c >>> 0;
}
function crc32(buf) {
  let c = 0xFFFFFFFF;
  for (const b of buf) c = crcTable[(c ^ b) & 0xFF] ^ (c >>> 8);
  return (c ^ 0xFFFFFFFF) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length, 0);
  const tb = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(Buffer.concat([tb, data])), 0);
  return Buffer.concat([len, tb, data, crc]);
}
const sig = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4);
ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
const png = Buffer.concat([
  sig,
  chunk("IHDR", ihdr),
  chunk("IDAT", zdata),
  chunk("IEND", Buffer.alloc(0)),
]);

const __dirname = dirname(fileURLToPath(import.meta.url));
const outDir = resolve(__dirname, "../crates/stubhouse-app/icons");
for (const name of ["icon.png", "32x32.png", "128x128.png", "128x128@2x.png"]) {
  writeFileSync(resolve(outDir, name), png);
}
console.log("wrote placeholder icons to", outDir);
