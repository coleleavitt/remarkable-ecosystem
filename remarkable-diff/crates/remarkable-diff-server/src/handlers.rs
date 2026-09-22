//! API handlers

use axum::{
    extract::{Path, State, Json, Multipart},
};
use remarkable_diff_core::{
    self as core,
    DocumentVersion, DocumentDiff, MergeResult, Timeline,
    MergeStrategy, ConflictResolution,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state::AppState;
use crate::error::{AppError, Result};
use crate::parsers;

type SharedState = Arc<RwLock<AppState>>;

// ============ Document Handlers ============

#[derive(Serialize)]
pub struct DocumentListItem {
    id: Uuid,
    name: String,
    version_count: usize,
}

pub async fn list_documents(
    State(state): State<SharedState>,
) -> Json<Vec<DocumentListItem>> {
    let state = state.read().await;
    let docs: Vec<_> = state.documents.values()
        .map(|d| DocumentListItem {
            id: d.id,
            name: d.name.clone(),
            version_count: d.versions.len(),
        })
        .collect();
    Json(docs)
}

pub async fn upload_document(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<DocumentListItem>> {
    let mut name = String::new();
    let mut rm_files: Vec<(Uuid, Vec<u8>)> = Vec::new();
    let mut content_data: Option<Vec<u8>> = None;
    
    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::ParseError(e.to_string()))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "name" => {
                name = field.text().await
                    .map_err(|e| AppError::ParseError(e.to_string()))?;
            }
            "content" => {
                content_data = Some(field.bytes().await
                    .map_err(|e| AppError::ParseError(e.to_string()))?
                    .to_vec());
            }
            _ if field_name.ends_with(".rm") => {
                let page_id = Uuid::new_v4();
                let data = field.bytes().await
                    .map_err(|e| AppError::ParseError(e.to_string()))?
                    .to_vec();
                rm_files.push((page_id, data));
            }
            _ => {}
        }
    }
    
    if name.is_empty() {
        name = "Untitled".to_string();
    }
    
    let version = parsers::create_version_from_upload(
        name.clone(),
        rm_files,
        content_data,
        1, // Default author ID
    )?;
    
    let mut state = state.write().await;
    let doc_id = state.add_document(name.clone(), version);
    let version_count = state.documents.get(&doc_id)
        .map(|d| d.versions.len())
        .unwrap_or(0);
    
    Ok(Json(DocumentListItem {
        id: doc_id,
        name,
        version_count,
    }))
}

pub async fn get_document(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentListItem>> {
    let state = state.read().await;
    let doc = state.get_document(&id)
        .ok_or_else(|| AppError::DocumentNotFound(id.to_string()))?;
    
    Ok(Json(DocumentListItem {
        id: doc.id,
        name: doc.name.clone(),
        version_count: doc.versions.len(),
    }))
}

#[derive(Serialize)]
pub struct VersionListItem {
    id: Uuid,
    name: String,
    timestamp: String,
    author_id: u64,
    page_count: usize,
    operation_count: usize,
}

pub async fn list_versions(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VersionListItem>>> {
    let state = state.read().await;
    let doc = state.get_document(&id)
        .ok_or_else(|| AppError::DocumentNotFound(id.to_string()))?;
    
    let versions: Vec<_> = doc.versions.iter()
        .map(|v| VersionListItem {
            id: v.id,
            name: v.name.clone(),
            timestamp: v.timestamp.to_rfc3339(),
            author_id: v.author_id,
            page_count: v.pages.len(),
            operation_count: v.operations.len(),
        })
        .collect();
    
    Ok(Json(versions))
}

pub async fn get_version(
    State(state): State<SharedState>,
    Path((doc_id, version_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DocumentVersion>> {
    let state = state.read().await;
    let version = state.get_version(&doc_id, &version_id)
        .ok_or_else(|| AppError::VersionNotFound(version_id.to_string()))?;
    
    Ok(Json(version.clone()))
}

// ============ Diff Handlers ============

#[derive(Deserialize)]
pub struct ComputeDiffRequest {
    version_a_id: Uuid,
    version_b_id: Uuid,
    document_id: Uuid,
}

#[derive(Serialize)]
pub struct DiffResponse {
    id: Uuid,
    diff: DocumentDiff,
}

pub async fn compute_diff(
    State(state): State<SharedState>,
    Json(req): Json<ComputeDiffRequest>,
) -> Result<Json<DiffResponse>> {
    let state_read = state.read().await;
    
    let version_a = state_read.get_version(&req.document_id, &req.version_a_id)
        .ok_or_else(|| AppError::VersionNotFound(req.version_a_id.to_string()))?;
    let version_b = state_read.get_version(&req.document_id, &req.version_b_id)
        .ok_or_else(|| AppError::VersionNotFound(req.version_b_id.to_string()))?;
    
    let diff = core::compute_diff(version_a, version_b);
    let diff_id = Uuid::new_v4();
    
    drop(state_read);
    
    let mut state_write = state.write().await;
    state_write.diffs.insert(diff_id, diff.clone());
    
    Ok(Json(DiffResponse { id: diff_id, diff }))
}

pub async fn get_diff(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentDiff>> {
    let state = state.read().await;
    let diff = state.diffs.get(&id)
        .ok_or_else(|| AppError::DocumentNotFound(id.to_string()))?;
    
    Ok(Json(diff.clone()))
}

// ============ Merge Handlers ============

#[derive(Deserialize)]
pub struct MergeRequest {
    version_a_id: Uuid,
    version_b_id: Uuid,
    document_id: Uuid,
    #[serde(default)]
    strategy: MergeStrategy,
}

#[derive(Serialize)]
pub struct MergeResponse {
    id: Uuid,
    result: MergeResult,
}

pub async fn merge_versions(
    State(state): State<SharedState>,
    Json(req): Json<MergeRequest>,
) -> Result<Json<MergeResponse>> {
    let state_read = state.read().await;
    
    let version_a = state_read.get_version(&req.document_id, &req.version_a_id)
        .ok_or_else(|| AppError::VersionNotFound(req.version_a_id.to_string()))?;
    let version_b = state_read.get_version(&req.document_id, &req.version_b_id)
        .ok_or_else(|| AppError::VersionNotFound(req.version_b_id.to_string()))?;
    
    let result = core::merge_documents(version_a, version_b, req.strategy, &HashMap::new());
    let merge_id = Uuid::new_v4();
    
    drop(state_read);
    
    let mut state_write = state.write().await;
    state_write.merges.insert(merge_id, result.clone());
    
    Ok(Json(MergeResponse { id: merge_id, result }))
}

#[derive(Deserialize)]
pub struct ResolveConflictRequest {
    conflict_id: Uuid,
    resolution: ConflictResolution,
}

pub async fn resolve_conflict(
    State(state): State<SharedState>,
    Path(merge_id): Path<Uuid>,
    Json(req): Json<ResolveConflictRequest>,
) -> Result<Json<MergeResult>> {
    let mut state = state.write().await;
    
    let result = state.merges.get_mut(&merge_id)
        .ok_or_else(|| AppError::DocumentNotFound(merge_id.to_string()))?;
    
    core::apply_resolution(result, req.conflict_id, req.resolution)
        .map_err(|e| AppError::MergeError(e))?;
    
    Ok(Json(result.clone()))
}

pub async fn export_merged(
    State(state): State<SharedState>,
    Path(merge_id): Path<Uuid>,
) -> Result<Json<DocumentVersion>> {
    let state = state.read().await;
    
    let result = state.merges.get(&merge_id)
        .ok_or_else(|| AppError::DocumentNotFound(merge_id.to_string()))?;
    
    Ok(Json(result.merged.clone()))
}

// ============ Timeline Handlers ============

pub async fn get_timeline(
    State(state): State<SharedState>,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<Timeline>> {
    let mut state = state.write().await;
    
    // Check cache
    if let Some(timeline) = state.timelines.get(&doc_id) {
        return Ok(Json(timeline.clone()));
    }
    
    // Build timeline from all versions
    let doc = state.documents.get(&doc_id)
        .ok_or_else(|| AppError::DocumentNotFound(doc_id.to_string()))?;
    
    let all_ops: Vec<_> = doc.versions.iter()
        .flat_map(|v| v.operations.iter().cloned())
        .collect();
    
    let timeline = core::build_timeline(&all_ops);
    state.timelines.insert(doc_id, timeline.clone());
    
    Ok(Json(timeline))
}

#[derive(Deserialize)]
pub struct TimelineFilterRequest {
    #[serde(default)]
    author_id: Option<u64>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    start_counter: Option<u64>,
    #[serde(default)]
    end_counter: Option<u64>,
}

pub async fn filter_timeline(
    State(state): State<SharedState>,
    Path(doc_id): Path<Uuid>,
    Json(req): Json<TimelineFilterRequest>,
) -> Result<Json<Timeline>> {
    let state = state.read().await;
    
    let timeline = state.timelines.get(&doc_id)
        .ok_or_else(|| AppError::DocumentNotFound(doc_id.to_string()))?;
    
    let mut filtered = timeline.clone();
    
    if let Some(author_id) = req.author_id {
        filtered = core::filter_by_author(&filtered, author_id);
    }
    
    if let Some(category) = &req.category {
        filtered = core::filter_by_category(&filtered, category);
    }
    
    if let (Some(start), Some(end)) = (req.start_counter, req.end_counter) {
        let start_ts = core::CrdtTimestamp::new(0, start);
        let end_ts = core::CrdtTimestamp::new(0, end);
        filtered = core::filter_by_range(&filtered, start_ts, end_ts);
    }
    
    Ok(Json(filtered))
}

// ============ Render Handlers ============

#[derive(Serialize)]
pub struct StrokeRenderData {
    svg_path: String,
    color: String,
    stroke_width: f32,
}

pub async fn render_stroke(
    State(_state): State<SharedState>,
    Path(_stroke_id): Path<String>,
) -> Result<Json<StrokeRenderData>> {
    // Placeholder - would render stroke to SVG path
    Ok(Json(StrokeRenderData {
        svg_path: "M 0 0 L 100 100".to_string(),
        color: "#000000".to_string(),
        stroke_width: 2.0,
    }))
}

#[derive(Serialize)]
pub struct PageRenderData {
    svg: String,
    width: f32,
    height: f32,
}

pub async fn render_page(
    State(_state): State<SharedState>,
    Path(_page_id): Path<Uuid>,
) -> Result<Json<PageRenderData>> {
    // Placeholder - would render full page to SVG
    Ok(Json(PageRenderData {
        svg: r#"<svg xmlns="http://www.w3.org/2000/svg" width="1404" height="1872"></svg>"#.to_string(),
        width: 1404.0,
        height: 1872.0,
    }))
}
