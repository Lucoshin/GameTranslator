# 补丁格式 v1

补丁目录保持相对于游戏根目录的文件结构，并包含 `patch-manifest.json`。

```json
{
  "format_version": 1,
  "files": [
    {
      "relative_path": "data/Map001.json",
      "source_sha256": "...",
      "target_sha256": "..."
    }
  ]
}
```

## 应用规则

1. `relative_path` 必须保持在目标游戏根目录内。
2. 应用前计算现有文件 SHA-256，并与 `source_sha256` 比较。
3. 任一文件不匹配时停止整个操作，不能部分覆盖。
4. 写入后验证 `target_sha256`。
5. JSON 文件必须重新解析成功。

当前实现负责安全生成补丁；面向用户的补丁应用器将在后续里程碑实现。

