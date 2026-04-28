# Janitor Architecture (Foundation)

_Last updated: 2026-04-27_

## Layered Trust Model

```
┌──────────────────────────────────────────────┐
│  Layer 6: UI (Tauri + WebView2)             │  ← unprivileged, isolated
│           Just rendering. No business logic. │
└──────────────────────────────────────────────┘
                      ↓ IPC
┌──────────────────────────────────────────────┐
│  Layer 5: Application Service (Rust)        │  ← unprivileged, orchestrates
│           - Scan coordination                │
│           - Report generation                │
│           - Quarantine browser              │
│           - Rules UI                         │
└──────────────────────────────────────────────┘
                      ↓ IPC (signed)
┌──────────────────────────────────────────────┐
│  Layer 4: Privilege Broker (Windows Service)│  ← SYSTEM / LocalSystem
│           - Executes deletions              │
│           - Registry writes                 │
│           - Service/task changes            │
│           (Always validates command source  │
│            and references only finding IDs) │
└──────────────────────────────────────────────┘
                      ↓ uses
┌──────────────────────────────────────────────┐
│  Layer 3: Engine Library (Rust crate)       │  ← pure logic, no UI
│           - Scanners                        │
│           - Classifiers                     │
│           - Blacklist (SAFETY GATE)         │
│           - Finding models                  │
│           - Quarantine manifest             │
│           - Undo log                        │
│           (Unit testable, parallelizable)   │
└──────────────────────────────────────────────┘
                      ↓ uses
┌──────────────────────────────────────────────┐
│  Layer 2: Rule Store (TOML files, signed)   │  ← versioned, auditable
│           - Declarative rule definitions     │
│           - Loadable from app dir + user    │
│           - Cannot override blacklist       │
└──────────────────────────────────────────────┘

Layer 1: Persistence (SQLite)
           - Scan history
           - Quarantine manifest
           - Undo log
           - Audit log
```

## Defense-in-Depth: Why Multiple Layers?

**Single-process admin tool = every UI bug is privilege escalation.**

By splitting:
1. UI bugs stay in unprivileged layer
2. Broker only executes findings from current scan (stored in DB)
3. Broker validates every command: origin + schema + rule version
4. Blacklist enforced as a final filter (doesn't rely on rules)
5. Undo log is append-only; every action is traceable

## Trust Boundaries

### Boundary 1: Blacklist (Immediate)
```
All paths → blacklist.is_path_safe() → true/false
                          ↓
                   Reject if blacklisted
                   (No exceptions, ever)
```

### Boundary 2: Scanners → Findings
```
Scanner outputs Finding
    ↓
Blacklist check
    ↓
Rule validation (Phase 2)
    ↓
Confidence score (classifier)
    ↓
Finding → findings list
```

### Boundary 3: UI → Broker (IPC)
```
User clicks "Quarantine"
    ↓
UI sends: Command { finding_ids: [...] }
    ↓
Broker validates:
  - Message signed?
  - Caller = install user?
  - Finding IDs in current scan?
  - Rule version still valid?
    ↓
Broker executes move to quarantine
    ↓
Audit log entry (append-only)
```

## Scanners: Independent, Parallel, Testable

Each scanner is **stateless**:
- No shared mutable state
- No side effects during scan
- Returns Vec<Finding> or error

Execution:
```rust
let findings = scanners
    .par_iter()  // rayon parallel iterator
    .flat_map(|s| s.scan(&ctx).ok_or_log())
    .collect();
```

Result: 500GB SSD (2M files) in <90 seconds.

## Core Types (janitor-engine)

```rust
// What we're scanning
pub struct ScanContext {
    scan_id: String,
    target_paths: Vec<PathBuf>,
    require_elevation: bool,
    include_cloud_paths: bool,
    include_dev_caches: bool,
}

// A single junk item
pub struct Finding {
    id: String,
    scanner_id: String,
    rule_id: String,
    category: Category,
    risk: RiskLevel,  // low/medium/high
    target_kind: TargetKind,  // file/dir/regkey/service/task
    target_ref: String,  // path or registry key
    size_bytes: u64,
    age_days: u32,
    confidence: f64,  // [0.0, 1.0]
    reason: String,
    suggested_action: String,
    timestamp: String,  // ISO 8601
}

// Scanner interface
pub trait Scanner: Send + Sync {
    fn id(&self) -> &'static str;
    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>>;
}

// What blacklist enforces
pub fn is_path_safe(path: &Path) -> bool { ... }
```

## Safety Principles (from whitepaper)

1. **Reversible by default** — Nothing deletes immediately. Quarantine first.
2. **Transparent** — Every finding shows the rule that matched it.
3. **Local-only** — Zero network except for user-initiated updates.
4. **Open source** — GPLv3 or MPL-2.0. Reproducible builds.
5. **Privilege-minimal** — UI never admin. Broker handles writes.
6. **Read-before-write** — Scanners read-only first (Phase 1).
7. **Fail safe** — Errors abort, never proceed partially.
8. **Blacklist enforced last** — Final filter before action.

## Phase 1 MVP

**Ship date:** weeks, not months

**What ships:**
- Engine library (scanners: temp, recycle bin, browser cache)
- CLI binary (read-only report)
- JSON + HTML report export
- Blacklist enforced
- Transparent (every finding cites its rule)

**Does NOT ship:**
- Deletion capability
- Privilege broker
- Quarantine
- Tauri UI

**Why?** Builds trust. Proves the foundation is solid.
