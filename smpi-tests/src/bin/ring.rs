fn main() {
    smpi::main(|comm| async move {
        let size = comm.size();
        let id = comm.id();
        let next = (id + 1) % size;
        let prev = (id + size - 1) % size;

        let x: i32 = 0;
        if id == 0 {
            // Initiate the first send
            let x = comm.send_nb(x, next).await.unwrap();
            comm.recv_nb(x, prev).await.unwrap();
        } else {
            let mut x = comm.recv_nb(x, prev).await.unwrap();
            x += 1;
            println!("(id: {}) incremented x to {}", id, x);
            comm.send_nb(x, next).await.unwrap();
        }

        // println!("total processes: {}", comm.size());
        // println!("process ID: {}", comm.id());
        // println!("ciao, mondo");
    })
}
