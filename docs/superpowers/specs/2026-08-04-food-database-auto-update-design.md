# Auto-update food database on startup

## Goal
Automatically synchronize bundled and external food CSV sources when application starts, while skipping import when source contents are unchanged.

## Design
- Discover `.csv` files from bundled Tauri resources and external `DatabaseMakanan` directory.
- Compute SHA-256 digest for every source and build a deterministic manifest keyed by source filename and digest.
- Store manifest in `app_state` under `food_sources_manifest`.
- If current manifest matches stored manifest, skip food import.
- If manifest differs, replace food and nutrient data in one transaction, import all discovered sources in sorted order, and persist the new manifest in the same transaction.
- If no CSV sources are available, preserve existing food data and manifest.
- On read, hashing, or import failure, return the error and preserve the previous manifest/database through transaction rollback.

## Compatibility
- Existing manually imported foods remain available until a configured source change triggers a full synchronization.
- Existing `seed_complete` readiness state remains true after successful synchronization.
- Bundled resources continue using `tauri.conf.json`; external files are discovered from the project `DatabaseMakanan` directory and its existing runtime locations.

## Verification
- First startup imports sources and stores manifest.
- Repeated startup with unchanged contents skips import.
- Modified external or bundled source triggers synchronization.
- Failed synchronization leaves previous database and manifest intact.
- Existing import/search tests remain passing.
