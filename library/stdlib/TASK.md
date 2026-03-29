# stdlib 任务清单

X 语言标准库 - 提供语言运行时必需的核心功能。

## 设计原则

根据 DESIGN_GOALS.md:
- 系统能力通过 C FFI 调用 C 标准库实现
- 核心类型用纯 X 语言实现
- 安全封装，错误通过 Result 处理
- 跨平台，屏蔽平台差异

## 已完成 ✅

| 模块 | 状态 | 描述 |
|------|------|------|
| prelude | ✅ 已完成 | 补全：print/println/assert/Some/None/Ok/Err 自动导入基本定义 |
| types/core | ✅ 已完成 | 补全 Option/Result/List/Map 完整实现 - 包含完整方法集合 |
| std.io | ✅ 已完成 | 输入输出：print/println 已经在 prelude，read_line 实现 |
| std.fs | ✅ 已完成 | 文件系统操作：打开/关闭/读写/删除/重命名/创建目录 |
| std.math | ✅ 已完成 | 数学函数：三角函数/指数对数/取整/常数/组合数等 |
| std.collections | ✅ 已完成 | 集合数据结构：HashSet, Stack, Queue, LinkedList, 堆（MinHeap/MaxHeap）, BST |
| std.unsafe | ✅ 已完成 | 不安全操作：内存分配/内存操作/指针操作/类型转换 |
| std.panic | ✅ 已完成 | panic 处理：可自定义处理函数，栈回溯占位，退出支持 |
| std.time | ✅ 已完成 | 日期时间处理：Duration, Instant, DateTime, 睡眠, 计时 |
| std.random | ✅ 已完成 | 随机数生成：XorShift64 RNG，正态分布，指数分布，洗牌，全局生成器 |
| std.encoding | ✅ 已完成 | 编码：hex（大小写），base64（标准和 URL 安全），UTF-8 验证 |
| std.hash | ✅ 已完成 | 哈希函数：FNV-1a, FNV-1, DJB2, SDBM, Jenkins, MurmurHash3，组合 |
| std.process | ✅ 已完成 | 进程管理：退出，执行命令，环境变量，工作目录，PID 获取 |
| std.net | ✅ 已完成 | 网络操作：TCP listener/stream, UDP socket，IPv4 地址解析 |

## 待完成 ⬜

| 序号 | 模块 | 优先级 | 描述 |
|------|------|--------|------|
| |  |  | **所有模块已完成！** |

## 验收标准

- [ ] 所有核心模块可用
- [ ] 能被 Zig 后端（或其他后端）正确编译
- [ ] 功能正确，能被用户程序调用

## 依赖

- 编译器完成，后端能正确编译 X 源代码
- C FFI 功能正常
