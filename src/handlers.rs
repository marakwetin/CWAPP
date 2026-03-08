use actix_web::{delete, get, patch, post, web, HttpResponse};
use chrono::Local;
use serde::Deserialize;
use tera::{Context, Tera};

use crate::db::{
    create_job as db_create_job, daily_stats, delete_job as db_delete_job, get_job as db_get_job,
    list_all_jobs, list_jobs_by_date, staff_earnings, update_status as db_update_status, DbPool,
    NewJob,
};
use crate::errors::AppError;

const STAFF: [&str; 5] = ["James", "Mary", "Brian", "Alice", "Kevin"];
const VEHICLES: [(&str, i64); 4] = [
    ("Saloon", 500),
    ("Sedan", 500),
    ("SUV", 800),
    ("Lorry", 1500),
];

#[derive(Debug, Deserialize)]
pub struct JobsQuery {
    pub date: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct DateQuery {
    pub date: Option<String>,
}

fn today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn validate_new_job(payload: &NewJob) -> Result<(), AppError> {
    if !VEHICLES
        .iter()
        .any(|(v, _)| v.eq_ignore_ascii_case(&payload.vehicle_type))
    {
        return Err(AppError::BadRequest("Invalid vehicle type".into()));
    }
    if payload.plate.trim().is_empty() {
        return Err(AppError::BadRequest("Number plate is required".into()));
    }
    if !STAFF.iter().any(|s| s.eq_ignore_ascii_case(&payload.staff)) {
        return Err(AppError::BadRequest("Invalid staff member".into()));
    }
    if payload.amount <= 0 {
        return Err(AppError::BadRequest(
            "Amount must be greater than zero".into(),
        ));
    }
    Ok(())
}

#[get("/")]
pub async fn index(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
) -> Result<HttpResponse, AppError> {
    let date = today_date();
    let jobs = list_jobs_by_date(&pool, &date, None)?;
    let stats = daily_stats(&pool, &date)?;
    let earnings = staff_earnings(&pool, &date)?;

    let mut context = Context::new();
    context.insert("jobs", &jobs);
    context.insert("stats", &stats);
    context.insert("earnings", &earnings);
    context.insert("staff", &STAFF);
    context.insert("vehicles", &VEHICLES);
    context.insert("today", &date);

    let html = tmpl.render("index.html", &context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[post("/api/jobs")]
pub async fn create_job(
    pool: web::Data<DbPool>,
    payload: web::Json<NewJob>,
) -> Result<HttpResponse, AppError> {
    validate_new_job(&payload)?;
    let job = db_create_job(&pool, payload.into_inner())?;
    Ok(HttpResponse::Created().json(job))
}

#[get("/api/jobs")]
pub async fn list_jobs(
    pool: web::Data<DbPool>,
    query: web::Query<JobsQuery>,
) -> Result<HttpResponse, AppError> {
    let jobs = match &query.date {
        Some(date) => list_jobs_by_date(&pool, date, query.search.as_deref())?,
        None => list_all_jobs(&pool, query.search.as_deref())?,
    };
    Ok(HttpResponse::Ok().json(jobs))
}

#[get("/api/jobs/{id}")]
pub async fn get_job(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let job = db_get_job(&pool, id)?.ok_or(AppError::NotFound("Job not found".into()))?;
    Ok(HttpResponse::Ok().json(job))
}

#[patch("/api/jobs/{id}/status")]
pub async fn update_status(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
    payload: web::Json<StatusPayload>,
) -> Result<HttpResponse, AppError> {
    if payload.status != "washing" && payload.status != "done" {
        return Err(AppError::BadRequest(
            "Status must be either 'washing' or 'done'".into(),
        ));
    }

    let id = path.into_inner();
    let updated = db_update_status(&pool, id, &payload.status)?;
    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/api/jobs/{id}")]
pub async fn delete_job(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    db_delete_job(&pool, path.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/api/stats")]
pub async fn get_stats(
    pool: web::Data<DbPool>,
    query: web::Query<DateQuery>,
) -> Result<HttpResponse, AppError> {
    let date = query.date.clone().unwrap_or_else(today_date);
    let stats = daily_stats(&pool, &date)?;
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/api/earnings")]
pub async fn get_earnings(
    pool: web::Data<DbPool>,
    query: web::Query<DateQuery>,
) -> Result<HttpResponse, AppError> {
    let date = query.date.clone().unwrap_or_else(today_date);
    let earnings = staff_earnings(&pool, &date)?;
    Ok(HttpResponse::Ok().json(earnings))
}
