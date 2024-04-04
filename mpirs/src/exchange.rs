use serde::{Deserialize, Serialize};
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

/// Exchange data to be sent between each process.
#[derive(Serialize, Deserialize)]
struct ExchangeData {
    /// Rank of process.
    rank: usize,

    /// Data to be exchanged.
    data: Vec<u8>,
}

/// Attempt to connect to the stream multiple times.
fn stream_connect(addr: &str) -> TcpStream {
    loop {
        match TcpStream::connect(addr) {
            Ok(stream) => return stream,
            Err(_) => (),
        }
    }
}

/// Receive a single address on the listener and ensure it has the proper rank.
fn recv_address(listener: &TcpListener, rank: usize) -> Vec<u8> {
    let (mut stream, _) = listener
        .accept()
        .expect("failed to accept new connection for address exchange");
    let data: ExchangeData =
        serde_json::from_reader(&mut stream).expect("failed to deserialize incoming data");
    assert_eq!(data.rank, rank);
    data.data
}

/// Perform the address exchange between this rank and all other processes.
pub fn address_exchange(rank: usize, conn_list: &[String], addr: &[u8]) -> Vec<Option<Vec<u8>>> {
    // This basically does a naïve all-to-all of the addresses with TCP streams.
    let local_addr = &conn_list[rank];
    let listener = TcpListener::bind(local_addr).expect("failed to bind to address");
    listener.set_nonblocking(true);

    let mut addrs = vec![None; conn_list.len()];
    let mut count = 0;
    let mut next_rank = 0;
    // Continue looping until all addresses have been sent and received.
    while count < (conn_list.len() - 1) || next_rank < conn_list.len() {
        // Try to receive an address.
        match listener.accept() {
            Ok((mut stream, _)) => {
                let data: ExchangeData = serde_json::from_reader(&mut stream)
                    .expect("failed to deserialize incoming data");
                addrs[data.rank].insert(data.data);
                count += 1;
            }
            Err(ref err) if err.kind() != io::ErrorKind::WouldBlock => {
                panic!("io error: {err}");
            }
            _ => (),
        }

        // Now try to send an address.
        if next_rank >= conn_list.len() {
            continue;
        } else if next_rank == rank {
            next_rank += 1;
            continue;
        }
        let exchange_addr = &conn_list[next_rank];
        match TcpStream::connect(exchange_addr) {
            Ok(mut stream) => {
                serde_json::to_writer(
                    &mut stream,
                    &ExchangeData {
                        rank,
                        data: addr.to_vec(),
                    },
                )
                .expect("failed to send address data");
                stream.flush().expect("failed to flush stream");
                next_rank += 1;
            }
            Err(ref err) if err.kind() != io::ErrorKind::ConnectionRefused => {
                panic!("io error: {err}");
            }
            Err(_) => (),
        }
    }
    addrs
}
