extern crate protobuf;
extern crate grpcio;
extern crate futures;

pub mod protos;
pub mod storage;

use std::sync::Arc;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::collections::HashMap;

use protos::kvserver::{GetRequest, PutRequest, DeleteRequest, ScanRequest};
use protos::kvserver_grpc::KvClient;
use storage::{Key, Value};

struct Client {
    client: KvClient,
}

impl Client {
    pub fn new(host: String, port: u16) -> Self {
        let addr = format!("{}:{}", host, port);
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(addr.as_ref());
        let kv_client = KvClient::new(ch);

        Client {
            client: kv_client,
        }
    }

	pub fn get(&self, k: Key) -> Option<String> {
        let mut request = GetRequest::new();
        request.set_key(k);
        let ret = self.client.get(&request).expect("RPC failed");
        if ret.empty {
            return None;
        }
        Some(ret.value.clone())
    }

    pub fn put(&self, k: Key, v: Value) {
        let mut put_req = PutRequest::new();
        put_req.set_key(k);
        put_req.set_value(v);
        self.client.put(&put_req).expect("RPC failed");
    }

    pub fn delete(&self, k: Key) {
        let mut del_req = DeleteRequest::new();
        del_req.set_key(k);
        self.client.delete(&del_req).expect("RPC failed");
    }

    pub fn scan(&self, k1: Key, k2: Key) -> Option<HashMap<String,String>> {
        let mut request = ScanRequest::new();
        request.set_key_begin(k1);
        request.set_key_end(k2);
        let ret = self.client.scan(&request).expect("RPC failed");
        if ret.empty {
            return None;
        }
        Some(ret.key_value)
    }

}

fn main(){
    let test_host = String::from("127.0.0.1");
    let test_port = 20001;

    let client = Client::new(test_host.clone(), test_port);

    client.put("aa".to_string(),"aaaaa".to_string());
    client.put("bb".to_string(),"bbbbb".to_string());
    client.put("cc".to_string(),"ccccc".to_string());
    let ret = client.get("aa".to_string());
    match ret {
        Some(v) => println!("get:aa's value:{}", v),
        None => println!("get None")
    }
    client.delete("aa".to_string());
    client.put("dd".to_string(),"ccccc".to_string());
    client.put("dd".to_string(),"ddddd".to_string());
    let ret = client.scan("aa".to_string(),"dd".to_string());
    match ret {
        Some(v) => println!("scan{{ {:?} }}",v),
        None => println!("scan None")
    }
}


