# Pre-patched GfxEngine DLL Files

This directory contains pre-patched GfxEngine.dll files for The Settlers 4 GOG Gold Edition v2.50.1508.

## Files Included

### Production (Runtime)
- **GfxEngine_default.dll** (292 KB)
  - Original vanilla game DLL (1024×768)
  - Used as base for dynamic patching
  - **Embedded in production binary**

### Testing (Byte-Identical Verification)
- **GfxEngine_1024x600.dll** (292 KB) - Low resolution test
- **GfxEngine_1680x1050.dll** (292 KB) - Mid resolution test
- **GfxEngine_1920x1080.dll** (292 KB) - High resolution test

These 3 DLLs are used for the critical `test_dynamic_patch_produces_identical_dlls()` test, which verifies that dynamic patching produces byte-for-byte identical DLLs to the pre-patched versions.

**Total size:** 1.2 MB (4 files)

## What Happened to Other Resolutions?

The following pre-patched DLLs were **removed** as they're not needed for testing:
- ~~GfxEngine_1280x720.dll~~ - Can be dynamically generated
- ~~GfxEngine_1280x800.dll~~ - Can be dynamically generated
- ~~GfxEngine_1366x768.dll~~ - Can be dynamically generated
- ~~GfxEngine_1440x900.dll~~ - Can be dynamically generated
- ~~GfxEngine_1920x1200.dll~~ - Can be dynamically generated

**Savings:** 1.5 MB removed from repository

## How Dynamic Patching Works

The tool now uses dynamic patching to support **any resolution** (not just 8 predefined ones):

1. Starts with `GfxEngine_default.dll`
2. Patches 2 values at fixed offsets:
   - Offset 67214 (0x1068E): Screen width
   - Offset 67219 (0x10693): Screen height
3. Produces byte-identical output to pre-patched DLLs

This allows users to patch to **any resolution** (e.g., 3440×1440 ultrawide, 3840×2160 4K, etc.) without needing pre-patched files.

## Testing

The 3 test DLLs are only embedded in test builds (`cargo test`), not production builds (`cargo build --release`). They verify:

✅ Low resolution values (1024×600)
✅ Mid resolution values (1680×1050)
✅ High resolution values (1920×1080)

All 3 produce byte-identical results with dynamic patching.

## Regenerating Removed DLLs

If you need the removed DLLs for reference, regenerate them using dynamic patching:

```bash
# Patch the game to any resolution
cargo run -- patch --game-path ~/Games/Settlers4 --resolution 1280x720

# The patched GfxEngine.dll will be byte-identical to the original
```

## SHA1 Hashes

For GOG version validation:

| Resolution | SHA1 Hash |
|------------|-----------|
| Default (1024×768) | `F25CA243F617BB626614EFA8AB611509C971E6C4` |
| 1024×600 | `4968B9D20D87C901F57AB37F1BCAAC405365A89A` |
| 1680×1050 | `B9923B050E51C1A5F9E1DE8828861111DF811980` |
| 1920×1080 | `183DE9D83D2971AE9DCFD0E1ADB41A1A581C63FE` |

## Credits

Pre-patched DLLs originally from [FireEmerald's Settlers4-Widescreen-Tool](https://github.com/FireEmerald/Settlers4-Widescreen-Tool).

This Linux port adds dynamic patching to support unlimited resolutions without requiring pre-patched files.
