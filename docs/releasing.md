# Releasing

Linux packages are built by GitHub Actions, not on a developer machine. This
means releases do not require access to a Linux box — and that every published
artefact is built the same way, from a clean checkout.

## Cutting a release

1. **Bump the version in both places.** They must agree, and CI enforces it:

   - `apps/desktop/src-tauri/tauri.conf.json` → `version`
   - `Cargo.toml` → `workspace.package.version`

2. **Commit and tag.** The tag drives everything:

   ```bash
   git commit -am "Release 0.2.0"
   git tag v0.2.0
   git push origin master --tags
   ```

3. **Wait for the build.** `release.yml` builds the `.deb`, `.rpm` and
   AppImage, generates `SHA256SUMS`, and creates a **draft** release.

4. **Review and publish.** Open the draft on the Releases page, check the
   generated notes, and publish. The draft step is deliberate: an accidental
   tag push does not become a public release on its own.

## Testing the packaging without releasing

Run the workflow manually from the Actions tab — pick **Release**, then **Run
workflow**. It builds exactly the same artefacts and uploads them as a build
artefact, but publishes nothing. Use this to check packaging changes before
committing to a tag.

## Why `ubuntu-22.04`

glibc is backwards-compatible but not forwards-compatible: a binary linked
against a newer glibc will not start on an older distribution. Building on the
oldest supported runner therefore gives the widest install base. Tauri v2 also
requires `webkit2gtk-4.1`, which rules out anything older.

If you bump this runner, the packages stop working on older distributions.
That is a deliberate decision, not a routine maintenance bump.

## What CI checks

`ci.yml` runs on every push and pull request:

| Job | Checks |
| --- | --- |
| Frontend | `vue-tsc` typecheck, Vite build, Vitest suite |
| Rust | `cargo fmt --check`, `clippy -D warnings`, `cargo test` |

By the time a tag is pushed, all of this has already passed — `release.yml`
only builds and packages.

## A note on the AppImage

The `.deb` and `.rpm` run `postinst.sh`, which installs the udev rule, creates
the `openlogi` group and adds the installing user to it.

**An AppImage runs no install script.** Anyone using it must run
`sudo ./scripts/install-udev-rules.sh` from the repository once, then log out
and back in. The release notes say so, but it is worth knowing when triaging
"it does not detect my device" reports from AppImage users.
