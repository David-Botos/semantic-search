// src/main.rs
mod db;
mod embedding;

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
// Removed unused import: std::sync::Arc

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
    limit: Option<i32>,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    name: String,
    description: Option<String>,
    short_description: Option<String>,
    status: String,
    organization_name: Option<String>,
    similarity: f64,
}

// Initialize with model and DB connection
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Semantic Search API is running")
}

#[post("/search")]
async fn search(
    query: web::Json<SearchQuery>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(10);
    
    info!("Search request received: '{}', limit: {}", query.query, limit);
    
    // Generate embedding for the query
    let embedding_result = embedding::generate_embedding(&query.query, &app_data.embedding_model).await;
    
    let query_embedding = match embedding_result {
        Ok(embedding) => embedding,
        Err(e) => {
            error!("Failed to generate embedding: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate embedding"
            }));
        }
    };
    
    // Perform vector search in database
    match search_services(&app_data.db_pool, &query_embedding, limit).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            error!("Search failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Search failed"
            }))
        }
    }
}

async fn search_services(
    pool: &db::PgPool,
    query_embedding: &[f32],
    limit: i32,
) -> Result<Vec<SearchResult>> {
    let client = pool.get().await?;
    
    // Convert Vec<f32> to pgvector::Vector
    let query_vector = pgvector::Vector::from(query_embedding.to_vec());
    
    // Convert i32 limit to i64 for PostgreSQL compatibility
    let limit_i64: i64 = limit.into();
    
    // Query using cosine similarity
    let rows = client
        .query(
            "SELECT 
                s.id, 
                s.name, 
                s.description, 
                s.short_description,
                s.status,
                o.name as organization_name,
                (1 - (s.embedding <=> $1))::float8 as similarity
            FROM 
                service s
            JOIN
                organization o ON s.organization_id = o.id
            WHERE
                s.embedding IS NOT NULL
            ORDER BY 
                s.embedding <=> $1
            LIMIT $2",
            &[&query_vector, &limit_i64],
        )
        .await?;
    
    let results = rows
        .iter()
        .map(|row| SearchResult {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            short_description: row.get("short_description"),
            status: row.get("status"),
            organization_name: row.get("organization_name"),
            similarity: row.get("similarity"),
        })
        .collect();
    
    Ok(results)
}

// Application state
struct AppState {
    db_pool: db::PgPool,
    embedding_model: embedding::EmbeddingModel,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    info!("Starting Semantic Search Microservice");
    
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();
    
    // Initialize database connection pool
    let db_pool = match db::connect().await {
        Ok(pool) => {
            info!("Database connection pool initialized successfully");
            pool
        },
        Err(e) => {
            error!("Failed to initialize database connection pool: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database connection failed: {}", e),
            ));
        }
    };
    
    // Initialize embedding model
    let embedding_model = match embedding::init_model() {
        Ok(model) => {
            info!("Embedding model initialized successfully");
            model
        },
        Err(e) => {
            error!("Failed to initialize embedding model: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Model initialization failed: {}", e),
            ));
        }
    };
    
    // Read server configuration from environment variables
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    // Create shared application state
    let app_state = web::Data::new(AppState {
        db_pool,
        embedding_model,
    });
    
    info!("Starting HTTP server on {}:{}", host, port);
    
    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS for the Next.js frontend
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(index)
            .service(search)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}