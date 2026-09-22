//! WebRTC Handler for Screen Share
//!
//! Manages WebRTC peer connections and data channels using webrtc-rs.

use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::constants::DEFAULT_STUN_SERVERS;
use crate::error::{Error, Result};

/// WebRTC connection state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

impl From<RTCPeerConnectionState> for ConnectionState {
    fn from(state: RTCPeerConnectionState) -> Self {
        match state {
            RTCPeerConnectionState::New => Self::New,
            RTCPeerConnectionState::Connecting => Self::Connecting,
            RTCPeerConnectionState::Connected => Self::Connected,
            RTCPeerConnectionState::Disconnected => Self::Disconnected,
            RTCPeerConnectionState::Failed => Self::Failed,
            RTCPeerConnectionState::Closed => Self::Closed,
            RTCPeerConnectionState::Unspecified => Self::New,
        }
    }
}

/// ICE candidate for signaling
#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

/// WebRTC handler
pub struct WebRtcHandler {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<RwLock<Option<Arc<RTCDataChannel>>>>,
    state: Arc<RwLock<ConnectionState>>,
    ice_candidates_tx: mpsc::Sender<IceCandidate>,
    data_tx: broadcast::Sender<Vec<u8>>,
}

impl WebRtcHandler {
    /// Create new WebRTC handler
    pub async fn new(
        stun_servers: Option<Vec<String>>,
    ) -> Result<(Self, mpsc::Receiver<IceCandidate>, broadcast::Receiver<Vec<u8>>)> {
        // Set up media engine (not used for data-only, but required)
        let mut media_engine = MediaEngine::default();
        
        // Set up interceptors
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)
            .map_err(|e| Error::WebRtc(format!("Failed to register interceptors: {}", e)))?;
        
        // Build API
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();
        
        // Configure ICE servers
        let servers = stun_servers.unwrap_or_else(|| {
            DEFAULT_STUN_SERVERS.iter().map(|s| s.to_string()).collect()
        });
        
        let ice_servers: Vec<RTCIceServer> = servers
            .iter()
            .map(|url| RTCIceServer {
                urls: vec![url.clone()],
                ..Default::default()
            })
            .collect();
        
        let config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };
        
        // Create peer connection
        let peer_connection = api
            .new_peer_connection(config)
            .await
            .map_err(|e| Error::PeerConnection(format!("Failed to create peer connection: {}", e)))?;
        
        let peer_connection = Arc::new(peer_connection);
        let state = Arc::new(RwLock::new(ConnectionState::New));
        let data_channel: Arc<RwLock<Option<Arc<RTCDataChannel>>>> = Arc::new(RwLock::new(None));
        
        // Create channels
        let (ice_tx, ice_rx) = mpsc::channel(32);
        let (data_tx, data_rx) = broadcast::channel(64);
        
        // Set up state change handler
        let state_clone = state.clone();
        peer_connection.on_peer_connection_state_change(Box::new(move |s| {
            let state = state_clone.clone();
            Box::pin(async move {
                info!("Peer connection state changed: {:?}", s);
                *state.write().await = ConnectionState::from(s);
            })
        }));
        
        // Set up ICE candidate handler
        let ice_tx_clone = ice_tx.clone();
        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            let ice_tx = ice_tx_clone.clone();
            Box::pin(async move {
                if let Some(c) = candidate {
                    debug!("New ICE candidate: {}", c.to_json().unwrap_or_default().candidate);
                    if let Ok(json) = c.to_json() {
                        let _ = ice_tx.send(IceCandidate {
                            candidate: json.candidate,
                            sdp_mid: json.sdp_mid,
                            sdp_mline_index: json.sdp_mline_index,
                        }).await;
                    }
                }
            })
        }));
        
        // Set up data channel handler
        let data_channel_clone = data_channel.clone();
        let data_tx_clone = data_tx.clone();
        peer_connection.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
            let data_channel = data_channel_clone.clone();
            let data_tx = data_tx_clone.clone();
            
            info!("Data channel opened: {}", dc.label());
            
            Box::pin(async move {
                *data_channel.write().await = Some(Arc::clone(&dc));
                
                // Set up message handler
                let dc_msg = Arc::clone(&dc);
                dc_msg.on_message(Box::new(move |msg: DataChannelMessage| {
                    let data_tx = data_tx.clone();
                    Box::pin(async move {
                        let _ = data_tx.send(msg.data.to_vec());
                    })
                }));
                
                dc.on_close(Box::new(|| {
                    info!("Data channel closed");
                    Box::pin(async {})
                }));
            })
        }));
        
        Ok((
            Self {
                peer_connection,
                data_channel,
                state,
                ice_candidates_tx: ice_tx,
                data_tx,
            },
            ice_rx,
            data_rx,
        ))
    }
    
    /// Get current connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }
    
    /// Set remote offer and create answer
    pub async fn accept_offer(&self, sdp: &str) -> Result<String> {
        let offer = RTCSessionDescription::offer(sdp.to_string())
            .map_err(|e| Error::WebRtc(format!("Invalid SDP offer: {}", e)))?;
        
        self.peer_connection
            .set_remote_description(offer)
            .await
            .map_err(|e| Error::WebRtc(format!("Failed to set remote description: {}", e)))?;
        
        let answer = self.peer_connection
            .create_answer(None)
            .await
            .map_err(|e| Error::WebRtc(format!("Failed to create answer: {}", e)))?;
        
        let sdp = answer.sdp.clone();
        
        self.peer_connection
            .set_local_description(answer)
            .await
            .map_err(|e| Error::WebRtc(format!("Failed to set local description: {}", e)))?;
        
        Ok(sdp)
    }
    
    /// Add remote ICE candidate
    pub async fn add_ice_candidate(
        &self,
        candidate: &str,
        sdp_mid: Option<&str>,
        sdp_mline_index: Option<u16>,
    ) -> Result<()> {
        let init = RTCIceCandidateInit {
            candidate: candidate.to_string(),
            sdp_mid: sdp_mid.map(String::from),
            sdp_mline_index,
            ..Default::default()
        };
        
        self.peer_connection
            .add_ice_candidate(init)
            .await
            .map_err(|e| Error::IceNegotiation(format!("Failed to add ICE candidate: {}", e)))?;
        
        Ok(())
    }
    
    /// Send data on the data channel
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        let dc = self.data_channel.read().await;
        let dc = dc.as_ref()
            .ok_or_else(|| Error::WebRtc("Data channel not open".into()))?;
        
        dc.send(&bytes::Bytes::copy_from_slice(data))
            .await
            .map_err(|e| Error::WebRtc(format!("Failed to send data: {}", e)))?;
        
        Ok(())
    }
    
    /// Close the peer connection
    pub async fn close(&self) -> Result<()> {
        self.peer_connection
            .close()
            .await
            .map_err(|e| Error::WebRtc(format!("Failed to close connection: {}", e)))?;
        
        Ok(())
    }
    
    /// Subscribe to data channel messages
    pub fn subscribe_data(&self) -> broadcast::Receiver<Vec<u8>> {
        self.data_tx.subscribe()
    }
}
