# FFI实现计划

## 目标
通过FFI（Foreign Function Interface）实现系统相关函数，将sys.x中的模拟实现替换为实际的系统调用。

## 实现步骤

### [ ] 任务1：扩展运行时文件，添加系统函数的C实现
- **优先级**：P0
- **依赖**：None
- **描述**：
  - 在x_runtime.h中添加系统相关函数的C实现
  - 实现环境变量、进程操作、系统信息等函数
- **成功标准**：
  - 所有系统函数都有对应的C实现
  - 函数签名与X语言的函数签名匹配
- **测试要求**：
  - `programmatic` TR-1.1：运行时编译通过
  - `programmatic` TR-1.2：函数调用能够返回正确的结果

### [ ] 任务2：更新sys.x文件，使用FFI调用
- **优先级**：P0
- **依赖**：任务1
- **描述**：
  - 修改sys.x中的函数实现，使用FFI调用C实现
  - 确保所有函数都正确调用对应的C函数
- **成功标准**：
  - sys.x中的所有函数都使用FFI调用
  - 函数签名保持不变
- **测试要求**：
  - `programmatic` TR-2.1：sys.x文件编译通过
  - `programmatic` TR-2.2：函数调用能够返回正确的结果

### [ ] 任务3：测试所有系统函数
- **优先级**：P1
- **依赖**：任务2
- **描述**：
  - 创建测试脚本，测试所有系统函数
  - 验证函数是否能够正确返回结果
- **成功标准**：
  - 所有系统函数都能正常工作
  - 测试脚本能够正确运行
- **测试要求**：
  - `programmatic` TR-3.1：所有测试用例通过
  - `human-judgement` TR-3.2：函数行为符合预期

## 详细实现计划

### 任务1：扩展运行时文件

#### 1.1 环境变量相关函数
- `get_env`：获取环境变量
- `set_env`：设置环境变量
- `unset_env`：删除环境变量
- `env_vars`：获取所有环境变量

#### 1.2 进程操作相关函数
- `getpid`：获取当前进程ID
- `getppid`：获取父进程ID
- `exit`：终止当前进程
- `system`：执行系统命令
- `command_output`：执行命令并获取输出

#### 1.3 系统信息相关函数
- `os_type`：获取操作系统类型
- `os_version`：获取操作系统版本
- `hostname`：获取主机名
- `arch`：获取系统架构
- `free_memory`：获取可用内存
- `total_memory`：获取总内存
- `cpu_count`：获取CPU核心数

#### 1.4 路径操作相关函数
- `current_dir`：获取当前工作目录
- `chdir`：改变工作目录
- `path_dirname`：获取路径的目录部分
- `path_basename`：获取路径的文件名部分
- `path_extension`：获取路径的扩展名
- `path_exists`：检查路径是否存在
- `is_file`：检查路径是否为文件
- `is_dir`：检查路径是否为目录

#### 1.5 临时文件相关函数
- `temp_file`：创建临时文件
- `temp_dir`：创建临时目录
- `get_temp_dir`：获取系统临时目录

#### 1.6 信号处理相关函数
- `signal`：注册信号处理器
- `kill`：发送信号到进程

#### 1.7 其他函数
- `syscall`：系统调用
- `uptime`：获取系统启动时间
- `random`：生成随机整数
- `random_float`：生成随机浮点数
- `srand`：设置随机数种子
- `getuid`：获取用户ID
- `getgid`：获取组ID
- `get_username`：获取用户名
- `get_groupname`：获取组名

### 任务2：更新sys.x文件

对于每个函数，将模拟实现替换为FFI调用，例如：

```x
function get_env(name: String): Option<String> {
  // 使用FFI调用C实现
  let value = __ffi_get_env(name)
  if value == "" {
    None
  } else {
    Some(value)
  }
}
```

### 任务3：测试系统函数

创建测试脚本，测试所有系统函数的功能，确保它们能够正确返回结果。