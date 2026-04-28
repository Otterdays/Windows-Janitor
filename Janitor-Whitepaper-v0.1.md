# Janitor: A Trust-First Windows System Cleaner

**Whitepaper v0.1**
**Working codename:** Janitor
**Target platform:** Windows 10 (22H2+) and Windows 11
**Scope:** Architecture, functionality, threat model, and engineering plan for a future-first Windows OS cleanup utility.

---

## 1. Abstract

- Windows accumulates junk: temp files, caches, broken registry entries, dead startup hooks, orphan installers, stale logs, abandoned downloads.
- No trustworthy native or third-party tool currently solves this end-to-end.
- CCleaner: compromised in 2017 (Floxif/ShadowPad), now Avast-owned bloatware with telemetry.
- Storage Sense (built-in): shallow, opaque, no registry / startup / classification.
- Goal: a fully local, open source, reversible, fast cleaner. No telemetry. No upsells. No black boxes.
- This whitepaper specifies the full system: architecture, scanners, classification, quarantine, privilege model, AI assist, and roadmap.

---

## 2. Problem Statement

### 2.1 What "junk" actually means on Windows

- **Filesystem junk**
  - Temp dirs (`%TEMP%`, `C:\Windows\Temp`, per-service temp dirs)
  - Browser caches (Chrome, Edge, Firefox, Brave — per profile)
  - App caches (Discord, Slack, Teams, Spotify, Steam shadercache, Epic Games)
  - Crash dumps, Windows Error Reporting (WER), `MEMORY.DMP`, minidumps
  - Update leftovers (`SoftwareDistribution\Download`, DeliveryOptimization, `$WINDOWS.~BT`, `$WINDOWS.~WS`, `Windows.old`)
  - Log files (event logs rotated, IIS, CBS, DISM, setupact, setupapi)
  - Thumbnail / icon / font cache
  - Recycle Bin contents per volume
  - Orphan installer payloads (`C:\ProgramData\Package Cache`)
  - Driver installer leftovers (NVIDIA, AMD, Intel, Realtek)
  - Developer caches (npm, pnpm, yarn, cargo, pip, gradle, maven) — opt-in only
  - Abandoned downloads (installers in Downloads folder never executed)
  - Duplicates (hash-equal, not just name-equal)
  - Empty nested folders, zero-byte files, stale `.tmp`

- **Registry junk**
  - Orphan uninstall entries (`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`)
  - Dead file associations
  - Stale CLSID / COM entries pointing at missing DLLs
  - Broken AppPath entries
  - MUI cache bloat
  - `Run` / `RunOnce` entries with deleted targets
  - Stale Shell extensions

- **Execution surface junk**
  - Startup folder, `Run` keys, Task Scheduler, Services, WMI subscriptions
  - Scheduled tasks with broken target binaries
  - Disabled-but-present startup items
  - Auto-start drivers no longer needed

- **Behavioral junk** (heuristic, not deterministic)
  - Files in Downloads untouched for N days
  - Files matching installer naming patterns (`setup*.exe`, `*-installer.msi`) older than threshold
  - Application directories with no recent access whose parent app is uninstalled

### 2.2 Constraints

- Windows is hostile to bulk filesystem operations. Locked files, ACL denials, reparse point loops, ADS streams, MAX_PATH, OneDrive placeholders, case-sensitive flags.
- WinSxS (component store) cannot be cleaned by file deletion — only via DISM. Touching it bricks systems.
- Antivirus engines flag any program that enumerates registry + walks filesystem + deletes in protected paths. False positive rate without code signing is ~100%.
- Trust is the binding constraint, not capability.

---

## 3. Design Principles

1. **Reversible by default.** Nothing deletes immediately. Everything quarantines first.
2. **Transparent.** Every flagged item shows the rule that matched it.
3. **Local-only.** Zero network calls except for explicit user-initiated update checks.
4. **Open source.** GPLv3 or MPL-2.0. Reproducible builds.
5. **Scope-disciplined.** Phases gated. No feature creep into the engine.
6. **Privilege-minimal.** UI never runs as admin. Elevated helper does the work.
7. **Read-before-write.** Every destructive module ships read-only first.
8. **Fail safe.** Errors abort the operation, never proceed partially.

---

## 4. System Architecture

### 4.1 Layered model

```
┌─────────────────────────────────────────────────────────┐
│  Layer 6: UI (Tauri / WebView2 frontend)                │  unprivileged
├─────────────────────────────────────────────────────────┤
│  Layer 5: Application service (Rust)                    │  unprivileged
│           - scan orchestration                          │
│           - report generation                           │
│           - quarantine browser                          │
├─────────────────────────────────────────────────────────┤
│  Layer 4: IPC (named pipe, signed messages)             │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Privilege broker (Windows service)            │  SYSTEM / admin
│           - executes deletions                          │
│           - registry writes                             │
│           - service / task changes                      │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Engine library (Rust crate)                   │
│           - scanners                                    │
│           - classifiers                                 │
│           - quarantine manifest                         │
│           - undo log                                    │
├─────────────────────────────────────────────────────────┤
│  Layer 1: Rule store (TOML files, signed)               │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Component responsibilities

- **Rule store** — declarative definitions of what counts as junk. Versioned. Signed by maintainers. Loadable from app dir + user dir (for custom rules).
- **Engine library** — pure Rust crate, no UI dependencies. Implements `Scanner`, `Classifier`, `Quarantine`, `UndoLog` traits. Unit-testable in isolation.
- **Privilege broker** — a separately-installed Windows service (or scheduled COM elevation). Receives signed JSON commands over named pipe. Validates origin, schema, and rule version. Refuses anything unsigned or out of schema.
- **IPC** — Windows named pipe with SDDL restricting access to the installing user + SYSTEM. Messages are CBOR-encoded with HMAC.
- **Application service** — orchestrates scans, builds reports, manages quarantine browsing. Runs as the user. No filesystem writes outside `%LOCALAPPDATA%\Janitor`.
- **UI** — Tauri shell. Pure rendering + user input. No business logic.

### 4.3 Stack rationale

- **Rust** for engine + service + broker
  - No GC pauses on million-file scans
  - `windows-rs` gives full Win32 surface
  - `rayon` for parallel hashing / walking
  - Single static binary, easy to sign and ship
  - Memory safety reduces footgun count in privileged code
- **Tauri** for UI
  - 5–10 MB shipped vs ~150 MB Electron
  - Web frontend lets us reuse skills (React or Svelte)
  - WebView2 already on every modern Windows install
- **SQLite** for persistence
  - Scan history, quarantine manifest, undo log
  - Single-file, atomic, well-understood
- **TOML** for rule files
  - Human-readable, diff-friendly, supports comments

### 4.4 Alternative stacks considered

- **C# + WinUI 3** — viable. Native feel. But heavier .NET runtime, slower walk performance under load, and Microsoft's tooling churn (UWP→WinUI2→WinUI3→…) is a long-term liability.
- **Go** — simpler than Rust but GC pauses hurt at scale, and the Windows API surface is less ergonomic.
- **Electron** — rejected. Ironic to ship a 200MB cleaner.
- **Pure Win32 / C++** — fastest, but every memory bug becomes a privilege escalation in the broker. Not worth it.

---

## 5. Scanner Engine

### 5.1 Scanner trait

```rust
trait Scanner {
    fn id(&self) -> &'static str;
    fn category(&self) -> Category;
    fn requires_elevation(&self) -> bool;
    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>, ScanError>;
}
```

- Every scanner is independent. No shared mutable state.
- `Finding` records: path or registry key, size, last access, matched rule, confidence, suggested action.
- Scanners run in parallel via `rayon` thread pool, capped at `cpu_count - 1`.

### 5.2 Built-in scanners (Phase 1–3)

- `temp_dirs` — `%TEMP%`, `%LOCALAPPDATA%\Temp`, `C:\Windows\Temp`
- `recycle_bin` — per-volume `$Recycle.Bin`
- `windows_update` — `SoftwareDistribution\Download`, DeliveryOptimization, `Windows.old`, `$WINDOWS.~BT`
- `crash_dumps` — `%LOCALAPPDATA%\CrashDumps`, `MEMORY.DMP`, WER queue
- `thumbnail_cache` — `%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db`, `IconCache.db`
- `event_logs` — rotated `.evtx` outside the active set
- `browser_caches` — Chrome / Edge / Firefox / Brave per profile, with profile auto-discovery
- `app_caches` — registry-driven list (Discord, Slack, Teams, Spotify, Steam shadercache, Epic Games)
- `installer_leftovers` — `C:\ProgramData\Package Cache` orphans (cross-referenced against installed product table)
- `downloads_orphans` — heuristic: installer-shaped files in Downloads, age > N days, never executed
- `dev_caches` — opt-in: npm, pnpm, yarn, cargo, pip, gradle
- `empty_dirs` — recursive empty / zero-byte cleanup (gated; opt-in)
- `duplicates` — content-hash dedup (BLAKE3, two-phase: size group → hash)

### 5.3 Walk strategy

- Use `FindFirstFileExW` with `FIND_FIRST_EX_LARGE_FETCH` for throughput.
- Resolve every path through `\\?\` prefix to handle long paths (>260 chars).
- Detect reparse points before recursing. Never follow:
  - Mount points to other volumes
  - Symbolic links pointing outside the original root
  - OneDrive / cloud placeholder reparse tags (`IO_REPARSE_TAG_CLOUD*`) — these are not local files; touching them triggers downloads.
- Read Alternate Data Streams via `FindFirstStreamW` only when the rule explicitly requires it.
- Skip files marked `FILE_ATTRIBUTE_OFFLINE` and cloud-pinned states unless user explicitly opts in.

### 5.4 Hard blacklist (never-touch)

Scanners must never propose findings inside:

- `C:\Windows\WinSxS` (component store — DISM territory)
- `C:\Windows\System32`, `C:\Windows\SysWOW64`
- `C:\Windows\servicing`
- `C:\Windows\assembly`
- Any path whose ACL grants `TrustedInstaller` exclusive write
- `pagefile.sys`, `hiberfil.sys`, `swapfile.sys` (kernel-controlled)
- BitLocker metadata regions
- Any path inside an active OneDrive / Dropbox / iCloud sync root flagged as cloud-only

This list is enforced as a final filter after scanner output, before findings reach the user. Defense in depth.

---

## 6. Rule Engine

### 6.1 Rule format

```toml
[rule]
id = "browser.chrome.cache"
category = "browser_cache"
version = 1
description = "Chrome browser cache files"
requires_elevation = false
risk = "low"

[rule.match]
paths = [
  "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\*\\Cache",
  "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\*\\Code Cache",
]
extensions = ["*"]
min_age_days = 0
exclude_locked = true

[rule.action]
default = "quarantine"
allow_purge_after_days = 30
```

### 6.2 Rule properties

- **Versioned.** Engine refuses rules with unknown schema version.
- **Signed.** Official rule pack bundled with binary; signature checked at load.
- **Composable.** Rules can reference other rules' exclusions.
- **Risk-tagged.** `low`, `medium`, `high`. UI defaults filter by risk band.
- **Auditable.** Each finding records the exact rule ID and version that matched.

### 6.3 User extensibility

- Users can drop custom rules in `%APPDATA%\Janitor\rules\`.
- Custom rules are flagged in the UI as unsigned.
- Custom rules cannot override hard blacklist.
- Custom rules cannot escalate risk band of a built-in rule.

---

## 7. Classification System

### 7.1 Two-tier classifier

- **Tier 1 — Deterministic.** Path + extension + age + ACL match against rules. Output: certain.
- **Tier 2 — Heuristic.** For unknown files in user space, score by:
  - Filename entropy (random hash names → cache-like)
  - Size distribution
  - Last access time
  - Parent directory population
  - Whether parent app is installed
  - Magic bytes vs extension
- Heuristic findings are always presented as suggestions with confidence score, never auto-actioned.

### 7.2 Tier 3 — Local AI assist (Phase 6)

- Optional local SLM (Phi-3.5-mini, Qwen2.5-3B, or similar GGUF via `llama.cpp`)
- Input: redacted path, size, age, magic bytes, parent context
- Output: classification label + confidence + one-sentence reasoning
- Runs entirely offline. Model bundled or user-downloaded.
- AI output is advisory only. Cannot delete. Cannot quarantine without user confirmation.
- Reasoning text shown to user verbatim.

---

## 8. Quarantine & Undo

### 8.1 Quarantine flow

1. User accepts findings → engine sends `quarantine` command to broker.
2. Broker moves files to `%PROGRAMDATA%\Janitor\Quarantine\<scan-id>\`.
3. Original ACLs preserved. Original path stored in manifest.
4. Manifest entry: `{ id, original_path, new_path, size, hash, rule_id, timestamp }`.
5. SQLite `undo_log` row written before move; `committed=true` only on success.
6. Files held for configurable retention (default 30 days), then purged.

### 8.2 Registry quarantine

- Registry edits never delete inline.
- Each edit produces a `.reg` export of the affected subtree first.
- Export stored in quarantine dir alongside file manifest.
- Restore = re-apply `.reg` file via `reg import`.

### 8.3 Undo

- Any quarantine entry restorable until purge.
- Bulk restore by scan ID.
- Undo log immutable; restores produce new rows, not overwrites.

### 8.4 Purge

- Manual purge: user button.
- Auto purge: daemon checks daily, removes entries older than retention.
- Purge is the only true delete. Everything else is a move.

---

## 9. Privilege Model

### 9.1 Why split

- A single-process admin app means every UI bug is a privilege escalation.
- Splitting reduces attack surface: UI code can have bugs without filesystem consequences.

### 9.2 Broker design

- Installed as Windows Service running as `LocalSystem` (or per-user elevated COM server, depending on install mode).
- Listens on named pipe `\\.\pipe\janitor-broker-<sid>`.
- Pipe ACL restricted to installing user + SYSTEM via SDDL.
- Accepts only signed CBOR messages from a known schema.
- Each command references a finding ID from the most recent scan; broker validates against the SQLite manifest before acting.
- No free-form path arguments. Broker resolves paths from scan records, not from message payload.

### 9.3 Command set (minimal)

- `Quarantine(finding_ids: [u64])`
- `Restore(quarantine_ids: [u64])`
- `Purge(quarantine_ids: [u64])`
- `RegistryExport(key: KeyRef)` — read-only
- `RegistryDelete(key: KeyRef, value: Option<String>)`
- `ServiceDisable(name: String)` — never delete services
- `TaskDisable(name: String)`

Note: no generic `DeleteFile(path)`. Every destructive operation is mediated through a finding ID.

### 9.4 Audit log

- Broker writes append-only audit log (separate from undo log) to `%PROGRAMDATA%\Janitor\audit\`.
- Each entry: timestamp, command, finding ref, caller SID, result.
- Log rotated weekly, signed with HMAC chain (each entry hashes previous).

---

## 10. Windows Subsystem Specifics

### 10.1 Filesystem footguns

- **MAX_PATH (260 chars).** Always use `\\?\` prefix on path inputs to Win32 APIs.
- **Reparse points.** Check `FILE_ATTRIBUTE_REPARSE_POINT` and `IO_REPARSE_TAG_*` before recursing.
- **Cloud placeholders.** OneDrive uses `IO_REPARSE_TAG_CLOUD`. Do not open these unless user has elected cloud mode.
- **Case sensitivity.** Per-directory case sensitivity flag exists since Win10 1803. Honor it.
- **Junctions vs symlinks.** Junctions are filesystem-level redirects; symlinks require privilege to create but not to follow. Treat both with caution.
- **Locked files.** Use `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` and accept that some files will remain locked. Skip with clear reason in report.
- **Sparse files / compressed files.** Report logical size and on-disk size separately.

### 10.2 Registry

- Use `RegOpenKeyExW` with `KEY_WOW64_64KEY` and `KEY_WOW64_32KEY` separately on x64 — same logical key has two physical views.
- Never recurse into `HKLM\SAM` or `HKLM\SECURITY` — access denied even as SYSTEM without specific privileges, and there is nothing useful for cleaning there.
- For uninstall scanning: read both `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` and `HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall`, plus per-user equivalents.
- Validate orphan candidates by checking `InstallLocation`, `UninstallString`, and `DisplayIcon` — only flag as orphan if all referenced paths are absent.

### 10.3 Services and tasks

- Services enumerated via SCM (`EnumServicesStatusExW`).
- Never delete services. Only disable. Service deletion can break Windows Update chains.
- Tasks enumerated via Task Scheduler 2.0 COM API (`ITaskService`).
- Flag tasks whose `Actions` reference missing executables.

### 10.4 Startup

- Aggregate from:
  - `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` and `RunOnce`
  - `HKLM\Software\Microsoft\Windows\CurrentVersion\Run` and `RunOnce`
  - WOW6432Node equivalents
  - `Startup` folders (per-user, all-users)
  - Task Scheduler tasks with logon triggers
  - Services with `Auto` start type
  - WMI `__EventConsumer` subscriptions (often missed; common malware persistence)

### 10.5 WinSxS / Component Store

- Documented and enforced as untouchable.
- The only correct way to reduce its size is `DISM /Online /Cleanup-Image /StartComponentCleanup`.
- Janitor will offer a button that invokes DISM with these flags and shows the output. Janitor itself does not modify the store.

---

## 11. Storage Visualization

- Treemap view (à la WinDirStat / SpaceSniffer) built into the main UI.
- Backing data: a single fast walk of the volume that records `(path, size, last_access, owner_app)`.
- Cached in SQLite, regenerated on demand or on schedule.
- Coloring by category (system, user, cache, junk, unknown).
- Click-through into directory drill-down.
- Inline action: any treemap node can be sent to scan input.

---

## 12. Data Persistence

### 12.1 SQLite schema (abbreviated)

```sql
CREATE TABLE scan (
  id INTEGER PRIMARY KEY,
  started_at INTEGER,
  finished_at INTEGER,
  scanner_set TEXT,
  rule_pack_version TEXT
);

CREATE TABLE finding (
  id INTEGER PRIMARY KEY,
  scan_id INTEGER REFERENCES scan(id),
  scanner_id TEXT,
  rule_id TEXT,
  rule_version INTEGER,
  target_kind TEXT,        -- 'file' | 'dir' | 'regkey' | 'regvalue' | 'service' | 'task'
  target_ref TEXT,         -- path or key, opaque to broker
  size INTEGER,
  age_days INTEGER,
  confidence REAL,
  risk TEXT,
  suggested_action TEXT
);

CREATE TABLE quarantine (
  id INTEGER PRIMARY KEY,
  finding_id INTEGER REFERENCES finding(id),
  original_ref TEXT,
  storage_ref TEXT,
  hash TEXT,
  quarantined_at INTEGER,
  purge_after INTEGER,
  status TEXT              -- 'held' | 'restored' | 'purged'
);

CREATE TABLE audit (
  id INTEGER PRIMARY KEY,
  ts INTEGER,
  command TEXT,
  finding_id INTEGER,
  caller_sid TEXT,
  result TEXT,
  prev_hmac TEXT,
  this_hmac TEXT
);
```

### 12.2 File layout

```
%PROGRAMDATA%\Janitor\
  janitor.db                  # main SQLite
  audit\                      # rotated audit logs
  Quarantine\
    <scan-id>\
      manifest.json
      files\…
      registry\*.reg
  rules\                      # bundled signed rules
%APPDATA%\Janitor\
  config.toml
  rules\                      # user-added rules (unsigned)
%LOCALAPPDATA%\Janitor\
  cache\                      # treemap cache, scan intermediates
  logs\
```

---

## 13. Security

### 13.1 Code signing

- EV certificate required for SmartScreen reputation.
- Sign: main exe, broker service exe, every DLL, the MSI installer, and the rule pack.
- Reproducible builds; publish SHA-256 of every release artifact.
- Submit to Microsoft Defender, Kaspersky, ESET, Bitdefender for whitelisting before public release.

### 13.2 Update channel

- Updates are user-initiated only.
- Update server returns a signed manifest. Client verifies signature against pinned public key bundled in the binary.
- No auto-update for the broker service without explicit consent.

### 13.3 Threat model

- **Adversary 1: malicious local app trying to use Janitor as a confused deputy.**
  - Mitigation: pipe ACL restricts to installing user. Commands reference only finding IDs from current scan, not free paths.
- **Adversary 2: tampering with on-disk rule files.**
  - Mitigation: signed rule pack; user rules sandboxed and cannot bypass blacklist.
- **Adversary 3: supply chain attack on dependencies.**
  - Mitigation: minimal dependency set, vendored where reasonable, `cargo deny` enforced, reproducible builds.
- **Adversary 4: compromised release pipeline.**
  - Mitigation: detached signing on offline machine; CI cannot sign.

---

## 14. Performance

### 14.1 Targets

- Cold scan of 500 GB SSD with 2 M files: under 90 seconds for filesystem scanners.
- Registry scan: under 5 seconds.
- Memory ceiling during scan: 300 MB.
- UI responsive throughout — scan runs on worker threads, UI polls progress channel.

### 14.2 Techniques

- Parallel directory walk with work-stealing (`rayon`).
- Two-phase duplicate detection: bucket by size, then BLAKE3 only on collisions.
- Mmap for hashing files >1 MB.
- Bloom filter on visited inodes to defeat cycles.
- Defer ACL queries until needed (don't read ACL on files we won't act on).

---

## 15. UI / UX

### 15.1 Main views

- **Dashboard** — last scan summary, total reclaimable, big "Scan" button.
- **Scan progress** — per-scanner progress, current path, throughput.
- **Findings** — grouped by category, filterable by risk, sortable by size / age.
  - Each finding row: path, size, age, rule ID (clickable → rule explanation), suggested action, checkbox.
- **Quarantine** — held items, restore / purge, retention countdown.
- **Treemap** — visual storage view, click to drill in.
- **Startup audit** — read-only inventory of every persistence vector. Toggle to disable.
- **Rules** — list of loaded rules, source (bundled / user), version, ability to disable individual rules.
- **Settings** — retention period, scanner toggles, dev-cache opt-in, AI classifier toggle, scheduling.

### 15.2 Interaction principles

- Default selections are conservative (low-risk only).
- Any "select all" requires confirming the risk bands included.
- Big destructive buttons are not the primary action color.
- Every action has a 5-second cancel toast after invocation.
- Reports exportable as JSON and human-readable HTML.

---

## 16. Phasing

### Phase 1 — Read-only scanner (MVP)

- Engine library, basic CLI, three scanners: temp, recycle bin, browser caches.
- No deletion. Outputs JSON report + HTML report.
- Ship this alone as a free utility. Builds trust.

### Phase 2 — Quarantine deletion

- Privilege broker.
- Quarantine + undo for filesystem findings.
- UI shell (Tauri).

### Phase 3 — Storage visualization + more scanners

- Treemap.
- Windows Update, crash dumps, installer leftovers, app caches.

### Phase 4 — Registry module

- Read-only scan + `.reg` export.
- Quarantined edits with rollback.

### Phase 5 — Startup / services / tasks audit

- Show, allow disable, never delete.
- WMI subscription enumeration.

### Phase 6 — AI classifier

- Local SLM optional integration.
- Suggestion-only mode.

### Phase 7 — Scheduling, profiles, multi-machine

- Scheduled scans.
- Named profiles (e.g., "weekly light", "monthly deep").
- Optional CLI for power users / automation.

### Phase 8 — Hardening

- Third-party security audit.
- Defender / Kaspersky / ESET whitelist submissions.
- Reproducible build pipeline.

---

## 17. Comparison

| Tool | Reversible | Local-only | Open source | Registry | Startup | Treemap | AI assist |
|------|------------|------------|-------------|----------|---------|---------|-----------|
| CCleaner | partial | no (telemetry) | no | yes | yes | no | no |
| Storage Sense | no | yes | no | no | no | no | no |
| BleachBit | no | yes | yes | partial | no | no | no |
| WinDirStat | n/a | yes | yes | no | no | yes | no |
| **Janitor** | **yes** | **yes** | **yes** | **yes** | **yes** | **yes** | **opt-in** |

---

## 18. Open Questions

- Driver store (`C:\Windows\System32\DriverStore\FileRepository`): legitimately bloats with old driver versions. `pnputil /enum-drivers` + `/delete-driver` is the correct path. Worth a Phase 4+ module?
- Should Janitor offer a "system restore point before scan" option? Trades safety for time. Likely yes, opt-in.
- Bundle a Phase 6 SLM, or require user download? Bundling balloons the installer; downloading needs network on a "local-only" tool. Lean: separate optional installer.
- Multi-user systems: per-user quarantine vs shared? Lean: per-user, with admin able to view all.

---

## 19. Risks

- **AV false positives at launch.** Real risk. Mitigated by signing, transparency, and proactive vendor whitelisting — but the first month will be rough.
- **Bricking risk.** Mitigated by quarantine + blacklist + read-only-first phasing. Cannot be eliminated entirely.
- **Trust cold-start.** Cleaner space is a graveyard of bad actors. Open source from day one is the only credible answer.
- **Maintainer burden.** Rule packs need updating as Windows evolves. Plan: small core team + community-contributed rule PRs with review gating.
- **Microsoft API churn.** WinUI / WinRT instability is why we lean Win32 + Tauri. Reduces but doesn't remove.

---

## 20. Summary

- The opportunity exists because the incumbents are compromised, abandoned, or shallow.
- The technical scope is large but bounded.
- The architecture is conservative on purpose: split privilege, quarantine everything, signed rules, blacklist enforced last.
- Trust is the product. Every architectural choice is downstream of that.
- Phase 1 is shippable in weeks, not months. Each later phase compounds the moat.

---

**Next deliverables (engineering):**

1. Engine crate skeleton + `Scanner` trait + first scanner (`temp_dirs`).
2. Quarantine manifest schema + SQLite migrations.
3. Broker IPC schema (CBOR) + signed-message format.
4. Hard blacklist as a single source-of-truth file, unit-tested.
5. Reference HTML report renderer for Phase 1 read-only ship.
