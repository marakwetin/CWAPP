use chrono::Local;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

pub type DbPool = Pool<SqliteConnectionManager>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WashJob {
    pub id: i64,
    pub vehicle_type: String,
    pub plate: String,
    pub staff: String,
    pub amount: i64,
    pub commission: i64,
    pub status: String,
    pub time_in: String,
    pub time_done: Option<String>,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffEarnings {
    pub staff: String,
    pub job_count: i64,
    pub total_wash_value: i64,
    pub commission_total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub todays_jobs: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub revenue: i64,
    pub total_staff_pay: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJob {
    pub vehicle_type: String,
    pub plate: String,
    pub staff: String,
    pub amount: i64,
}

pub fn build_pool(database_url: &str) -> Result<DbPool, AppError> {
    let manager = SqliteConnectionManager::file(database_url);
    let pool = Pool::new(manager).map_err(AppError::Pool)?;
    let conn = pool.get().map_err(AppError::Pool)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS wash_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vehicle_type TEXT NOT NULL,
            plate TEXT NOT NULL,
            staff TEXT NOT NULL,
            amount INTEGER NOT NULL,
            commission INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'queued',
            time_in TEXT NOT NULL,
            time_done TEXT,
            date TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_date ON wash_jobs (date);
        CREATE INDEX IF NOT EXISTS idx_status ON wash_jobs (status);
        CREATE INDEX IF NOT EXISTS idx_staff ON wash_jobs (staff);",
    )
    .map_err(AppError::Db)?;

    Ok(pool)
}

pub fn create_job(pool: &DbPool, payload: NewJob) -> Result<WashJob, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let commission = payload.amount * 30 / 100;
    let now = Local::now();
    let time_in = now.to_rfc3339();
    let date = now.format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT INTO wash_jobs (vehicle_type, plate, staff, amount, commission, status, time_in, date)
         VALUES (?1, ?2, ?3, ?4, ?5, 'queued', ?6, ?7)",
        params![
            payload.vehicle_type,
            payload.plate.to_uppercase(),
            payload.staff,
            payload.amount,
            commission,
            time_in,
            date
        ],
    )
    .map_err(AppError::Db)?;

    let id = conn.last_insert_rowid();
    get_job(pool, id)?.ok_or(AppError::NotFound("Job not found after insert".into()))
}

pub fn list_jobs_by_date(
    pool: &DbPool,
    date: &str,
    search: Option<&str>,
) -> Result<Vec<WashJob>, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let mut jobs = Vec::new();

    match search {
        Some(term) if !term.trim().is_empty() => {
            let like = format!("%{}%", term.trim().to_lowercase());
            let mut stmt = conn
                .prepare(
                    "SELECT id, vehicle_type, plate, staff, amount, commission, status, time_in, time_done, date
                     FROM wash_jobs
                     WHERE date = ?1
                       AND (LOWER(plate) LIKE ?2 OR LOWER(staff) LIKE ?2)
                     ORDER BY id DESC",
                )
                .map_err(AppError::Db)?;
            let rows = stmt
                .query_map(params![date, like], |row| {
                    Ok(WashJob {
                        id: row.get(0)?,
                        vehicle_type: row.get(1)?,
                        plate: row.get(2)?,
                        staff: row.get(3)?,
                        amount: row.get(4)?,
                        commission: row.get(5)?,
                        status: row.get(6)?,
                        time_in: row.get(7)?,
                        time_done: row.get(8)?,
                        date: row.get(9)?,
                    })
                })
                .map_err(AppError::Db)?;

            for row in rows {
                jobs.push(row.map_err(AppError::Db)?);
            }
        }
        _ => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, vehicle_type, plate, staff, amount, commission, status, time_in, time_done, date
                     FROM wash_jobs
                     WHERE date = ?1
                     ORDER BY id DESC",
                )
                .map_err(AppError::Db)?;
            let rows = stmt
                .query_map(params![date], |row| {
                    Ok(WashJob {
                        id: row.get(0)?,
                        vehicle_type: row.get(1)?,
                        plate: row.get(2)?,
                        staff: row.get(3)?,
                        amount: row.get(4)?,
                        commission: row.get(5)?,
                        status: row.get(6)?,
                        time_in: row.get(7)?,
                        time_done: row.get(8)?,
                        date: row.get(9)?,
                    })
                })
                .map_err(AppError::Db)?;

            for row in rows {
                jobs.push(row.map_err(AppError::Db)?);
            }
        }
    }

    Ok(jobs)
}

pub fn list_all_jobs(pool: &DbPool, search: Option<&str>) -> Result<Vec<WashJob>, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let mut jobs = Vec::new();

    match search {
        Some(term) if !term.trim().is_empty() => {
            let like = format!("%{}%", term.trim().to_lowercase());
            let mut stmt = conn
                .prepare(
                    "SELECT id, vehicle_type, plate, staff, amount, commission, status, time_in, time_done, date
                     FROM wash_jobs
                     WHERE LOWER(plate) LIKE ?1 OR LOWER(staff) LIKE ?1
                     ORDER BY id DESC",
                )
                .map_err(AppError::Db)?;
            let rows = stmt
                .query_map(params![like], |row| {
                    Ok(WashJob {
                        id: row.get(0)?,
                        vehicle_type: row.get(1)?,
                        plate: row.get(2)?,
                        staff: row.get(3)?,
                        amount: row.get(4)?,
                        commission: row.get(5)?,
                        status: row.get(6)?,
                        time_in: row.get(7)?,
                        time_done: row.get(8)?,
                        date: row.get(9)?,
                    })
                })
                .map_err(AppError::Db)?;

            for row in rows {
                jobs.push(row.map_err(AppError::Db)?);
            }
        }
        _ => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, vehicle_type, plate, staff, amount, commission, status, time_in, time_done, date
                     FROM wash_jobs
                     ORDER BY id DESC",
                )
                .map_err(AppError::Db)?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(WashJob {
                        id: row.get(0)?,
                        vehicle_type: row.get(1)?,
                        plate: row.get(2)?,
                        staff: row.get(3)?,
                        amount: row.get(4)?,
                        commission: row.get(5)?,
                        status: row.get(6)?,
                        time_in: row.get(7)?,
                        time_done: row.get(8)?,
                        date: row.get(9)?,
                    })
                })
                .map_err(AppError::Db)?;

            for row in rows {
                jobs.push(row.map_err(AppError::Db)?);
            }
        }
    }

    Ok(jobs)
}

pub fn update_status(pool: &DbPool, id: i64, status: &str) -> Result<WashJob, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let job = get_job(pool, id)?.ok_or(AppError::NotFound("Job not found".into()))?;

    if (job.status == "queued" && status == "washing")
        || (job.status == "washing" && status == "done")
    {
        if status == "done" {
            conn.execute(
                "UPDATE wash_jobs SET status = ?1, time_done = ?2 WHERE id = ?3",
                params![status, Local::now().to_rfc3339(), id],
            )
            .map_err(AppError::Db)?;
        } else {
            conn.execute(
                "UPDATE wash_jobs SET status = ?1 WHERE id = ?2",
                params![status, id],
            )
            .map_err(AppError::Db)?;
        }

        get_job(pool, id)?.ok_or(AppError::NotFound("Job not found after update".into()))
    } else {
        Err(AppError::BadRequest(format!(
            "Invalid status transition from '{}' to '{}'",
            job.status, status
        )))
    }
}

pub fn delete_job(pool: &DbPool, id: i64) -> Result<(), AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let affected = conn
        .execute("DELETE FROM wash_jobs WHERE id = ?1", params![id])
        .map_err(AppError::Db)?;

    if affected == 0 {
        return Err(AppError::NotFound("Job not found".into()));
    }

    Ok(())
}

pub fn get_job(pool: &DbPool, id: i64) -> Result<Option<WashJob>, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, vehicle_type, plate, staff, amount, commission, status, time_in, time_done, date
             FROM wash_jobs
             WHERE id = ?1",
        )
        .map_err(AppError::Db)?;

    let job = stmt
        .query_row(params![id], |row| {
            Ok(WashJob {
                id: row.get(0)?,
                vehicle_type: row.get(1)?,
                plate: row.get(2)?,
                staff: row.get(3)?,
                amount: row.get(4)?,
                commission: row.get(5)?,
                status: row.get(6)?,
                time_in: row.get(7)?,
                time_done: row.get(8)?,
                date: row.get(9)?,
            })
        })
        .optional()
        .map_err(AppError::Db)?;

    Ok(job)
}

pub fn staff_earnings(pool: &DbPool, date: &str) -> Result<Vec<StaffEarnings>, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let mut stmt = conn
        .prepare(
            "SELECT staff,
                    COUNT(*) as job_count,
                    SUM(amount) as total_wash_value,
                    SUM(commission) as commission_total
             FROM wash_jobs
             WHERE date = ?1 AND status = 'done'
             GROUP BY staff
             ORDER BY commission_total DESC",
        )
        .map_err(AppError::Db)?;

    let rows = stmt
        .query_map(params![date], |row| {
            Ok(StaffEarnings {
                staff: row.get(0)?,
                job_count: row.get(1)?,
                total_wash_value: row.get(2)?,
                commission_total: row.get(3)?,
            })
        })
        .map_err(AppError::Db)?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(AppError::Db)?);
    }

    Ok(out)
}

pub fn daily_stats(pool: &DbPool, date: &str) -> Result<DailyStats, AppError> {
    let conn = pool.get().map_err(AppError::Pool)?;
    let mut stmt = conn
        .prepare(
            "SELECT
                COUNT(*) as todays_jobs,
                SUM(CASE WHEN status = 'washing' THEN 1 ELSE 0 END) as in_progress,
                SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as completed,
                SUM(CASE WHEN status = 'done' THEN amount ELSE 0 END) as revenue,
                SUM(CASE WHEN status = 'done' THEN commission ELSE 0 END) as total_staff_pay
            FROM wash_jobs
            WHERE date = ?1",
        )
        .map_err(AppError::Db)?;

    let stats = stmt
        .query_row(params![date], |row| {
            Ok(DailyStats {
                todays_jobs: row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                in_progress: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                completed: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                revenue: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                total_staff_pay: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            })
        })
        .map_err(AppError::Db)?;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TestCtx {
        pool: DbPool,
        path: std::path::PathBuf,
    }

    impl Drop for TestCtx {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn test_ctx() -> TestCtx {
        let path = std::env::temp_dir().join(format!("cwapp-test-{}.db", uuid::Uuid::new_v4()));
        let pool = build_pool(path.to_str().expect("temp path should be valid"))
            .expect("pool should build");
        TestCtx { pool, path }
    }

    #[test]
    fn create_job_persists_commission_at_30_percent() {
        let ctx = test_ctx();
        let created = create_job(
            &ctx.pool,
            NewJob {
                vehicle_type: "SUV".into(),
                plate: "kda123x".into(),
                staff: "James".into(),
                amount: 800,
            },
        )
        .expect("job should be created");

        assert_eq!(created.commission, 240);
        assert_eq!(created.status, "queued");
        assert_eq!(created.plate, "KDA123X");
    }

    #[test]
    fn status_workflow_is_one_direction_only() {
        let ctx = test_ctx();
        let created = create_job(
            &ctx.pool,
            NewJob {
                vehicle_type: "Saloon".into(),
                plate: "KAA999A".into(),
                staff: "Mary".into(),
                amount: 500,
            },
        )
        .expect("job should be created");

        let invalid =
            update_status(&ctx.pool, created.id, "done").expect_err("queued -> done is invalid");
        assert!(matches!(invalid, AppError::BadRequest(_)));

        let washing =
            update_status(&ctx.pool, created.id, "washing").expect("queued -> washing should work");
        assert_eq!(washing.status, "washing");

        let done =
            update_status(&ctx.pool, created.id, "done").expect("washing -> done should work");
        assert_eq!(done.status, "done");
        assert!(done.time_done.is_some());
    }

    #[test]
    fn aggregates_only_count_done_jobs() {
        let ctx = test_ctx();

        let done_job = create_job(
            &ctx.pool,
            NewJob {
                vehicle_type: "SUV".into(),
                plate: "KCC111C".into(),
                staff: "Brian".into(),
                amount: 800,
            },
        )
        .expect("job should be created");

        let _queued_job = create_job(
            &ctx.pool,
            NewJob {
                vehicle_type: "Sedan".into(),
                plate: "KDD222D".into(),
                staff: "Brian".into(),
                amount: 500,
            },
        )
        .expect("job should be created");

        update_status(&ctx.pool, done_job.id, "washing").expect("should advance to washing");
        update_status(&ctx.pool, done_job.id, "done").expect("should advance to done");

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let stats = daily_stats(&ctx.pool, &today).expect("stats should load");
        assert_eq!(stats.todays_jobs, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.revenue, 800);
        assert_eq!(stats.total_staff_pay, 240);

        let earnings = staff_earnings(&ctx.pool, &today).expect("earnings should load");
        assert_eq!(earnings.len(), 1);
        assert_eq!(earnings[0].staff, "Brian");
        assert_eq!(earnings[0].job_count, 1);
        assert_eq!(earnings[0].total_wash_value, 800);
        assert_eq!(earnings[0].commission_total, 240);
    }
}
