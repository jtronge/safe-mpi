use safe_mpi;
use std::env;

fn main() {
    // TODO: Use clap for this
    let mut args = env::args();
    args.next();
    // Get the other address
    let address = args.next().unwrap();
    let send = &args.next().unwrap() == "send";
    let sm = safe_mpi::init(&address).expect("Failed to init safe_mpi");
    let comm = sm.world();
/*
    if send {
        comm.send(&[1, 2, 3, 4]);
    } else {
        let mut buf = [0; 4];
        comm.recv(&mut buf);
        println!("{:?}", buf);
    }
 */
}
