# Docker Compose 集成测试报告

- 测试时间: 2026-03-01 17:03:06 (+08:00)
- 项目路径: /Users/dongowu/code/project/project_sonlana/solana-stablecoin-standard
- 目标配置: services/docker-compose.yml
- 执行命令: `docker-compose -f services/docker-compose.yml config`

## 1) 配置检查结果

- 语法与展开验证状态: **PASS**
- 命令退出码: **0**
- 解析后服务数量: **6**
- 警告行数: **1**

## 2) 关键检查项

- 已识别服务: mint-burn, compliance, indexer, webhook
- 共享卷: sss-data
- 依赖关系: indexer 依赖 webhook (service_healthy)
- 健康检查: mint-burn / compliance / webhook 均定义了 healthcheck

## 3) 结论

- 本次 `docker-compose config` 验证通过，配置可被 Docker Compose 正常解析。
- 存在 1 条非阻断警告（`version` 字段已废弃），建议后续移除 `version: "3.8"` 以避免歧义。

## 4) 证据文件

- 标准输出: `docker-compose-config.stdout.txt`
- 标准错误: `docker-compose-config.stderr.txt`

