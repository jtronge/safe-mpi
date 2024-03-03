//! Point to point messaging base code and interface.
use std::future::Future;
use std::pin::pin;
use std::sync::{Arc, Mutex};
use smpi_base::{Result, Error, BufRead, BufWrite, Reachability, Provider};
use smpi_runtime::Runtime;

/// Internal data structure for managing P2P calls.
pub(crate) struct Manager {
    runtime: Arc<Mutex<Runtime>>,
    providers: Vec<Arc<Box<dyn Provider>>>,
}

impl Manager {
    pub(crate) fn new(runtime: Arc<Mutex<Runtime>>) -> Manager {
        Manager {
            runtime,
            providers: vec![],
        }
    }

    /// Determine the best provider for the target process.
    fn best_provider(&self, target_id: u64) -> Option<Arc<Box<dyn Provider>>> {
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

    pub(crate) fn send_nb<T: BufRead>(
        &self,
        data: T,
        target: u64,
    ) -> impl Future<Output = Result<T>> {
        let provider = self.best_provider(target);
        async move {
            if let Some(provider) = provider {
                pin!(provider)
                    .send_nb(&data, target)
                    .await?;
                Ok(data)
            } else {
                Err(Error::Unreachable)
            }
        }
    }

    /// Non-blocking receive a message from another process.
    pub(crate) fn recv_nb<T: BufWrite>(
        &self,
        data: T,
        source: u64,
    ) -> impl Future<Output = Result<T>> {
        let provider = self.best_provider(source);
        async move {
            if let Some(provider) = provider {
                pin!(provider)
                    .recv_nb(&data, source)
                    .await?;
                Ok(data)
            } else {
                Err(Error::Unreachable)
            }
        }
    }
}
