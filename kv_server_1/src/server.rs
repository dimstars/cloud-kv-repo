extern crate protobuf;
extern crate grpcio;
extern crate futures;

pub mod protos;
pub mod storage;

use std::io::Read;
use std::sync::Arc;
use std::{io, thread};

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink, Server as GrpcServer};

use protos::kvserver::{GetRequest, GetResponse, PutRequest, PutResponse, DeleteRequest, DeleteResponse, ScanRequest, ScanResponse};
use protos::kvserver_grpc::{self, Kv};

use storage::engine::Engine;

#[derive(Clone)]
struct KvService{
    engine: Engine,
}

impl KvService {
    pub fn new() -> Self {
        KvService {
            engine: Engine::new(),
        }
    }
}

impl Kv for KvService {
    fn get(&mut self, ctx: RpcContext, req: GetRequest, sink: UnarySink<GetResponse>) {
        let mut response = GetResponse::new();
        println!("Received GetRequest {{ {:?} }}", req);
        let engine = &self.engine;
        let ret = engine.get(req.key);
        match ret {
            Ok(op) => match op {
                Some(value) => {
                    response.set_value(value);
                    response.set_empty(false)
                }
                None => response.set_empty(true),
            }
            Err(_) => response.set_error(String::from("errors")),
        }

        let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn put(&mut self, ctx: RpcContext, req: PutRequest, sink: UnarySink<PutResponse>) {
        let mut response = PutResponse::new();
        println!("Received PutRequest {{ {:?} }}", req);
        let engine = &mut self.engine;
        let ret = engine.put(req.key, req.value);

        match ret {
            Ok(_) => (),
            Err(_) => response.set_error(String::from("errors")),
        }

        let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn delete(&mut self, ctx: RpcContext, req: DeleteRequest, sink: UnarySink<DeleteResponse>) {
        let mut response = DeleteResponse::new();
        println!("Received DeleteResponse {{ {:?} }}", req);
        let engine = &mut self.engine;
        let ret = engine.delete(req.key);
        match ret {
            Ok(_) => (),
            Err(_) => response.set_error(String::from("errors")),
        }
        let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn scan(&mut self, ctx: RpcContext, req: ScanRequest, sink: UnarySink<ScanResponse>) {
        let mut response = ScanResponse::new();
        println!("Received ScanRequest {{ {:?} }}", req);
        let engine = &self.engine;
        let ret = engine.scan(req.key_begin, req.key_end);
        match ret {
            Ok(op) => match op {
                Some(key_value) => {
                    response.set_key_value(key_value);
                    response.set_empty(false);
                }
                None => response.set_empty(true),
            }
            Err(_) => response.set_error(String::from("errors")),
        }

        let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

}

fn main(){
    let env = Arc::new(Environment::new(1));
    let service  = kvserver_grpc::create_kv(KvService::new());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 20001)
        .build()
        .unwrap();

    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();   
}
