# CWAPP — Car Wash Business Management App

A full-stack dark-themed car wash management system built with **Rust (Actix-Web 4)**, **SQLite**, **Tera templates**, and **vanilla JavaScript fetch()** updates.

## Features

- Commission rule: staff earns **30%** per wash (stored at job creation time).
- Commission only counts toward payout when job status is `done`.
- One-way workflow: `queued → washing → done`.
- Daily stats bar:
  - Today's Jobs
  - In Progress
  - Completed
  - Revenue (KES)
  - Total Staff Pay (KES)
- New wash form with:
  - Vehicle selector and suggested pricing
  - Auto-uppercase number plate
  - Live commission preview
- Wash records table with:
  - Filter chips and search
  - Commission pending/done display modes
  - Advance status and delete actions
- Daily staff earnings panel with progress bars and payout total.
- Receipt modal shown on completion, with print button.
- Toast notifications for all actions.

## Vehicle Types and Suggested Prices (KES)

- Saloon — 500
- Sedan — 500
- SUV — 800
- Lorry — 1500

## Staff Roster

- James
- Mary
- Brian
- Alice
- Kevin

---

## Project Structure

```text
src/
  main.rs
  db.rs
  handlers.rs
  errors.rs
  filters.rs
templates/
  index.html
.env.example
README.md
```

---

## Setup

### 1) Prerequisites

- Rust stable toolchain (`rustup`)
- Cargo

### 2) Configure environment

```bash
cp .env.example .env
```

Edit `.env` as needed:

```env
HOST=127.0.0.1
PORT=8080
DATABASE_URL=cwapp.db
RUST_LOG=info
```

### 3) Run

```bash
cargo run
```

Open: `http://127.0.0.1:8080`

On startup, SQLite migration runs automatically and creates table/indexes if needed.

---

## API Reference

| Method | Endpoint | Description | Body |
|---|---|---|---|
| GET | `/` | Render dashboard page | - |
| POST | `/api/jobs` | Create a wash job | `{ "vehicle_type", "plate", "staff", "amount" }` |
| GET | `/api/jobs` | List jobs (`?date=YYYY-MM-DD&search=term` optional) | - |
| GET | `/api/jobs/{id}` | Get one job by id | - |
| PATCH | `/api/jobs/{id}/status` | Move status forward (`washing` or `done`) | `{ "status": "washing" | "done" }` |
| DELETE | `/api/jobs/{id}` | Delete a job | - |
| GET | `/api/stats` | Daily stats (`?date=YYYY-MM-DD` optional) | - |
| GET | `/api/earnings` | Staff earnings (`?date=YYYY-MM-DD` optional) | - |

---

## Business Rule Customization

### Change commission rate

Commission is currently calculated in `src/db.rs` inside `create_job`:

```rust
let commission = payload.amount * 30 / 100;
```

To change to 25%, update to:

```rust
let commission = payload.amount * 25 / 100;
```

### Add or remove staff members

Staff validation and frontend dropdown are driven by `STAFF` constant in `src/handlers.rs`.

Update:

```rust
const STAFF: [&str; 5] = ["James", "Mary", "Brian", "Alice", "Kevin"];
```

to include your new roster.

---

## Notes

- SQLite runs in WAL mode for better concurrency.
- Status transition validation prevents backward or skipped transitions.
- Only `done` jobs contribute to revenue and payout aggregates.
