// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::net::SocketAddr;
use std::sync::Arc;

use grpc_clients::poll::ClientPool;
use log::{error, info};
use protocol::journal_server::codec::JournalEnginePacket;
use protocol::journal_server::journal_engine::{
    ApiKey, ApiVersion, CreateShardResp, CreateShardRespBody, DeleteShardResp, DeleteShardRespBody,
    GetActiveSegmentResp, GetActiveSegmentRespBody, GetClusterMetadataResp,
    GetClusterMetadataRespBody, JournalEngineError, OffsetCommitResp, OffsetCommitRespBody,
    ReadResp, ReadRespBody, RespHeader, WriteResp, WriteRespBody,
};

use super::cluster::ClusterHandler;
use super::data::DataHandler;
use super::shard::ShardHandler;
use crate::core::cache::CacheManager;
use crate::core::offset::OffsetManager;
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;

#[derive(Clone)]
pub struct Command {
    cluster_handler: ClusterHandler,
    shard_handler: ShardHandler,
    data_handler: DataHandler,
}

impl Command {
    pub fn new(
        client_poll: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        offset_manager: Arc<OffsetManager>,
    ) -> Self {
        let cluster_handler = ClusterHandler::new(cache_manager.clone());
        let shard_handler = ShardHandler::new(cache_manager.clone(), client_poll);
        let data_handler = DataHandler::new(cache_manager, offset_manager);
        Command {
            cluster_handler,
            shard_handler,
            data_handler,
        }
    }

    pub async fn apply(
        &self,
        connect_manager: Arc<ConnectionManager>,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        packet: JournalEnginePacket,
    ) -> Option<JournalEnginePacket> {
        info!("recv packet: {:?}", packet);
        match packet {
            /* Cluster Handler */
            JournalEnginePacket::GetClusterMetadataReq(request) => {
                let header = RespHeader {
                    api_key: ApiKey::GetClusterMetadata.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                let resp = GetClusterMetadataResp {
                    header: Some(header),
                    body: Some(GetClusterMetadataRespBody {
                        nodes: self.cluster_handler.get_cluster_metadata(),
                    }),
                };

                return Some(JournalEnginePacket::GetClusterMetadataResp(resp));
            }

            /* Shard Handler */
            JournalEnginePacket::CreateShardReq(request) => {
                let mut resp = CreateShardResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::CreateShard.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.create_shard(request).await {
                    Ok(replicas) => {
                        resp.body = Some(CreateShardRespBody {
                            active_segment: Some(replicas),
                        });
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                        resp.body = Some(CreateShardRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::CreateShardResp(resp));
            }

            JournalEnginePacket::DeleteShardReq(request) => {
                let mut resp = DeleteShardResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::DeleteShard.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.delete_shard(request).await {
                    Ok(replicas) => {
                        resp.body = Some(DeleteShardRespBody {});
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                        resp.body = Some(DeleteShardRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::DeleteShardResp(resp));
            }

            JournalEnginePacket::GetActiveSegmentReq(request) => {
                let mut resp = GetActiveSegmentResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::GetActiveSegment.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.shard_handler.active_segment(request).await {
                    Ok(segments) => {
                        resp.body = Some(GetActiveSegmentRespBody { segments });
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                        resp.body = Some(GetActiveSegmentRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::GetActiveSegmentResp(resp));
            }

            /* Data Handler */
            JournalEnginePacket::WriteReq(request) => {
                let mut resp = WriteResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::Write.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.data_handler.write(request).await {
                    Ok(status) => {
                        resp.body = Some(WriteRespBody { status });
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                        resp.body = Some(WriteRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::WriteResp(resp));
            }

            JournalEnginePacket::ReadReq(request) => {
                let mut resp = ReadResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::Read.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.data_handler.read(request).await {
                    Ok(status) => {
                        resp.body = Some(ReadRespBody {});
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                        resp.body = Some(ReadRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::ReadResp(resp));
            }

            JournalEnginePacket::OffsetCommitReq(request) => {
                let mut resp = OffsetCommitResp::default();
                let mut header = RespHeader {
                    api_key: ApiKey::OffsetCommit.into(),
                    api_version: ApiVersion::V0.into(),
                    ..Default::default()
                };
                match self.data_handler.offset_commit(request).await {
                    Ok(data) => {
                        resp.body = Some(OffsetCommitRespBody { resp: data });
                    }
                    Err(e) => {
                        header.error = Some(JournalEngineError {
                            code: 1,
                            error: e.to_string(),
                        });
                        resp.body = Some(OffsetCommitRespBody::default());
                    }
                }
                resp.header = Some(header);
                return Some(JournalEnginePacket::OffsetCommitResp(resp));
            }

            _ => {
                error!(
                    "server received an unrecognized request, request info: {:?}",
                    packet
                );
            }
        }
        None
    }
}