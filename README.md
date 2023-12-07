# Backend services for YUM project

diesel 支持 SSL 不好, 目前需要数据库配置 require_secure_transport=OFF
docker 环境里需要安装 libssl-dev pkg-config ca-certificates 才能对外发送 https 请求

Dockerfile 参考 https://github.com/hseeberger/hello-rs/blob/main/Dockerfile
