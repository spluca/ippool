use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpPoolError {
    NoAvailableIps,
    IpNotFound,
    InvalidIp,
}

impl std::fmt::Display for IpPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpPoolError::NoAvailableIps => write!(f, "no available IPs in pool"),
            IpPoolError::IpNotFound => write!(f, "IP not found in allocations"),
            IpPoolError::InvalidIp => write!(f, "invalid IP address"),
        }
    }
}

impl std::error::Error for IpPoolError {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpAllocation {
    pub ip: String,
    pub vm_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IpPool {
    inner: Arc<RwLock<IpPoolInner>>,
}

#[derive(Debug)]
struct IpPoolInner {
    network: String,
    gateway: String,
    start: u8,
    end: u8,
    allocated: HashMap<String, String>, // IP -> VM_ID
    vm_to_ip: HashMap<String, String>,  // VM_ID -> IP
    available: Vec<String>,
}

impl IpPool {
    pub fn new(network: String, gateway: String) -> Self {
        let start = 2;
        let end = 254;
        let mut available = Vec::with_capacity((end - start + 1) as usize);

        // Use network prefix as-is (e.g., "172.16.0")
        let prefix = network.clone();

        // Initialize available IPs
        for i in start..=end {
            let ip = format!("{}.{}", prefix, i);
            available.push(ip);
        }

        let inner = IpPoolInner {
            network: prefix,
            gateway,
            start,
            end,
            allocated: HashMap::new(),
            vm_to_ip: HashMap::new(),
            available,
        };

        IpPool {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub async fn allocate_ip(&self, vm_id: String) -> Result<String, IpPoolError> {
        let mut inner = self.inner.write().await;

        // Check if VM already has an IP (idempotent)
        if let Some(ip) = inner.vm_to_ip.get(&vm_id) {
            return Ok(ip.clone());
        }

        // Check if there are available IPs
        if inner.available.is_empty() {
            return Err(IpPoolError::NoAvailableIps);
        }

        // Take first available IP
        let ip = inner.available.remove(0);

        // Mark as allocated
        inner.allocated.insert(ip.clone(), vm_id.clone());
        inner.vm_to_ip.insert(vm_id, ip.clone());

        Ok(ip)
    }

    pub async fn release_ip(&self, vm_id: &str) -> Result<(), IpPoolError> {
        let mut inner = self.inner.write().await;

        // Find IP for this VM
        let ip = inner
            .vm_to_ip
            .get(vm_id)
            .ok_or(IpPoolError::IpNotFound)?
            .clone();

        // Remove allocation
        inner.allocated.remove(&ip);
        inner.vm_to_ip.remove(vm_id);

        // Add back to available pool
        inner.available.push(ip);

        Ok(())
    }

    pub async fn release_ip_by_address(&self, ip: &str) -> Result<(), IpPoolError> {
        let mut inner = self.inner.write().await;

        // Validate IP is in our network
        if !Self::is_valid_ip(&inner.network, ip) {
            return Err(IpPoolError::InvalidIp);
        }

        // Find VM for this IP
        let vm_id = inner
            .allocated
            .get(ip)
            .ok_or(IpPoolError::IpNotFound)?
            .clone();

        // Remove allocation
        inner.allocated.remove(ip);
        inner.vm_to_ip.remove(&vm_id);

        // Add back to available pool
        inner.available.push(ip.to_string());

        Ok(())
    }

    pub async fn get_allocation(&self, vm_id: &str) -> Result<IpAllocation, IpPoolError> {
        let inner = self.inner.read().await;

        let ip = inner
            .vm_to_ip
            .get(vm_id)
            .ok_or(IpPoolError::IpNotFound)?
            .clone();

        Ok(IpAllocation {
            ip,
            vm_id: vm_id.to_string(),
            hostname: None,
        })
    }

    pub async fn list_allocations(&self) -> Vec<IpAllocation> {
        let inner = self.inner.read().await;

        inner
            .allocated
            .iter()
            .map(|(ip, vm_id)| IpAllocation {
                ip: ip.clone(),
                vm_id: vm_id.clone(),
                hostname: None,
            })
            .collect()
    }

    pub async fn get_stats(&self) -> serde_json::Value {
        let inner = self.inner.read().await;

        let total = (inner.end - inner.start + 1) as usize;
        let allocated = inner.allocated.len();
        let available = inner.available.len();
        let usage = (allocated as f64 / total as f64) * 100.0;

        serde_json::json!({
            "network": format!("{}.0/24", inner.network),
            "gateway": inner.gateway,
            "total": total,
            "allocated": allocated,
            "available": available,
            "usage": usage,
        })
    }

    fn is_valid_ip(network: &str, ip: &str) -> bool {
        // Parse IP address
        if ip.parse::<Ipv4Addr>().is_err() {
            return false;
        }

        // Check if IP starts with network prefix
        let network_prefix = format!("{}.", network);
        ip.starts_with(&network_prefix)
    }

    #[allow(dead_code)]
    pub async fn clear(&self) {
        let mut inner = self.inner.write().await;

        inner.allocated.clear();
        inner.vm_to_ip.clear();
        inner.available.clear();

        // Reinitialize available IPs
        for i in inner.start..=inner.end {
            let ip = format!("{}.{}", inner.network, i);
            inner.available.push(ip);
        }
    }

    pub async fn get_network(&self) -> String {
        let inner = self.inner.read().await;
        inner.network.clone()
    }

    pub async fn get_gateway(&self) -> String {
        let inner = self.inner.read().await;
        inner.gateway.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_ip_pool() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());
        let stats = pool.get_stats().await;

        assert_eq!(stats["total"].as_u64().unwrap(), 253);
        assert_eq!(stats["allocated"].as_u64().unwrap(), 0);
        assert_eq!(stats["available"].as_u64().unwrap(), 253);
    }

    #[tokio::test]
    async fn test_allocate_ip() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        let ip = pool.allocate_ip("vm-1".to_string()).await.unwrap();
        assert_eq!(ip, "172.16.0.2");

        let stats = pool.get_stats().await;
        assert_eq!(stats["allocated"].as_u64().unwrap(), 1);
        assert_eq!(stats["available"].as_u64().unwrap(), 252);
    }

    #[tokio::test]
    async fn test_allocate_ip_idempotent() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        let ip1 = pool.allocate_ip("vm-1".to_string()).await.unwrap();
        let ip2 = pool.allocate_ip("vm-1".to_string()).await.unwrap();

        assert_eq!(ip1, ip2);

        let stats = pool.get_stats().await;
        assert_eq!(stats["allocated"].as_u64().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_release_ip() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        pool.allocate_ip("vm-1".to_string()).await.unwrap();
        pool.release_ip("vm-1").await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats["allocated"].as_u64().unwrap(), 0);
        assert_eq!(stats["available"].as_u64().unwrap(), 253);
    }

    #[tokio::test]
    async fn test_release_ip_by_address() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        let ip = pool.allocate_ip("vm-1".to_string()).await.unwrap();
        pool.release_ip_by_address(&ip).await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats["allocated"].as_u64().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_allocation() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        let ip = pool.allocate_ip("vm-1".to_string()).await.unwrap();
        let allocation = pool.get_allocation("vm-1").await.unwrap();

        assert_eq!(allocation.ip, ip);
        assert_eq!(allocation.vm_id, "vm-1");
    }

    #[tokio::test]
    async fn test_list_allocations() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        pool.allocate_ip("vm-1".to_string()).await.unwrap();
        pool.allocate_ip("vm-2".to_string()).await.unwrap();
        pool.allocate_ip("vm-3".to_string()).await.unwrap();

        let allocations = pool.list_allocations().await;
        assert_eq!(allocations.len(), 3);
    }

    #[tokio::test]
    async fn test_no_available_ips() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        // Manually exhaust the pool
        pool.inner.write().await.available.clear();

        let result = pool.allocate_ip("vm-overflow".to_string()).await;
        assert!(matches!(result, Err(IpPoolError::NoAvailableIps)));
    }

    #[tokio::test]
    async fn test_concurrent_allocations() {
        let pool = IpPool::new("172.16.0".to_string(), "172.16.0.1".to_string());

        let mut handles = vec![];
        for i in 0..100 {
            let pool = pool.clone();
            let handle = tokio::spawn(async move {
                let vm_id = format!("vm-{}", i);
                pool.allocate_ip(vm_id).await
            });
            handles.push(handle);
        }

        let mut ips = std::collections::HashSet::new();
        for handle in handles {
            let ip = handle.await.unwrap().unwrap();
            ips.insert(ip);
        }

        // All IPs should be unique
        assert_eq!(ips.len(), 100);

        let stats = pool.get_stats().await;
        assert_eq!(stats["allocated"].as_u64().unwrap(), 100);
    }
}
