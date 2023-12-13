# Backend services for YUM project

## 生产环境说明

diesel 支持 SSL 不好, 目前需要数据库配置 require_secure_transport=OFF

docker 环境里需要安装 libssl-dev pkg-config ca-certificates 才能对外发送 https 请求

Dockerfile 参考 https://github.com/hseeberger/hello-rs/blob/main/Dockerfile


## Helm

**Install**
```shell
export KUBECONFIG=~/.kube/config-muse-ai
helm install yum ./chart -f chart/values.yaml -f chart/secrets/values.yaml --kube-context muse-ai-test --dry-run
```

正式安装的时候去掉 `--dry-run` 参数

**Upgrade**

```shell
export KUBECONFIG=~/.kube/config-muse-ai
helm upgrade yum ./chart -f chart/values.yaml -f chart/secrets/values.yaml --kube-context muse-ai-test --dry-run
```

## Comfy

输入输出要支持 base64 需要这个 node https://github.com/ramyma/A8R8_ComfyUI_nodes
