pub mod pb {
    tonic::include_proto!("cathedral.v1");
}

use pb::cathedral_bridge_server::{CathedralBridge, CathedralBridgeServer};
use pb::{
    IngestRequest, IngestResponse,
    GovernanceRequest, GovernanceResponse, GovernanceVerdict,
    QueryProvenanceRequest, QueryProvenanceResponse,
};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct MyBridge {}

#[tonic::async_trait]
impl CathedralBridge for MyBridge {
    async fn ingest(&self, request: Request<IngestRequest>) -> Result<Response<IngestResponse>, Status> {
        let req = request.into_inner();
        println!("Ingested events for project: {}", req.project_id);

        let response = IngestResponse {
            success: true,
            message: "Events ingested".into(),
            events_accepted: req.events.len() as u32,
            rejected_event_ids: vec![],
        };
        Ok(Response::new(response))
    }

    async fn request_governance(&self, request: Request<GovernanceRequest>) -> Result<Response<GovernanceResponse>, Status> {
        let req = request.into_inner();
        println!("Governance request for project: {}", req.project_id);

        let response = GovernanceResponse {
            request_id: req.request_id,
            verdict: GovernanceVerdict::Approved as i32,
            rationale: "Auto-approved by stub".into(),
            conditions: vec![],
            evaluated_by: "stub".into(),
            evaluated_at: None,
        };
        Ok(Response::new(response))
    }

    async fn query_provenance(&self, _request: Request<QueryProvenanceRequest>) -> Result<Response<QueryProvenanceResponse>, Status> {
        Ok(Response::new(QueryProvenanceResponse {
            entries: vec![],
            has_more: false,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let bridge = MyBridge::default();

    println!("CathedralBridgeServer listening on {}", addr);

    Server::builder()
        .add_service(CathedralBridgeServer::new(bridge))
        .serve(addr)
        .await?;

    Ok(())
}
