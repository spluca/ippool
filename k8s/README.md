# Kubernetes Deployment for IP Pool

This directory contains Kubernetes manifests for deploying the IP Pool service.

## 📋 Prerequisites

- Kubernetes cluster (v1.19+)
- `kubectl` configured to access your cluster
- Container registry with the `ippool` image
- (Optional) Ingress controller (e.g., nginx-ingress) for external access
- (Optional) cert-manager for SSL/TLS certificates

## 🏗️ Architecture

The deployment consists of:

- **Deployment**: Single replica (stateful service) with health checks
- **Service**: ClusterIP service on port 8090
- **PersistentVolumeClaim**: 1Gi storage for IP allocation state
- **ConfigMap**: Environment configuration
- **Ingress**: (Optional) External access with SSL/TLS

## 🚀 Quick Start

### Option 1: Using kubectl

```bash
# Create namespace (optional)
kubectl create namespace ippool

# Apply all manifests
kubectl apply -f configmap.yaml
kubectl apply -f pvc.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml

# Optional: Apply ingress for external access
kubectl apply -f ingress.yaml
```

### Option 2: Using kustomize

```bash
# Edit kustomization.yaml first to set your namespace and image

# Preview what will be applied
kubectl kustomize .

# Apply with kustomize
kubectl apply -k .
```

## 📦 Building and Pushing the Container Image

Before deploying, you need to build and push the container image:

```bash
# Build the Docker image
cd ..
docker build -t ippool:latest .

# Tag for your registry
docker tag ippool:latest your-registry/ippool:latest

# Push to registry
docker push your-registry/ippool:latest
```

Update `kustomization.yaml` with your registry URL:
```yaml
images:
  - name: ippool
    newName: your-registry/ippool
    newTag: latest
```

## ⚙️ Configuration

### Environment Variables

Edit `configmap.yaml` to configure:

- `RUST_LOG`: Log level (default: `info`)
- `RUST_BACKTRACE`: Enable backtraces (default: `1`)

### Network Configuration

The network configuration is currently hardcoded in the application:
- Network: `172.16.0.0/24`
- Gateway: `172.16.0.1`

To change these, you'll need to modify the source code and rebuild the image.

### Resource Limits

Edit `deployment.yaml` to adjust resource limits:

```yaml
resources:
  requests:
    memory: "64Mi"
    cpu: "100m"
  limits:
    memory: "256Mi"
    cpu: "500m"
```

### Storage

The PVC requests 1Gi of storage. Edit `pvc.yaml` to change:

```yaml
spec:
  resources:
    requests:
      storage: 1Gi
  storageClassName: your-storage-class  # Uncomment and specify
```

## 🌐 External Access

### Using Ingress

1. Install an Ingress controller (e.g., nginx-ingress):
   ```bash
   kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml
   ```

2. Edit `ingress.yaml` and set your domain:
   ```yaml
   rules:
   - host: ippool.yourdomain.com
   ```

3. Apply the ingress:
   ```bash
   kubectl apply -f ingress.yaml
   ```

### Using NodePort (Alternative)

Change the Service type in `service.yaml`:

```yaml
spec:
  type: NodePort
  ports:
  - name: http
    port: 8090
    targetPort: 8090
    nodePort: 30090  # Optional: specify a port (30000-32767)
```

### Using LoadBalancer (Cloud)

Change the Service type in `service.yaml`:

```yaml
spec:
  type: LoadBalancer
  ports:
  - name: http
    port: 8090
    targetPort: 8090
```

## 🔍 Monitoring and Debugging

### Check Deployment Status

```bash
# Check pods
kubectl get pods -l app=ippool

# Check deployment
kubectl get deployment ippool

# Check service
kubectl get service ippool

# Check PVC
kubectl get pvc ippool-data
```

### View Logs

```bash
# View logs
kubectl logs -l app=ippool

# Follow logs
kubectl logs -l app=ippool -f

# View last 100 lines
kubectl logs -l app=ippool --tail=100
```

### Access the Service

```bash
# Port forward to local machine
kubectl port-forward svc/ippool 8090:8090

# Test health endpoint
curl http://localhost:8090/api/v1/health
```

### Describe Resources

```bash
# Describe pod (for troubleshooting)
kubectl describe pod -l app=ippool

# Describe deployment
kubectl describe deployment ippool

# View events
kubectl get events --sort-by=.metadata.creationTimestamp
```

## 🔧 Maintenance

### Update Deployment

```bash
# Update image
kubectl set image deployment/ippool ippool=your-registry/ippool:v1.1.0

# Or apply updated manifests
kubectl apply -f deployment.yaml

# Check rollout status
kubectl rollout status deployment/ippool

# View rollout history
kubectl rollout history deployment/ippool
```

### Rollback

```bash
# Rollback to previous version
kubectl rollout undo deployment/ippool

# Rollback to specific revision
kubectl rollout undo deployment/ippool --to-revision=2
```

### Restart Deployment

```bash
kubectl rollout restart deployment/ippool
```

### Scale Deployment

⚠️ **Warning**: This service maintains state, scaling to multiple replicas may cause conflicts.

```bash
# Scale (not recommended for this stateful service)
kubectl scale deployment ippool --replicas=1
```

## 🧪 Testing

### Health Check

```bash
# From within the cluster
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- curl http://ippool:8090/api/v1/health

# Expected response
{"status":"healthy"}
```

### Allocate IP

```bash
# Port forward first
kubectl port-forward svc/ippool 8090:8090

# Allocate IP
curl -X POST http://localhost:8090/api/v1/ip/allocate \
  -H "Content-Type: application/json" \
  -d '{"vm_id": "test-vm-001"}'

# List allocations
curl http://localhost:8090/api/v1/ip/allocations

# Get stats
curl http://localhost:8090/api/v1/ip/stats
```

## 🗑️ Cleanup

### Remove All Resources

```bash
# Using kubectl
kubectl delete -f ingress.yaml
kubectl delete -f service.yaml
kubectl delete -f deployment.yaml
kubectl delete -f pvc.yaml
kubectl delete -f configmap.yaml

# Using kustomize
kubectl delete -k .

# Delete namespace (if created)
kubectl delete namespace ippool
```

⚠️ **Warning**: Deleting the PVC will permanently delete all IP allocation data.

## 📊 Health Checks

The deployment includes:

### Liveness Probe
- Endpoint: `/api/v1/health`
- Initial delay: 5 seconds
- Period: 10 seconds
- Timeout: 3 seconds
- Failure threshold: 3

### Readiness Probe
- Endpoint: `/api/v1/health`
- Initial delay: 3 seconds
- Period: 5 seconds
- Timeout: 2 seconds
- Failure threshold: 2

## 🔒 Security Considerations

1. **Network Policies**: Consider adding NetworkPolicy to restrict access
2. **RBAC**: Create ServiceAccount with minimal required permissions
3. **Pod Security**: Use PodSecurityPolicy or PodSecurityStandards
4. **Secrets**: Don't store secrets in ConfigMaps (use Secrets instead)
5. **Resource Limits**: Set appropriate resource limits to prevent DoS
6. **TLS/SSL**: Use cert-manager for automatic certificate management

## 📝 Notes

- This is a **stateful service** - only one replica should run at a time
- The deployment uses `Recreate` strategy to ensure consistency
- IP allocation state is stored in the PersistentVolume
- Backup the PVC regularly to prevent data loss

## 🆘 Troubleshooting

### Pod Not Starting

```bash
# Check events
kubectl describe pod -l app=ippool

# Common issues:
# - Image pull errors: Check image name and registry credentials
# - PVC binding: Check if PVC is bound to a PV
# - Resource limits: Check if cluster has enough resources
```

### Health Check Failing

```bash
# Check logs
kubectl logs -l app=ippool

# Exec into pod
kubectl exec -it <pod-name> -- sh

# Test health endpoint internally
curl http://localhost:8090/api/v1/health
```

### PVC Not Binding

```bash
# Check PVC status
kubectl get pvc ippool-data

# Check available PVs
kubectl get pv

# If no PV available, you may need to:
# 1. Create a PV manually
# 2. Use a StorageClass with dynamic provisioning
# 3. Check if your cluster supports dynamic provisioning
```

## 📚 Additional Resources

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Kustomize Documentation](https://kustomize.io/)
- [NGINX Ingress Controller](https://kubernetes.github.io/ingress-nginx/)
- [cert-manager](https://cert-manager.io/)

---

**For production deployments, consider:**
- Setting up monitoring (Prometheus/Grafana)
- Implementing backup strategies for PVC
- Using GitOps (ArgoCD/FluxCD)
- Implementing proper RBAC and security policies
