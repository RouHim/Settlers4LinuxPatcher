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

## How Dynamic Patching Works

The tool now uses dynamic patching to support **any resolution** (not just 8 predefined ones):

1. Starts with `GfxEngine_default.dll`
2. Patches 2 values at fixed offsets:
   - Offset 67214 (0x1068E): Screen width
   - Offset 67219 (0x10693): Screen height
3. Produces byte-identical output to pre-patched DLLs

This allows users to patch to **any resolution** (e.g., 3440×1440 ultrawide, 3840×2160 4K, etc.) without needing pre-patched files.

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
