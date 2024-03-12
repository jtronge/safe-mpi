//! Point to point messaging base code and interface.
use std::future::Future;
use std::pin::pin;
use std::sync::{Arc, Mutex};
use smpi_base::{Result, Error, BufRead, BufWrite, Reachability, P2PProvider};
use smpi_runtime::Runtime;
use smpi_p2p_node::NodeP2P;

/// Internal data structure for managing P2P calls.
pub(crate) struct Manager {
    runtime: Arc<Mutex<Runtime>>,
    providers: Vec<Arc<Box<dyn P2PProvider>>>,
}

impl Manager {
    pub(crate) fn new(runtime: Arc<Mutex<Runtime>>) -> Manager {
        let providers: Vec<Arc<Box<dyn P2PProvider>>> = vec![
            Arc::new(Box::new(NodeP2P::new(Arc::clone(&runtime)))),
        ];
        Manager {
            runtime,
            providers,
        }
    }

    /// Determine the best provider for the target process.
    fn best_provider(&self, target_id: u64) -> Option<Arc<Box<dyn P2PProvider>>> {
        let mut best_i = None;
        let mut best_result = None;
        for (i, provider) in self.providers.iter().enumerate() {
            // Check the reachability of each provider and their estimated
            // latency and bandwidths
            match provider.reachability(target_id) {
                Reachability::Reachable(lat, bw) => {
                    if let Some((best_lat, best_bw)) = best_result {
                        if lat < best_lat && bw > best_bw {
                            let _ = best_i.insert(i);
                            let _ = best_result.insert((lat, bw));
                        }
                    } else {
                        let _ = best_i.insert(i);
                        let _ = best_result.insert((lat, bw));
                    }
                }
                _ => (),
            }
        }

        if let Some(i) = best_i {
            Some(Arc::clone(&self.providers[i]))
        } else {
            None
        }
    }

    /// Non-blocking send a message to another process.
    pub(crate) fn send_nb<T: BufRead>(
        &self,
        data: T,
        target: u64,
    ) -> impl Future<Output = Result<T>> {
        let provider = self.best_provider(target);
        async move {
            if let Some(provider) = provider {
                // SAFETY: The pointer passed from BufRead is valid as long as
                //         'data' is not moved
                unsafe {
                    pin!(provider)
                        .send_nb(data.ptr(), data.size(), T::type_id(), target)
                        .await?;
                }
                Ok(data)
            } else {
                Err(Error::Unreachable)
            }
        }
    }

    /// Non-blocking receive a message from another process.
    pub(crate) fn recv_nb<T: BufWrite>(
        &self,
        mut data: T,
        source: u64,
    ) -> impl Future<Output = Result<T>> {
        let provider = self.best_provider(source);
        async move {
            if let Some(provider) = provider {
                // SAFETY: The BufWrite pointer here is valid as long as 'data'
                //         is not moved.
                unsafe {
                    pin!(provider)
                        .recv_nb(data.ptr_mut(), data.size(), T::type_id(), source)
                        .await?;
                }
                Ok(data)
            } else {
                Err(Error::Unreachable)
            }
        }
    }
}
