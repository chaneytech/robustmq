mod common;
#[cfg(test)]
mod tests {
    use crate::common::get_placement_addr;
    use clients::{
        placement::mqtt::call::{create_blacklist, delete_blacklist, list_blacklist},
        poll::ClientPool,
    };
    use common_base::tools::now_second;
    use metadata_struct::acl::mqtt_blacklist::{MQTTAclBlackList, MQTTAclBlackListType};
    use protocol::placement_center::generate::mqtt::{
        CreateBlacklistRequest, DeleteBlacklistRequest, ListBlacklistRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn mqtt_blacklist_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let cluster_name: String = "test1".to_string();

        let blacklist = MQTTAclBlackList {
            blacklist_type: MQTTAclBlackListType::User,
            resource_name: "loboxu".to_string(),
            end_time: now_second() + 100,
            desc: "loboxu test".to_string(),
        };

        let request = CreateBlacklistRequest {
            cluster_name: cluster_name.clone(),
            blacklist: blacklist.encode().unwrap(),
        };
        match create_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = ListBlacklistRequest {
            cluster_name: cluster_name.clone(),
        };

        match list_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.blacklists {
                    let tmp = serde_json::from_slice::<MQTTAclBlackList>(raw.as_slice()).unwrap();
                    if tmp.blacklist_type == blacklist.blacklist_type
                        && tmp.resource_name == blacklist.resource_name
                        && tmp.end_time == blacklist.end_time
                        && tmp.desc == blacklist.desc
                    {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = DeleteBlacklistRequest {
            cluster_name: cluster_name.clone(),
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name.clone(),
        };
        match delete_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        let request = ListBlacklistRequest {
            cluster_name: cluster_name.clone(),
        };

        match list_blacklist(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                let mut flag = false;
                for raw in data.blacklists {
                    let tmp = serde_json::from_slice::<MQTTAclBlackList>(raw.as_slice()).unwrap();
                    if tmp.blacklist_type == blacklist.blacklist_type
                        && tmp.resource_name == blacklist.resource_name
                        && tmp.end_time == blacklist.end_time
                        && tmp.desc == blacklist.desc
                    {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }
}
