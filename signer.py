# SPDX-License-Identifier: MIT
# SPDX-FileCopyrightText: Copyright 2026 Sam Blenny
"""
Sign a binary firmware blob for Baochip Bao1x bootloader. This needs to be
followed by a UF2 packing stage, but that happens with a different script.

Usage:

    python3 signer.py <firmware.bin> <signed-blob.img>

This is a re-implimentation of xous-core/tools/src/bin/sign_image.rs intended
to facilitate bare metal development for Baochip free of Xous dependencies.

To run this, you need a recent version of libsodium, on Debian you can do:

    sudo apt install libsodium-dev

"""
import base64
import ctypes  # used for ctypes.CDLL("libsodium.so")
import os
import sys
import struct


# Developer signing key private key PEM (publicly disclosed, non-production)
# Source: xous-core/devkey/dev.key
DEV_KEY_PEM = """-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIKindlyNoteThisIsADevKeyDontUseForProduction
-----END PRIVATE KEY-----"""
# Raw key bytes using a PEM + ASN.1 decoder for the impatient
DEV_KEY_RAW = base64.b64decode(DEV_KEY_PEM.split('\n')[1])[-32:]

# Length of signature block including SignatureInFlash and SealedData's header
SIGBLOCK_LEN = 768

# Length of SignedBlob's header prefix (include signature, exclude sealed_data)
SBLOB_LEN = 132


class Pubkey:
    def __init__(self, pk, tag):
        self.pk = bytes.fromhex(pk)                                  # [u8;32]
        self.tag = tag                                               # [u8;4]
        assert len(self.pk) == 32, "expected pk length 32 bytes"
        assert len(self.tag) == 4, "expected tag length 4 bytes"

    def __bytes__(self):
        buf = self.pk + self.tag
        assert len(buf) == 36, "expected Pubkey length 36"
        return buf

PUBKEYS = (
    Pubkey("00" * 32,  b'\x00' * 4),
    Pubkey("79135dc667aff4f7d352b90328788ebf92c786782138b377370b15194e312888",
        b'bao2'),
    Pubkey("80979929edd04e40124b52cae9ae54b24bdff72a7b8a004c41065bd1402078a7",
        b'beta'),
    Pubkey("1c9beae32aeac87507c18094387eff1c74614282affd8152d871352edf3f58bb",
        b'dev '),  # note, space-padded (not null-padded!)
)


class SignedBlob:
    def __init__(self, dev_sk, sealed_data):
        # calculate ed25519ph signature
        dev_pk = PUBKEYS[3].pk
        signature = ed25519ph_sign(dev_sk, dev_pk, sealed_data)
        assert len(signature) == 64, "expected signature length 64"

        self.jal_x0 = 0x3000006f        # jump to payload start: 768,  u32
        self.signature = signature      # ed25519ph signature,         [u8;64]
        self.aad_len = 0                # unused Xous FIDO stuff,      u32
        self.aad = bytearray(60)        # unused Xous FIDO stuff,      [u8;60]
        self.sealed_data = sealed_data  # this is what gets signed,    [u8;?]
        assert SBLOB_LEN == 132, "expected SBLOB_LEN 132"

    def __bytes__(self):
        buf = struct.pack("<I 64s I 60s",
            self.jal_x0,
            self.signature,
            self.aad_len,
            self.aad
        )
        assert len(buf) == SBLOB_LEN, "expected SignedBlob header length 132"
        return buf + self.sealed_data


class SealedData:
    def __init__(
        self,
        payload,
        function_code=None,
        min_semver=None,
        semver=None
    ):
        self.version = 0x0100                                        # u32
        # NOTE: This big-endian unpack intentionally reverses the byte order
        # from left-to-right readable to the "ymuy3oaB" expected by bootloader.
        # Don't ask me why the bootloader wants it that way -- don't know.
        self.magic = struct.unpack(">2I", b"yumyBao3")               # [u32;2]
        self.signed_len = SIGBLOCK_LEN - SBLOB_LEN + len(payload)    # u32
        # See enum in xous-core/libs/bao1x-api/src/signatures.rs and usage in
        # xous-core/src/toos/sign_image.rs. Baremetal function code is 6.
        self.function_code = function_code or 6                      # u32
        self.reserved = 0                                            # u32
        self.min_semver = min_semver or bytearray(16)                # [u8;16]
        self.semver = semver or bytearray(16)                        # [u8;16]
        self.pubkeys = b''.join([bytes(pk) for pk in PUBKEYS])       # [u8;144]

        assert len(self.magic) == 2, "expected magic length [u32;2]"
        assert len(self.min_semver) == 16, "expected min_semver length [u8;16]"
        assert len(self.semver) == 16, "expected semver length [u8;16]"
        assert len(self.pubkeys) == 36*4, "expected pubkeys length [u8;144]"
        assert SIGBLOCK_LEN == 768, "expected SIGBLOCK_LEN 768"

        fields_len = (4 * 6) + 16 + 16 + 144                        # = 200
        pad_len = SIGBLOCK_LEN - SBLOB_LEN - fields_len             # = 436
        self.pad = b'\x00' * pad_len
        self.payload = payload

    def __bytes__(self):
        buf = struct.pack("<I 2I I I I 16s 16s 144s",
            self.version,
            self.magic[0], self.magic[1],
            self.signed_len,
            self.function_code,
            self.reserved,
            self.min_semver,
            self.semver,
            self.pubkeys
        )
        assert len(buf) == 200, "expected SealedData header fields length 200"
        assert len(self.pad) == 436, "expected SealedData pad length 436"
        return b''.join((buf, self.pad, self.payload))


def main():
    if len(sys.argv) != 3:
        print("Usage: python3 signer.py <firmware.bin> <signed-blob.img>")
        sys.exit(1)

    assert test_ed25519ph(), "ed25519 test vector signing failed"

    with open(sys.argv[1], "rb") as f:
        payload = f.read()
    print(f"binary payload size is {len(payload)} bytes")
    # Check if file already has the sealed_data header. If so, slice it off.
    starts_with_jal = payload[:4] == bytes.fromhex("6f000030")
    matches_magic = payload[0x88:0x90] == b'ymuy3oaB'
    if starts_with_jal and matches_magic:
        # Input file has a sealed_data header. Slice it off to make a new one.
        print("slicing 768 header bytes off the input file")
        payload = payload[SIGBLOCK_LEN:]

#     min_sem = b"\x00" * 16
#     semver = b"\x00" * 16
    min_semver = bytes.fromhex("00000900080017030000000000000000")
    semver     = bytes.fromhex("000009001000FC09F229F54701000000")

    blob = bytes(SealedData(payload, min_semver=min_semver, semver=semver))
    signed_blob = SignedBlob(DEV_KEY_RAW, sealed_data=blob)
    outfile = sys.argv[2]
    with open(outfile, "wb") as f:
        f.write(bytes(signed_blob))
    print(f"Signed firmware blob written to {outfile}")


# ===========================================================================
# ed25519ph stuff using libsodium + ctypes (sudo apt install python3-libnacl)
# ===========================================================================

# Load libsodium
na = ctypes.CDLL("libsodium.so")

# Function signatures
na.sodium_init.restype = ctypes.c_int

na.crypto_sign_ed25519ph_statebytes.restype = ctypes.c_size_t

na.crypto_sign_ed25519ph_init.argtypes = [ctypes.c_void_p]
na.crypto_sign_ed25519ph_init.restype = ctypes.c_int

na.crypto_sign_ed25519ph_update.argtypes = [ctypes.c_void_p, ctypes.c_void_p,
    ctypes.c_ulonglong]
na.crypto_sign_ed25519ph_update.restype = ctypes.c_int

na.crypto_sign_ed25519ph_final_create.argtypes = [ctypes.c_void_p,
    ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
na.crypto_sign_ed25519ph_final_create.restype = ctypes.c_int

# Initialize libsodium
assert na.sodium_init() >= 0, "libsodium initialization failed"

def ed25519ph_sign(secret_key_bytes, public_key_bytes, message):
    """
    Sign message using Ed25519ph with raw 32-byte secret key and public key.
    """
    if len(secret_key_bytes) != 32 or len(public_key_bytes) != 32:
        raise ValueError("sk_bytes and pk_bytes must be 32 bytes each")

    # Create libsodium's 64-byte secret key format: seed || public_key
    sk = secret_key_bytes + public_key_bytes

    # Sign using ed25519ph
    state_len = na.crypto_sign_ed25519ph_statebytes()
    state = ctypes.create_string_buffer(state_len)
    if na.crypto_sign_ed25519ph_init(state) != 0:
        raise RuntimeError("crypto_sign_ed25519ph_init() failed")
    if na.crypto_sign_ed25519ph_update(state, message, len(message)) != 0:
        raise RuntimeError("crypto_sign_ed25519ph_update() failed")
    signature = ctypes.create_string_buffer(64)
    if na.crypto_sign_ed25519ph_final_create(state, signature, None, sk) != 0:
        raise RuntimeError("crypto_sign_ed25519ph_final_create() failed")

    return signature.raw

def test_ed25519ph():
    """
    Test ed25519ph signing using the RFC8032 section 7.3 test vector.
    """
    secret_key = bytes.fromhex(
        '833fe62409237b9d62ec77587520911e'
        '9a759cec1d19755b7da901b96dca3d42'
    )
    public_key = bytes.fromhex(
        'ec172b93ad5e563bf4932c70e1245034'
        'c35467ef2efd4d64ebf819683467e2bf'
    )
    message = bytes.fromhex("616263")
    signature = bytes.fromhex(
        '98a70222f0b8121aa9d30f813d683f80'
        '9e462b469c7ff87639499bb94e6dae41'
        '31f85042463c2a355a2003d062adf5aa'
        'a10b8c61e636062aaad11c2a26083406'
    )
    buf = ed25519ph_sign(secret_key, public_key, message)
    return buf == signature


if __name__ == "__main__":
    main()
