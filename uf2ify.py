# SPDX-License-Identifier: MIT
# SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
"""
Pack a signed Bao1x firmware image into a UF2 file.

Usage:

    python3 uf2ify.py <signed-blob.img> <firmware.uf2>
"""
import sys
import struct


BAOCHIP_1X_UF2_FAMILY = 0xa7d7_6373
BAREMETAL_START = 0x60060000
UF2_MAGIC_START0 = 0x0A324655  # "UF2\n"
UF2_MAGIC_START1 = 0x9E5D5157
UF2_MAGIC_END = 0x0AB16F30


def main():
    if len(sys.argv) != 3:
        print("Usage: python3 uf2ify.py <signed-blob.img> <firmware.uf2>")
        sys.exit(1)

    with open(sys.argv[1], "rb") as f:
        signed_blob = f.read()
    print(f"signed blob file size is {len(signed_blob)} bytes")

    uf2 = uf2ify(bytes(signed_blob), BAOCHIP_1X_UF2_FAMILY, BAREMETAL_START)
    outfile = sys.argv[2]
    with open(outfile, "wb") as f:
        f.write(uf2)
    print(f"UF2 image written to {outfile}")


def uf2ify(data, family_id, app_start_addr):
    """Convert a binary blob into UF2 format (512-byte blocks)."""
    print(f"uf2ify data is {len(data)} bytes")
    nblocks = (len(data) + 255) // 256
    datapad = bytes(512 - 256 - 32 - 4)
    out = bytearray()

    for blockno in range(nblocks):
        ptr = 256 * blockno
        chunk = data[ptr:ptr + 256]
        if len(chunk) < 256:
            chunk += b'\x00' * (256 - len(chunk))

        flags = 0x2000 if family_id != 0 else 0

        header = struct.pack(
            "<8I",
            UF2_MAGIC_START0,
            UF2_MAGIC_START1,
            flags,
            ptr + app_start_addr,
            256,
            blockno,
            nblocks,
            family_id
        )

        out += header
        out += chunk
        out += datapad
        out += struct.pack("<I", UF2_MAGIC_END)

    return bytes(out)


if __name__ == "__main__":
    main()
