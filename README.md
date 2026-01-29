# Loongfang

> _龙牙_

简化[`axum`](https://github.com/tokio-rs/axum)及生态的集成！

## 配置示例

```toml
[general]
listen = "0.0.0.0:8000"
timezone = "Asia/Shanghai"

[logging]
level = "debug"    # trace > debug > info > warn > error
writer = "file"    # file | stdout
directory = "./log"
file_name_prefix = "loongfang.log"

[postgres]
url = "postgres://loongfang:100п9ƒ4п9@127.0.0.1:5432/loongfang"
max_connections = 10
min_connections = 1
acquire_timeout = 30    # 秒
idle_timeout = 600      # 秒
max_lifetime = 1800     # 秒

[redis]
url = "redis://127.0.0.1:6379"

```
