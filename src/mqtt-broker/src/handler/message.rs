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

use super::cache::CacheManager;
use common_base::tools::now_second;
use metadata_struct::mqtt::message::MQTTMessage;
use protocol::mqtt::common::PublishProperties;
use std::sync::Arc;

pub fn is_message_expire(message: &MQTTMessage) -> bool {
    return message.expiry_interval < now_second();
}

pub fn build_message_expire(
    cache_manager: &Arc<CacheManager>,
    publish_properties: &Option<PublishProperties>,
) -> u64 {
    if let Some(properties) = publish_properties {
        if let Some(expire) = properties.message_expiry_interval {
            if expire > 0 {
                return now_second() + expire as u64;
            }
        }
    }

    let cluster = cache_manager.get_cluster_info();
    return now_second() + cluster.protocol.max_message_expiry_interval;
}

#[cfg(test)]
mod tests {
    use crate::handler::{cache::CacheManager, message::build_message_expire};
    use clients::poll::ClientPool;
    use common_base::tools::now_second;
    use metadata_struct::mqtt::cluster::MQTTClusterDynamicConfig;
    use protocol::mqtt::common::PublishProperties;
    use std::sync::Arc;

    #[test]
    fn build_message_expire_test() {
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let mut cluster = MQTTClusterDynamicConfig::default();
        cluster.protocol.max_message_expiry_interval = 10;
        cache_manager.set_cluster_info(cluster);

        let publish_properties = None;
        let res = build_message_expire(&cache_manager, &publish_properties);
        assert_eq!(res, now_second() + 10);

        let mut publish_properties = PublishProperties::default();
        publish_properties.message_expiry_interval = Some(3);
        let res = build_message_expire(&cache_manager, &Some(publish_properties));
        assert_eq!(res, now_second() + 3);
    }
}
