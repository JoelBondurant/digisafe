## DigiSafe

A digital safe for secure storage of everything LastPass leaks and KeePass fails to backup.

### Features

 - 4GiB Argon2id / Sha3 salted and peppered master password derivation.
 - TPM locked pepper via systemd-creds.
 - Secret memory based master password storage.
 - The entire app is unswappable and undumpable.
 - Dynamic linker hijacking blockage.
 - Tracing blockage. (ptrace, strace, ltrace, gdb)
 - Forensic-resistant clipboard clearing mechanism.
 - Backblaze sync on load/save, with api creds TPM locked via systemd-creds.
 - Atomic file io.
 - Reed-Solomon 8:4 erasure coding with BLAKE3 checksums and 4KiB+ shards.
 - XChaCha20Poly1305 encryption.
 - LZ4 level 9 compression.
 - Simple TLV serialization.
 - CPU rending only via tiny-skia.
 - Wayland only.

### Known Weaknesses

 - iced String buffers.
 - no virtual keyboard input.
 - bpftrace.
 - wayland-data-control.

#### Screenshots:
![ScreenShot](https://github.com/JoelBondurant/digisafe/blob/main/doc/img/unlock_screen.png)

![ScreenShot](https://github.com/JoelBondurant/digisafe/blob/main/doc/img/empty_entry.png)

![ScreenShot](https://github.com/JoelBondurant/digisafe/blob/main/doc/img/sample_entry.png)

