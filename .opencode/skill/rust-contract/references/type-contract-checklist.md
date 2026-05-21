# Type Contract Checklist

- Replace stringly IDs and primitive domain values with newtypes.
- Replace boolean behavior flags with enums.
- Replace `Option` lifecycle state with explicit state variants.
- Parse external input once at the boundary.
- Represent domain failures with semantic error variants.
- Keep pure core free of I/O, time, network, storage, and randomness.
